use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILENAME: &str = "settings.json";
const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub schema_version: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub tray: TraySettings,
    pub polling: PollingSettings,
    pub popup: PopupSettings,
}

fn default_theme() -> String {
    "glass".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraySettings {
    pub items: Vec<String>,
    pub separator: String,
    pub show_labels: bool,
    pub show_units: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingSettings {
    pub ai_interval_sec: u64,
    pub system_interval_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionConfig {
    pub visible: bool,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupSettings {
    pub sections: HashMap<String, SectionConfig>,
}

impl Default for UserSettings {
    fn default() -> Self {
        let mut sections = HashMap::new();
        sections.insert("ai_usage".into(), SectionConfig { visible: true, order: 0 });
        sections.insert("compute".into(), SectionConfig { visible: true, order: 1 });
        sections.insert("storage_network".into(), SectionConfig { visible: true, order: 2 });
        sections.insert("hardware".into(), SectionConfig { visible: true, order: 3 });
        sections.insert("devices".into(), SectionConfig { visible: true, order: 4 });

        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            theme: "glass".into(),
            tray: TraySettings {
                items: vec!["cpu".into(), "temp_cpu".into(), "battery".into()],
                separator: " | ".into(),
                show_labels: true,
                show_units: true,
            },
            polling: PollingSettings {
                ai_interval_sec: 300,
                system_interval_sec: 3,
            },
            popup: PopupSettings { sections },
        }
    }
}

pub fn load_settings(app_data_dir: &PathBuf) -> UserSettings {
    let path = app_data_dir.join(SETTINGS_FILENAME);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| {
            let defaults = UserSettings::default();
            let _ = save_settings(app_data_dir, &defaults);
            defaults
        }),
        Err(_) => {
            let defaults = UserSettings::default();
            let _ = save_settings(app_data_dir, &defaults);
            defaults
        }
    }
}

pub fn save_settings(app_data_dir: &PathBuf, settings: &UserSettings) -> Result<(), String> {
    let path = app_data_dir.join(SETTINGS_FILENAME);
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write settings: {e}"))?;
    Ok(())
}
