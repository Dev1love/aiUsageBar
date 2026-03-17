use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::api::UsageData;

/// Wrapper for thread-safe database access.
pub struct Database(pub Mutex<Connection>);

/// A single daily-aggregated snapshot for chart display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySnapshot {
    pub date: String,
    pub five_hour_util: f64,
    pub seven_day_util: f64,
    pub extra_usage_util: f64,
}

/// Open (or create) the SQLite database at the app data directory.
pub fn open_database(app_data_dir: PathBuf) -> Result<Connection, String> {
    std::fs::create_dir_all(&app_data_dir).map_err(|e| format!("Failed to create app data dir: {e}"))?;

    let db_path = app_data_dir.join("claudebar.db");
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {e}"))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS usage_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            five_hour_util REAL NOT NULL,
            seven_day_util REAL NOT NULL,
            extra_usage_util REAL NOT NULL DEFAULT 0.0
        );
        CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON usage_snapshots(timestamp);",
    )
    .map_err(|e| format!("Failed to create table: {e}"))?;

    Ok(conn)
}

/// Insert a usage snapshot from the latest poll.
pub fn insert_snapshot(conn: &Connection, usage: &UsageData) {
    let extra_util = usage.extra_usage.utilization.unwrap_or(0.0);
    let result = conn.execute(
        "INSERT INTO usage_snapshots (five_hour_util, seven_day_util, extra_usage_util) VALUES (?1, ?2, ?3)",
        params![usage.five_hour.utilization, usage.seven_day.utilization, extra_util],
    );
    if let Err(e) = result {
        eprintln!("Failed to insert snapshot: {e}");
    }
}

/// Get daily aggregated snapshots (max utilization per day) for the last N days.
pub fn get_daily_snapshots(conn: &Connection, days: i32) -> Vec<DailySnapshot> {
    let mut stmt = match conn.prepare(
        "SELECT date(timestamp) as day,
                MAX(five_hour_util) as max_five,
                MAX(seven_day_util) as max_seven,
                MAX(extra_usage_util) as max_extra
         FROM usage_snapshots
         WHERE timestamp >= datetime('now', ?1)
         GROUP BY date(timestamp)
         ORDER BY day ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare daily snapshots query: {e}");
            return Vec::new();
        }
    };

    let offset = format!("-{days} days");
    let rows = stmt.query_map(params![offset], |row| {
        Ok(DailySnapshot {
            date: row.get(0)?,
            five_hour_util: row.get(1)?,
            seven_day_util: row.get(2)?,
            extra_usage_util: row.get(3)?,
        })
    });

    match rows {
        Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("Failed to query daily snapshots: {e}");
            Vec::new()
        }
    }
}
