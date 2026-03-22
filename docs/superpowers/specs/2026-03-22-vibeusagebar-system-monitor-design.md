# VibeUsageBar — System Monitor Integration Design

**Date:** 2026-03-22
**Status:** Approved
**Project:** aiUsageBar → VibeUsageBar

## Overview

Extend aiUsageBar from an AI-only usage tracker into a unified menubar app combining AI usage monitoring (Claude Code + Codex CLI) with full macOS system metrics. Rename to VibeUsageBar.

## Architecture

Three-layer architecture within a single .app bundle:

1. **Svelte Frontend** — UI rendering: AI usage, system metrics, settings, charts
2. **Rust Backend (Tauri v2)** — polling loops, state management, SQLite, tray icon
3. **Swift Dylib** — macOS-specific metrics via SMC/IOKit/IOBluetooth (C ABI FFI)

```
┌─────────────────────────────────────┐
│           VibeUsageBar.app          │
│                                     │
│  ┌──────────┐    ┌───────────────┐  │
│  │  Svelte  │◄──►│  Tauri/Rust   │  │
│  │ Frontend │IPC │   Backend     │  │
│  │          │    │               │  │
│  │ - AI bloc│    │ - AI polling  │  │
│  │ - System │    │   (5 min)     │  │
│  │   bloc   │    │ - Sys polling │  │
│  │ - Settings    │   (1-5 sec)   │  │
│  │ - Chart  │    │ - Settings    │  │
│  │          │    │ - SQLite DB   │  │
│  └──────────┘    │ - Tray text   │  │
│                  │               │  │
│                  │  ┌──────────┐ │  │
│                  │  │ Swift    │ │  │
│                  │  │ .dylib   │ │  │
│                  │  │ - SMC    │ │  │
│                  │  │ - Temps  │ │  │
│                  │  │ - Fans   │ │  │
│                  │  │ - BT     │ │  │
│                  │  └──────────┘ │  │
│                  └───────────────┘  │
└─────────────────────────────────────┘
```

### Metric Source Responsibility

| Metric | Source |
|---|---|
| CPU, RAM, Disk, Network | Rust (`sysinfo` crate) |
| GPU utilization | Rust (Metal/IOKit FFI) |
| Battery | Rust (IOKit via `core-foundation`) |
| CPU/GPU/SSD Temperature | **Swift dylib** (SMC) |
| Fan RPM | **Swift dylib** (SMC) |
| Bluetooth devices | **Swift dylib** (`IOBluetooth`) |

## Data Flow & Polling

Two independent polling loops running on tokio:

### AI Loop (5 min interval, unchanged)
- Fetch Claude API → `UsageData`
- Fetch Codex API → `CodexUsageData`
- Save snapshot → SQLite
- Emit `usage-update` → Frontend
- Update tray (if AI metrics enabled)

### System Loop (configurable 1-30 sec)
- `sysinfo.refresh_all()` → CPU, RAM, Disk, Net
- `swift_dylib::smc_read_temps()` + `smc_read_fans()` → Temps, Fans
- `swift_dylib::bt_get_devices()` → Bluetooth
- Emit `system-update` → Frontend
- Update tray (if system metrics enabled)

### State Management

```rust
struct AppState {
    ai_usage: Mutex<Option<AllUsage>>,
    system_metrics: Mutex<SystemMetrics>,
    settings: Mutex<UserSettings>,
}

struct SystemMetrics {
    cpu: CpuMetrics,          // overall %, per-core
    gpu: GpuMetrics,          // utilization %
    ram: RamMetrics,          // used/total GB
    disk: DiskMetrics,        // used/total, read/write speed
    network: NetMetrics,      // upload/download speed
    battery: BatteryMetrics,  // %, health, cycles, charging
    temps: Vec<TempSensor>,   // name + °C  (from Swift)
    fans: Vec<FanInfo>,       // name + RPM (from Swift)
    bluetooth: Vec<BtDevice>, // name + connected + battery
}
```

## Swift Dylib

