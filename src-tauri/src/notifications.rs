use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILENAME: &str = "notifications.json";

/// Persisted notification state — survives app restarts so we don't re-fire
/// alerts the user already dismissed in a previous session.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NotifState {
    /// `resets_at` value when last notified for five_hour 80% / 95%.
    pub five_hour_80_reset: Option<String>,
    pub five_hour_95_reset: Option<String>,
    /// `resets_at` value when last notified for seven_day 80% / 95%.
    pub seven_day_80_reset: Option<String>,
    pub seven_day_95_reset: Option<String>,
    /// Whether the monthly extra-usage 80% / 100% alerts already fired.
    pub extra_80_fired: bool,
    pub extra_100_fired: bool,
    /// Threshold-cross flags for battery. Set true after firing; cleared once
    /// the battery rises back above the threshold or is charging — so the
    /// next discharge cycle below will trigger anew, but a steady ≤10%
    /// reading no longer carpet-bombs the user.
    pub battery_low_fired: bool,
    pub battery_critical_fired: bool,
}

pub fn load(app_data_dir: &Path) -> NotifState {
    let path = app_data_dir.join(FILENAME);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(state: &NotifState, app_data_dir: &Path) {
    let path = app_data_dir.join(FILENAME);
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}
