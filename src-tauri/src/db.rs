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

/// A single battery history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySnapshot {
    pub timestamp: String,
    pub percent: f32,
    pub health_percent: f32,
    pub cycle_count: u32,
    pub charging: bool,
}

/// A single daily network usage record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDaily {
    pub date: String,
    pub total_download_bytes: u64,
    pub total_upload_bytes: u64,
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
        CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON usage_snapshots(timestamp);

        CREATE TABLE IF NOT EXISTS battery_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            percent REAL NOT NULL,
            health_percent REAL NOT NULL,
            cycle_count INTEGER NOT NULL,
            charging INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_battery_timestamp ON battery_history(timestamp);

        CREATE TABLE IF NOT EXISTS network_daily (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL UNIQUE,
            total_download_bytes INTEGER NOT NULL DEFAULT 0,
            total_upload_bytes INTEGER NOT NULL DEFAULT 0
        );",
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

/// Insert a battery snapshot.
pub fn insert_battery_snapshot(
    conn: &Connection,
    percent: f32,
    health_percent: f32,
    cycle_count: u32,
    charging: bool,
) {
    let result = conn.execute(
        "INSERT INTO battery_history (percent, health_percent, cycle_count, charging) VALUES (?1, ?2, ?3, ?4)",
        params![percent, health_percent, cycle_count, charging as i32],
    );
    if let Err(e) = result {
        eprintln!("Failed to insert battery snapshot: {e}");
    }
}

/// Get battery history for the last N days.
pub fn get_battery_history(conn: &Connection, days: i32) -> Vec<BatterySnapshot> {
    let mut stmt = match conn.prepare(
        "SELECT timestamp, percent, health_percent, cycle_count, charging
         FROM battery_history
         WHERE timestamp >= datetime('now', ?1)
         ORDER BY timestamp ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare battery history query: {e}");
            return Vec::new();
        }
    };

    let offset = format!("-{days} days");
    let rows = stmt.query_map(params![offset], |row| {
        Ok(BatterySnapshot {
            timestamp: row.get(0)?,
            percent: row.get(1)?,
            health_percent: row.get(2)?,
            cycle_count: row.get(3)?,
            charging: row.get::<_, i32>(4)? != 0,
        })
    });

    match rows {
        Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("Failed to query battery history: {e}");
            Vec::new()
        }
    }
}

/// Accumulate network bytes for today (upsert).
pub fn insert_network_daily(conn: &Connection, download_bytes: u64, upload_bytes: u64) {
    let result = conn.execute(
        "INSERT INTO network_daily (date, total_download_bytes, total_upload_bytes)
         VALUES (date('now'), ?1, ?2)
         ON CONFLICT(date) DO UPDATE SET
             total_download_bytes = total_download_bytes + ?1,
             total_upload_bytes = total_upload_bytes + ?2",
        params![download_bytes, upload_bytes],
    );
    if let Err(e) = result {
        eprintln!("Failed to insert network daily: {e}");
    }
}

/// Get daily network usage for the last N days.
pub fn get_network_daily(conn: &Connection, days: i32) -> Vec<NetworkDaily> {
    let mut stmt = match conn.prepare(
        "SELECT date, total_download_bytes, total_upload_bytes
         FROM network_daily
         WHERE date >= date('now', ?1)
         ORDER BY date ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare network daily query: {e}");
            return Vec::new();
        }
    };

    let offset = format!("-{days} days");
    let rows = stmt.query_map(params![offset], |row| {
        Ok(NetworkDaily {
            date: row.get(0)?,
            total_download_bytes: row.get(1)?,
            total_upload_bytes: row.get(2)?,
        })
    });

    match rows {
        Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("Failed to query network daily: {e}");
            Vec::new()
        }
    }
}