### File: `src-tauri/swift/SystemMonitor.swift`

Exported C ABI functions returning JSON strings (simplest FFI — no complex struct marshaling):

```swift
@_cdecl("smc_read_temps")
func smcReadTemps() -> UnsafePointer<CChar>
// → [{"name":"CPU","value":54.2},{"name":"GPU","value":48.1}]

@_cdecl("smc_read_fans")
func smcReadFans() -> UnsafePointer<CChar>
// → [{"name":"Fan 0","rpm":1820,"min":1100,"max":6200}]

@_cdecl("bt_get_devices")
func btGetDevices() -> UnsafePointer<CChar>
// → [{"name":"AirPods Pro","connected":true,"battery":82}]

@_cdecl("free_string")
func freeString(_ ptr: UnsafePointer<CChar>)
```

### Rust FFI Bridge: `src-tauri/src/swift_bridge.rs`

```rust
#[link(name = "system_monitor")]
extern "C" {
    fn smc_read_temps() -> *const c_char;
    fn smc_read_fans() -> *const c_char;
    fn bt_get_devices() -> *const c_char;
    fn free_string(ptr: *const c_char);
}

pub fn get_temps() -> Vec<TempSensor> {
    unsafe {
        let ptr = smc_read_temps();
        let json = CStr::from_ptr(ptr).to_str().unwrap();
        let result = serde_json::from_str(json).unwrap_or_default();
        free_string(ptr);
        result
    }
}
```

### Build Integration

```bash
# In build.rs
swiftc -emit-library \
  -o libsystem_monitor.dylib \
  -module-name SystemMonitor \
  src-tauri/swift/SystemMonitor.swift \
  -framework IOKit \
  -framework IOBluetooth
```

Dylib placed in `Contents/Frameworks/` inside .app bundle. `@rpath` configured via `install_name_tool`.

## Settings System

### Storage: `~/Library/Application Support/com.vibeusagebar.app/settings.json`

```json
{
  "tray": {
    "items": ["cpu", "temp_cpu", "battery"],
    "separator": " | ",
    "show_labels": true,
    "show_units": true
  },
  "polling": {
    "ai_interval_sec": 300,
    "system_interval_sec": 3
  },
  "popup": {
    "sections": {
      "ai_usage": { "visible": true, "order": 0 },
      "compute": { "visible": true, "order": 1 },
      "storage_network": { "visible": true, "order": 2 },
      "hardware": { "visible": true, "order": 3 },
      "devices": { "visible": true, "order": 4 }
    }
  }
}
```

### Available Tray Items

| Key | Display Example |
|---|---|
| `cpu` | CPU 23% |
| `ram` | RAM 12.4G |
| `temp_cpu` | 54°C |
| `temp_gpu` | 48°C |
| `fan` | 1820rpm |
| `battery` | 87% 🔋 |
| `net_down` | ↓ 2.4M |
| `net_up` | ↑ 340K |
| `disk_free` | 124G free |
| `ai_session` | S:45% |
| `ai_weekly` | W:78% |

### Settings UI

- Gear icon in popup header → settings page
- Drag & drop list for tray items (enable/disable, reorder)
- Slider for system polling interval (1-30 sec)
- Checkboxes for popup sections (show/hide)
- Live preview of tray appearance

## Frontend — Popup UI

### Window: 350x600px (up from 300x520)

