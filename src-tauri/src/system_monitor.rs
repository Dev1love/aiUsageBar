use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub overall: f32,
    pub per_core: Vec<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RamMetrics {
    pub used_gb: f64,
    pub total_gb: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub used_gb: f64,
    pub total_gb: f64,
    pub read_speed: u64,
    pub write_speed: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetMetrics {
    pub download_speed: u64,
    pub upload_speed: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatteryMetrics {
    pub percent: f32,
    pub health_percent: f32,
    pub cycle_count: u32,
    pub charging: bool,
    pub time_to_full: Option<f64>,
    pub time_to_empty: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TempSensor {
    pub name: String,
    pub value: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FanInfo {
    pub name: String,
    pub rpm: u32,
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BtDevice {
    pub name: String,
    pub connected: bool,
    pub battery: Option<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub ram: RamMetrics,
    pub disk: DiskMetrics,
    pub network: NetMetrics,
    pub battery: Option<BatteryMetrics>,
    pub temps: Vec<TempSensor>,
    pub fans: Vec<FanInfo>,
    pub bluetooth: Vec<BtDevice>,
}

/// Tracks previous network bytes for speed calculation.
pub struct NetworkTracker {
    prev_received: u64,
    prev_transmitted: u64,
}

impl NetworkTracker {
    pub fn new() -> Self {
        Self { prev_received: 0, prev_transmitted: 0 }
    }
}

pub fn refresh_sysinfo_metrics(
    sys: &mut System,
    networks: &mut sysinfo::Networks,
    net_tracker: &mut NetworkTracker,
    interval_secs: u64,
) -> (CpuMetrics, RamMetrics, DiskMetrics, NetMetrics) {
    // CPU
    sys.refresh_cpu_usage();
    let overall = sys.global_cpu_usage();
    let per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
    let cpu = CpuMetrics { overall, per_core };

    // RAM — must explicitly refresh memory
    sys.refresh_memory();
    let used_bytes = sys.used_memory();
    let total_bytes = sys.total_memory();
    let ram = RamMetrics {
        used_gb: used_bytes as f64 / 1_073_741_824.0,
        total_gb: total_bytes as f64 / 1_073_741_824.0,
    };

    // Disk
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut total_space = 0u64;
    let mut available_space = 0u64;
    for disk in disks.list() {
        if disk.mount_point() == std::path::Path::new("/") {
            total_space = disk.total_space();
            available_space = disk.available_space();
            break;
        }
    }
    let disk = DiskMetrics {
        used_gb: (total_space - available_space) as f64 / 1_073_741_824.0,
        total_gb: total_space as f64 / 1_073_741_824.0,
        read_speed: 0,
        write_speed: 0,
    };

    // Network — compute speed as delta / interval
    networks.refresh();
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    for (_name, data) in networks.iter() {
        total_rx += data.received();
        total_tx += data.transmitted();
    }
    let interval = interval_secs.max(1);
    let down_speed = (total_rx.saturating_sub(net_tracker.prev_received)) / interval;
    let up_speed = (total_tx.saturating_sub(net_tracker.prev_transmitted)) / interval;
    net_tracker.prev_received = total_rx;
    net_tracker.prev_transmitted = total_tx;

    let net = NetMetrics {
        download_speed: down_speed,
        upload_speed: up_speed,
    };

    (cpu, ram, disk, net)
}

/// Read battery info using the battery crate.
pub fn read_battery() -> Option<BatteryMetrics> {
    let manager = battery::Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;
    let batt = batteries.next()?.ok()?;

    let state = batt.state();
    let charging = state == battery::State::Charging;

    use battery::units::ratio::percent;
    use battery::units::time::minute;

    let soc = batt.state_of_charge().get::<percent>();
    let mut soh = batt.state_of_health().get::<percent>();

    // battery crate often returns wrong health on Apple Silicon — fallback to system_profiler
    if soh < 50.0 {
        soh = read_health_from_system_profiler().unwrap_or(soh);
    }

    let cycle_count = batt.cycle_count().unwrap_or_else(|| {
        read_cycle_count_from_ioreg().unwrap_or(0)
    });

    Some(BatteryMetrics {
        percent: soc,
        health_percent: soh,
        cycle_count,
        charging,
        time_to_full: if charging {
            batt.time_to_full().map(|t| t.get::<minute>() as f64)
        } else {
            None
        },
        time_to_empty: if !charging {
            batt.time_to_empty().map(|t| t.get::<minute>() as f64)
        } else {
            None
        },
    })
}

fn read_health_from_system_profiler() -> Option<f32> {
    let output = std::process::Command::new("system_profiler")
        .args(["SPPowerDataType", "-json"])
        .output()
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let power_data = json.get("SPPowerDataType")?.as_array()?.first()?;
    let health_info = power_data.get("sppower_battery_health_info")?;
    // "Maximum Capacity" is reported as a percentage string like "96%"
    // Key is "sppower_battery_health_maximum_capacity" with value like "96%"
    let max_capacity = health_info.get("sppower_battery_health_maximum_capacity")
        .or_else(|| health_info.get("sppower_battery_max_capacity"))
        .and_then(|v| v.as_str())?
        .trim_end_matches('%')
        .parse::<f32>()
        .ok()?;
    Some(max_capacity)
}

fn read_cycle_count_from_ioreg() -> Option<u32> {
    let output = std::process::Command::new("ioreg")
        .args(["-l", "-w0"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("CycleCount") && !line.contains("DesignCycleCount") {
            if let Some(val) = line.split('=').last() {
                return val.trim().parse::<u32>().ok();
            }
        }
    }
    None
}
