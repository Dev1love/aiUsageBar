use crate::settings::TraySettings;
use crate::system_monitor::SystemMetrics;
use crate::api::AllUsage;

pub fn format_tray_title(
    settings: &TraySettings,
    sys: &SystemMetrics,
    ai: &Option<AllUsage>,
) -> String {
    let parts: Vec<String> = settings
        .items
        .iter()
        .filter_map(|item| format_item(item, settings, sys, ai))
        .collect();
    parts.join(&settings.separator)
}

fn format_item(
    key: &str,
    settings: &TraySettings,
    sys: &SystemMetrics,
    ai: &Option<AllUsage>,
) -> Option<String> {
    let (label, value) = match key {
        "cpu" => ("CPU", format!("{:.0}%", sys.cpu.overall)),
        "ram" => ("RAM", format!("{:.1}G", sys.ram.used_gb)),
        "temp_cpu" => {
            let temp = sys.temps.iter().find(|t| t.name.contains("CPU"))?;
            ("", format!("{:.0}°C", temp.value))
        }
        "temp_gpu" => {
            let temp = sys.temps.iter().find(|t| t.name.contains("GPU"))?;
            ("", format!("{:.0}°C", temp.value))
        }
        "fan" => {
            let fan = sys.fans.first()?;
            ("", format!("{}rpm", fan.rpm))
        }
        "battery" => {
            let batt = sys.battery.as_ref()?;
            ("", format!("{:.0}%🔋", batt.percent))
        }
        "net_down" => ("↓", format_bytes(sys.network.download_speed)),
        "net_up" => ("↑", format_bytes(sys.network.upload_speed)),
        "disk_free" => {
            let free = sys.disk.total_gb - sys.disk.used_gb;
            ("", format!("{:.0}G free", free))
        }
        "ai_session" => {
            let claude = ai.as_ref()?.claude.as_ref()?;
            ("S", format!("{:.0}%", claude.five_hour.utilization))
        }
        "ai_weekly" => {
            let claude = ai.as_ref()?.claude.as_ref()?;
            ("W", format!("{:.0}%", claude.seven_day.utilization))
        }
        _ => return None,
    };

    if settings.show_labels && !label.is_empty() {
        Some(format!("{} {}", label, value))
    } else {
        Some(value)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{}K", bytes / 1024)
    } else {
        format!("{}B", bytes)
    }
}