```
┌──────────────────────────────┐
│ VibeUsageBar          ⚙️     │  header + settings gear
├──────────────────────────────┤
│ AI Usage                     │
│ ┌──────────────────────────┐ │
│ │ Claude Code              │ │
│ │ Session  ████████░░ 78%  │ │
│ │          resets in 2h13m │ │
│ │ Weekly   ██████░░░░ 56%  │ │
│ │          resets in 4d    │ │
│ ├──────────────────────────┤ │
│ │ Codex CLI                │ │
│ │ Session  ███░░░░░░░ 23%  │ │
│ │          resets in 3h45m │ │
│ └──────────────────────────┘ │
├──────────────────────────────┤
│ Compute                      │
│  CPU   ██████░░░░  58%       │
│  GPU   ██░░░░░░░░  15%       │
│  RAM   █████████░  11.2/16G  │
├──────────────────────────────┤
│ Storage & Network            │
│  Disk  ██████████  412/500G  │
│  ↓ 2.4 MB/s    ↑ 340 KB/s   │
├──────────────────────────────┤
│ Hardware                     │
│  CPU  54°C   GPU  48°C       │
│  Fan  1820 rpm               │
│  🔋 87%  Health 96%  158cyc  │
├──────────────────────────────┤
│ Bluetooth                    │
│  AirPods Pro      ● 82%      │
│  Magic Mouse      ● 64%      │
│  Keyboard         ○ --       │
├──────────────────────────────┤
│  ▂▃▅▆▃▂▅  Weekly AI Chart    │
│  M T W T F S S               │
└──────────────────────────────┘
```

### Svelte Components

| Component | Status | Purpose |
|---|---|---|
| `UsageBar.svelte` | Existing | Reuse for AI and CPU/RAM/Disk bars |
| `ExtraUsage.svelte` | Existing | Claude Code extra credits |
| `WeeklyChart.svelte` | Existing | 7-day AI history chart |
| `SystemSection.svelte` | **New** | Section wrapper with header |
| `TempGauge.svelte` | **New** | Temperature with color coding |
| `NetworkSpeed.svelte` | **New** | Upload/download with auto-units |
| `BluetoothList.svelte` | **New** | BT device list with battery |
| `SettingsPage.svelte` | **New** | Settings page with tray config |

### Temperature Color Coding
- < 50°C — green
- 50-80°C — yellow
- > 80°C — red

## Implementation Phases

### Phase 1: Foundation
- Rename project to VibeUsageBar (tauri.conf.json, identifiers)
- Settings system (JSON storage + Rust state + Svelte settings page)
- Refactor popup layout (sections, new window size)
- Migrate old SQLite DB path if exists

### Phase 2: Rust System Metrics (sysinfo)
- Add `sysinfo` crate — CPU, RAM, Disk, Network
- Battery via IOKit
- System polling loop (separate from AI loop)
- New Svelte components: SystemSection, NetworkSpeed
- Emit `system-update` events to frontend

### Phase 3: Swift Dylib
- Swift → dylib build in `build.rs`
- SMC temperature + fan reading
- Bluetooth device enumeration
- Rust FFI bridge (`swift_bridge.rs`)
- New Svelte components: TempGauge, BluetoothList
- Graceful fallback if dylib fails to load

### Phase 4: Tray Icon
- Text-based menubar rendering (replace bitmap approach)
- Configurable items from settings
- Update from both polling loops

### Phase 5: Polish
- Drag & drop in settings for tray items
- Tray preview in settings
- GPU utilization (Metal Performance Statistics)
- Collapsible sections in popup

## Preserved (No Changes)
- AI polling logic (`api.rs`)
- Keychain integration (`keychain.rs`)
- SQLite schema (add new table, keep existing)
- WeeklyChart, UsageBar, ExtraUsage components

## Risks & Mitigation

| Risk | Mitigation |
|---|---|
| SMC unreadable without root on Apple Silicon | Fallback: show "N/A", metric unavailable |
| swiftc missing on build machine | Require Xcode Command Line Tools (standard for macOS dev) |
| dylib fails to load at runtime | Integration test at startup: call `smc_read_temps()`, if crash — disable Swift metrics |
| Popup too tall | Scroll + collapsible sections (Phase 5) |

## Dependencies

### New Rust Crates
| Crate | Purpose |
|---|---|
| `sysinfo` | CPU, RAM, Disk, Network metrics |
| `core-foundation` | macOS IOKit for battery |

### Swift Frameworks
| Framework | Purpose |
|---|---|
| `IOKit` | SMC access for temps/fans |
| `IOBluetooth` | Bluetooth device enumeration |
