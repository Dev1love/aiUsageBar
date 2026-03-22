# VibeUsageBar System Monitor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend aiUsageBar into VibeUsageBar — a unified macOS menubar app combining AI usage tracking with full system monitoring (CPU, RAM, disk, network, battery, temps, fans, bluetooth).

**Architecture:** Hybrid Rust + Swift dylib. Rust backend handles polling via `sysinfo` crate (CPU/RAM/disk/net) and `battery` crate. Swift dylib provides SMC temps/fans and Bluetooth via IOKit/IOBluetooth. Svelte 5 frontend renders all metrics in a single popup with configurable sections.

**Tech Stack:** Tauri v2, Svelte 5, Rust, Swift (dylib), SQLite, sysinfo crate, battery crate

**Spec:** `docs/superpowers/specs/2026-03-22-vibeusagebar-system-monitor-design.md`

---

## File Structure

### New files to create:
| File | Responsibility |
|---|---|
| `src-tauri/src/settings.rs` | Settings load/save/defaults, UserSettings struct |
| `src-tauri/src/system_monitor.rs` | System metrics polling via sysinfo + battery crates |
| `src-tauri/src/swift_bridge.rs` | FFI bridge to Swift dylib (temps, fans, bluetooth) |
| `src-tauri/src/tray_text.rs` | Configurable text-based tray title rendering |
| `src-tauri/swift/SystemMonitor.swift` | Swift dylib: SMC + IOBluetooth access |
| `src/lib/SystemSection.svelte` | Collapsible section wrapper component |
| `src/lib/TempGauge.svelte` | Temperature display with color coding |
| `src/lib/NetworkSpeed.svelte` | Upload/download speed with auto-units |
| `src/lib/BluetoothList.svelte` | Bluetooth device list |
| `src/lib/SettingsPage.svelte` | Settings UI (tray config, polling, sections) |
| `src/lib/types.ts` | Shared TypeScript interfaces for all metrics |

### Files to modify:
| File | Changes |
|---|---|
| `src-tauri/tauri.conf.json` | Rename to VibeUsageBar, new identifier |
| `src-tauri/Cargo.toml` | Add sysinfo, battery crates; rename package |
| `src-tauri/build.rs` | Add Swift dylib compilation |
| `src-tauri/Info.plist` | Add Bluetooth usage description |
| `src-tauri/src/lib.rs` | Add system polling loop, new state, new commands |
| `src-tauri/src/main.rs` | Update crate name |
| `src/routes/+page.svelte` | Add system sections, settings gear, new layout |
| `package.json` | Rename package |

---

## Phase 1: Foundation

### Task 1: Rename project to VibeUsageBar

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs:58` (notification title)
- Modify: `src-tauri/src/lib.rs:230` (quit menu item)
- Modify: `src-tauri/src/lib.rs:237` (tooltip)
- Modify: `src/routes/+page.svelte:72` (header title)
- Modify: `package.json`

- [ ] **Step 1: Update tauri.conf.json**

Change `productName` to `VibeUsageBar`, `identifier` to `com.vibeusagebar.app`:

```json
{
  "productName": "VibeUsageBar",
  "version": "0.2.0",
  "identifier": "com.vibeusagebar.app"
}
```

- [ ] **Step 2: Update Cargo.toml**

```toml
[package]
name = "vibeusagebar"
version = "0.2.0"
description = "macOS menubar utility for tracking AI usage and system metrics"

[lib]
name = "vibeusagebar_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

- [ ] **Step 3: Update main.rs**

```rust
fn main() {
    vibeusagebar_lib::run()
}
```

- [ ] **Step 4: Update all "aiUsageBar" strings in lib.rs**

Replace notification title (line 58), quit menu label (line 230), tooltip (line 237), window title (line 288) — all from `"aiUsageBar"` to `"VibeUsageBar"`.

- [ ] **Step 5: Update +page.svelte header**

Change `<h1>aiUsageBar</h1>` to `<h1>VibeUsageBar</h1>` (line 72).

- [ ] **Step 6: Update package.json name**

Change `"name": "claudebar"` to `"name": "vibeusagebar"`.

- [ ] **Step 7: Update Info.plist with Bluetooth description**

```xml
<key>LSUIElement</key>
<true/>
<key>NSBluetoothAlwaysUsageDescription</key>
<string>VibeUsageBar needs Bluetooth access to show connected device status and battery levels.</string>
```

- [ ] **Step 8: Verify it compiles**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build` (frontend only, no tauri)
Expected: Build succeeds

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: rename aiUsageBar to VibeUsageBar v0.2.0"
```

---

### Task 2: Settings system

**Files:**
- Create: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create settings.rs with types and defaults**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILENAME: &str = "settings.json";
const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub schema_version: u32,
    pub tray: TraySettings,
    pub polling: PollingSettings,
    pub popup: PopupSettings,
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

/// Load settings from app data dir, or create defaults if missing/corrupt.
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

/// Save settings to JSON file.
pub fn save_settings(app_data_dir: &PathBuf, settings: &UserSettings) -> Result<(), String> {
    let path = app_data_dir.join(SETTINGS_FILENAME);
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write settings: {e}"))?;
    Ok(())
}
```

- [ ] **Step 2: Add settings module and state to lib.rs**

At top of `lib.rs`, add `mod settings;` after `mod tray_icon;`.

Add `use std::sync::RwLock;` import.

Add settings state struct:

```rust
struct SettingsState(RwLock<settings::UserSettings>);
```

- [ ] **Step 3: Add Tauri commands for settings**

```rust
#[tauri::command]
fn get_settings(state: tauri::State<'_, SettingsState>) -> Result<settings::UserSettings, String> {
    let s = state.0.read().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

#[tauri::command]
fn save_settings_cmd(
    new_settings: settings::UserSettings,
    state: tauri::State<'_, SettingsState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    settings::save_settings(&app_data_dir, &new_settings)?;
    let mut s = state.0.write().map_err(|e| e.to_string())?;
    *s = new_settings;
    Ok(())
}
```

- [ ] **Step 4: Register settings in setup and invoke_handler**

In `run()`, load settings after resolving `app_data_dir` (line 216-217):

```rust
let user_settings = settings::load_settings(&app_data_dir);
app.manage(SettingsState(RwLock::new(user_settings)));
```

Add commands to invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![get_usage, get_history, get_settings, save_settings_cmd])
```

- [ ] **Step 5: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`
Expected: Compiles with no errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/settings.rs src-tauri/src/lib.rs
git commit -m "feat: add settings system with load/save/defaults"
```

---

### Task 3: Data migration from old app identifier

**Files:**
- Modify: `src-tauri/src/lib.rs` (setup block)

- [ ] **Step 1: Add migration logic before database init**

Insert before the `db::open_database` call in setup (around line 218):

```rust
// Migrate data from old aiUsageBar app if exists
let old_app_dir = app_data_dir
    .parent()
    .map(|p| p.join("com.aiusagebar.app"));
if let Some(ref old_dir) = old_app_dir {
    let old_db = old_dir.join("claudebar.db");
    let new_db = app_data_dir.join("claudebar.db");
    if old_db.exists() && !new_db.exists() {
        let _ = std::fs::create_dir_all(&app_data_dir);
        if let Err(e) = std::fs::copy(&old_db, &new_db) {
            eprintln!("Failed to migrate old database: {e}");
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: auto-migrate data from old aiUsageBar app dir"
```

---

### Task 4: Shared TypeScript types

**Files:**
- Create: `src/lib/types.ts`

- [ ] **Step 1: Create types.ts with all interfaces**

```typescript
// AI Usage types (moved from +page.svelte)
export interface PeriodUsage {
  utilization: number;
  resets_at: string;
}

export interface ExtraUsageData {
  is_enabled: boolean;
  monthly_limit: number | null;
  used_credits: number | null;
  utilization: number | null;
}

export interface UsageData {
  five_hour: PeriodUsage;
  seven_day: PeriodUsage;
  extra_usage: ExtraUsageData;
}

export interface CodexCredits {
  remaining: number;
  has_credits: boolean;
}

export interface CodexUsageData {
  primary: PeriodUsage;
  secondary: PeriodUsage | null;
  credits: CodexCredits | null;
}

export interface AllUsage {
  claude: UsageData | null;
  codex: CodexUsageData | null;
}

// System metrics types
export interface CpuMetrics {
  overall: number;
  per_core: number[];
}

export interface RamMetrics {
  used_gb: number;
  total_gb: number;
}

export interface DiskMetrics {
  used_gb: number;
  total_gb: number;
  read_speed: number;
  write_speed: number;
}

export interface NetMetrics {
  download_speed: number;
  upload_speed: number;
}

export interface BatteryMetrics {
  percent: number;
  health_percent: number;
  cycle_count: number;
  charging: boolean;
  time_to_full: number | null;
  time_to_empty: number | null;
}

export interface TempSensor {
  name: string;
  value: number;
}

export interface FanInfo {
  name: string;
  rpm: number;
  min: number;
  max: number;
}

export interface BtDevice {
  name: string;
  connected: boolean;
  battery: number | null;
}

export interface SystemMetrics {
  cpu: CpuMetrics;
  ram: RamMetrics;
  disk: DiskMetrics;
  network: NetMetrics;
  battery: BatteryMetrics | null;
  temps: TempSensor[];
  fans: FanInfo[];
  bluetooth: BtDevice[];
}

// Settings types
export interface TraySettings {
  items: string[];
  separator: string;
  show_labels: boolean;
  show_units: boolean;
}

export interface PollingSettings {
  ai_interval_sec: number;
  system_interval_sec: number;
}

export interface SectionConfig {
  visible: boolean;
  order: number;
}

export interface PopupSettings {
  sections: Record<string, SectionConfig>;
}

export interface UserSettings {
  schema_version: number;
  tray: TraySettings;
  polling: PollingSettings;
  popup: PopupSettings;
}
```

- [ ] **Step 2: Update +page.svelte to import from types.ts**

Replace the inline interface declarations (lines 9-41) with:

```typescript
import type { AllUsage, SystemMetrics, UserSettings } from '$lib/types';
```

- [ ] **Step 3: Verify frontend builds**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/routes/+page.svelte
git commit -m "feat: extract shared TypeScript types to types.ts"
```

---

### Task 5: Refactor popup layout with sections and scroll

**Files:**
- Create: `src/lib/SystemSection.svelte`
- Modify: `src/routes/+page.svelte`
- Modify: `src-tauri/src/lib.rs` (popup dimensions)

- [ ] **Step 1: Create SystemSection.svelte**

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, children }: {
    title: string;
    children: Snippet;
  } = $props();
</script>

<section class="section">
  <div class="section-header">
    <span class="section-title">{title}</span>
  </div>
  <div class="section-body">
    {@render children()}
  </div>
</section>

<style>
  .section {
    margin-bottom: 8px;
  }
  .section-header {
    padding-bottom: 6px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    margin-bottom: 10px;
  }
  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    opacity: 0.4;
  }
  .section-body {
    padding: 0;
  }
</style>
```

- [ ] **Step 2: Update popup dimensions in lib.rs**

Change constants (lines 23-24):

```rust
const POPUP_WIDTH: f64 = 350.0;
const POPUP_HEIGHT: f64 = 600.0;
```

- [ ] **Step 3: Wrap AI usage in +page.svelte with SystemSection**

Import `SystemSection` and wrap the existing Claude/Codex blocks:

```svelte
<script lang="ts">
  import SystemSection from '$lib/SystemSection.svelte';
  // ... existing imports
</script>

<!-- Replace the existing provider-section divs with: -->
<SystemSection title="AI Usage">
  {#if usage.claude}
    <div class="provider-block">
      <div class="provider-label">Claude Code</div>
      <!-- existing UsageBar components -->
    </div>
  {/if}
  {#if usage.codex}
    <div class="provider-block">
      <div class="provider-label">Codex CLI</div>
      <!-- existing UsageBar/credits components -->
    </div>
  {/if}
</SystemSection>
```

- [ ] **Step 4: Update main CSS in +page.svelte**

Change `main` width to 350px, ensure `overflow-y: auto` on body (already present at line 152).

```css
main {
  padding: 18px 20px 16px;
  width: 350px;
  box-sizing: border-box;
}
```

- [ ] **Step 5: Add settings gear icon to header**

```svelte
<header>
  <h1>VibeUsageBar</h1>
  <div class="header-right">
    <span class="dot" class:online={!error && usage} class:offline={error}></span>
    <button class="gear-btn" onclick={() => showSettings = !showSettings}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>
  </div>
</header>
```

Add `showSettings` state variable:

```typescript
let showSettings = $state(false);
```

Add CSS for gear button and header-right:

```css
.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.gear-btn {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}
.gear-btn:hover {
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.06);
}
```

- [ ] **Step 6: Verify frontend builds**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build`
Expected: Build succeeds

- [ ] **Step 7: Commit**

```bash
git add src/lib/SystemSection.svelte src/routes/+page.svelte src-tauri/src/lib.rs
git commit -m "feat: refactor popup with section layout, settings gear, 350x600 window"
```

---

## Phase 2: Rust System Metrics

### Task 6: Add sysinfo and battery crates

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add dependencies to Cargo.toml**

Add after the existing dependencies:

```toml
sysinfo = "0.32"
battery = "0.7"
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`
Expected: Downloads crates and compiles

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add sysinfo and battery crates"
```

---

### Task 7: System monitor module

**Files:**
- Create: `src-tauri/src/system_monitor.rs`

- [ ] **Step 1: Create system_monitor.rs with metric types**

```rust
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

/// Refreshes CPU, RAM, disk, and network metrics using sysinfo.
/// Call this on each system polling tick.
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

    // Disk (slow-changing, caller should only call every ~10 ticks)
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
        read_speed: 0, // Deferred: macOS disk I/O requires IOKit counters
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
/// The battery crate v0.7 uses the `uom` crate for units.
/// state_of_charge() returns a Ratio (0.0-1.0), multiply by 100 for percent.
pub fn read_battery() -> Option<BatteryMetrics> {
    let manager = battery::Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;
    let batt = batteries.next()?.ok()?;

    let state = batt.state();
    let charging = state == battery::State::Charging;

    // state_of_charge/health return Ratio values (0.0–1.0)
    // .value field gives the raw f32; multiply by 100 for percent
    let soc = batt.state_of_charge();
    let soh = batt.state_of_health();

    Some(BatteryMetrics {
        percent: soc.value * 100.0,
        health_percent: soh.value * 100.0,
        cycle_count: batt.cycle_count().unwrap_or(0),
        charging,
        time_to_full: if charging {
            batt.time_to_full().map(|t| (t.value / 60.0) as f64) // seconds to minutes
        } else {
            None
        },
        time_to_empty: if !charging {
            batt.time_to_empty().map(|t| (t.value / 60.0) as f64) // seconds to minutes
        } else {
            None
        },
    })
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`

Note: The `battery` crate uses `uom` for units. If compilation fails on unit types, adjust to use `batt.state_of_charge().value * 100.0` etc. Check the crate docs for exact API.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/system_monitor.rs
git commit -m "feat: add system_monitor module with sysinfo + battery metrics"
```

---

### Task 8: System polling loop in lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add module declaration and imports**

At top of `lib.rs`, add `mod system_monitor;` and update imports:

```rust
use std::sync::RwLock;
use system_monitor::SystemMetrics;
```

- [ ] **Step 2: Add system metrics state**

After `UsageState` struct:

```rust
struct SystemState(RwLock<SystemMetrics>);
```

- [ ] **Step 3: Add get_system_metrics command**

```rust
#[tauri::command]
fn get_system_metrics(state: tauri::State<'_, SystemState>) -> Result<SystemMetrics, String> {
    let m = state.0.read().map_err(|e| e.to_string())?;
    Ok(m.clone())
}
```

Register in invoke_handler: add `get_system_metrics` to the `generate_handler!` macro.

- [ ] **Step 4: Register SystemState and spawn system polling loop**

In `setup`, after existing polling loop spawn:

```rust
app.manage(SystemState(RwLock::new(SystemMetrics::default())));

// Spawn system metrics polling loop
let sys_handle = app.handle().clone();
let sys_settings = app.state::<SettingsState>().0.read().unwrap().clone();
let sys_interval = sys_settings.polling.system_interval_sec;
tauri::async_runtime::spawn(async move {
    let mut sys = sysinfo::System::new();
    let mut networks = sysinfo::Networks::new_with_refreshed_list();
    let mut net_tracker = system_monitor::NetworkTracker::new();
    // Initial CPU refresh needs two calls (first returns 0)
    sys.refresh_cpu_usage();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let mut tick_count: u64 = 0;
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(sys_interval),
    );

    loop {
        interval.tick().await;
        tick_count += 1;

        let (cpu, ram, disk, net) = system_monitor::refresh_sysinfo_metrics(
            &mut sys, &mut networks, &mut net_tracker, sys_interval,
        );

        // Battery: refresh every 30 seconds (slow-changing)
        let battery = if tick_count % (30 / sys_interval).max(1) == 0 || tick_count == 1 {
            system_monitor::read_battery()
        } else {
            if let Some(state) = sys_handle.try_state::<SystemState>() {
                state.0.read().ok().and_then(|m| m.battery.clone())
            } else {
                None
            }
        };

        let metrics = SystemMetrics {
            cpu,
            ram,
            disk,
            network: net,
            battery,
            temps: vec![],     // Phase 3: Swift dylib
            fans: vec![],      // Phase 3: Swift dylib
            bluetooth: vec![], // Phase 3: Swift dylib
        };

        if let Some(state) = sys_handle.try_state::<SystemState>() {
            if let Ok(mut m) = state.0.write() {
                *m = metrics.clone();
            }
        }
        let _ = sys_handle.emit("system-update", &metrics);
    }
});
```

- [ ] **Step 5: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`
Expected: Compiles

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add system metrics polling loop with sysinfo + battery"
```

---

### Task 9: Frontend system metrics display

**Files:**
- Create: `src/lib/NetworkSpeed.svelte`
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Create NetworkSpeed.svelte**

```svelte
<script lang="ts">
  let { download, upload }: {
    download: number;
    upload: number;
  } = $props();

  function formatSpeed(bytes: number): string {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB/s`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB/s`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB/s`;
    return `${bytes} B/s`;
  }
</script>

<div class="network-speed">
  <span class="down">↓ {formatSpeed(download)}</span>
  <span class="up">↑ {formatSpeed(upload)}</span>
</div>

<style>
  .network-speed {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    padding: 2px 0;
  }
  .down {
    color: #34d399;
  }
  .up {
    color: #60a5fa;
  }
</style>
```

- [ ] **Step 2: Add system metrics sections to +page.svelte**

Add imports and state:

```typescript
import type { SystemMetrics } from '$lib/types';
import NetworkSpeed from '$lib/NetworkSpeed.svelte';

let systemMetrics: SystemMetrics | null = $state(null);
```

Add listener in `onMount`:

```typescript
let unlistenSystem: (() => void) | undefined;

listen<SystemMetrics>('system-update', (event) => {
  systemMetrics = event.payload;
}).then((fn) => { unlistenSystem = fn; });

// In cleanup:
return () => {
  unlistenUpdate?.();
  unlistenError?.();
  unlistenSystem?.();
};
```

Add sections after the AI Usage section and before WeeklyChart:

```svelte
{#if systemMetrics}
  <SystemSection title="Compute">
    <UsageBar label="CPU" utilization={systemMetrics.cpu.overall} resetsAt="" />
    <UsageBar
      label="RAM"
      utilization={(systemMetrics.ram.used_gb / systemMetrics.ram.total_gb) * 100}
      resetsAt=""
    />
    <div class="ram-detail">
      {systemMetrics.ram.used_gb.toFixed(1)} / {systemMetrics.ram.total_gb.toFixed(0)} GB
    </div>
  </SystemSection>

  <SystemSection title="Storage & Network">
    <UsageBar
      label="Disk"
      utilization={(systemMetrics.disk.used_gb / systemMetrics.disk.total_gb) * 100}
      resetsAt=""
    />
    <div class="disk-detail">
      {systemMetrics.disk.used_gb.toFixed(0)} / {systemMetrics.disk.total_gb.toFixed(0)} GB
    </div>
    <NetworkSpeed
      download={systemMetrics.network.download_speed}
      upload={systemMetrics.network.upload_speed}
    />
  </SystemSection>

  {#if systemMetrics.battery}
    <SystemSection title="Hardware">
      <div class="battery-info">
        <span class="battery-icon">{systemMetrics.battery.charging ? '⚡' : '🔋'}</span>
        <span class="battery-percent">{systemMetrics.battery.percent.toFixed(0)}%</span>
        <span class="battery-detail">
          Health {systemMetrics.battery.health_percent.toFixed(0)}%
          · {systemMetrics.battery.cycle_count} cycles
        </span>
      </div>
    </SystemSection>
  {/if}
{/if}
```

- [ ] **Step 3: Update UsageBar to handle optional resetsAt**

In `UsageBar.svelte`, make `resetsAt` optional and skip timer when empty:

Update the props:

```typescript
let { label, utilization, resetsAt = '' }: {
  label: string;
  utilization: number;
  resetsAt?: string;
} = $props();
```

Wrap the timer `$effect` to only run when resetsAt is set:

```typescript
$effect(() => {
  if (!resetsAt) return;
  const timer = setInterval(() => { now = Date.now(); }, 60_000);
  return () => clearInterval(timer);
});
```

And conditionally render the meta section:

```svelte
{#if resetsAt}
  <div class="meta">
    <span class="reset">Resets in {countdown}</span>
  </div>
{/if}
```

- [ ] **Step 4: Add CSS for new elements in +page.svelte**

```css
.ram-detail, .disk-detail {
  font-size: 11px;
  opacity: 0.4;
  text-align: right;
  margin-top: -8px;
  margin-bottom: 8px;
}

.battery-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
}
.battery-icon {
  font-size: 16px;
}
.battery-percent {
  font-weight: 700;
  font-size: 18px;
  font-variant-numeric: tabular-nums;
  color: #34d399;
}
.battery-detail {
  font-size: 11px;
  opacity: 0.4;
}
```

- [ ] **Step 5: Verify frontend builds**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build`
Expected: Build succeeds

- [ ] **Step 6: Commit**

```bash
git add src/lib/NetworkSpeed.svelte src/lib/UsageBar.svelte src/routes/+page.svelte
git commit -m "feat: display system metrics (CPU, RAM, disk, network, battery) in popup"
```

---

## Phase 3: Swift Dylib

### Task 10: Swift dylib source

**Files:**
- Create: `src-tauri/swift/SystemMonitor.swift`

- [ ] **Step 1: Create swift directory**

```bash
mkdir -p /Users/vladislavkonovalov/aiUsagebar/src-tauri/swift
```

- [ ] **Step 2: Create SystemMonitor.swift**

```swift
import Foundation
import IOKit

// MARK: - SMC Interface

private let SMC_KEY_CPU_TEMP: UInt32 = fourCharCode("TC0P")
private let SMC_KEY_GPU_TEMP: UInt32 = fourCharCode("TG0P")

private func fourCharCode(_ str: String) -> UInt32 {
    var result: UInt32 = 0
    for char in str.utf8 {
        result = (result << 8) | UInt32(char)
    }
    return result
}

// SMC connection
private var smcConnection: io_connect_t = 0

private struct SMCKeyData {
    var key: UInt32 = 0
    var vers: IOByteCount = 0
    var pLimitData: IOByteCount = 0
    var keyInfo: SMCKeyInfoData = SMCKeyInfoData()
    var result: UInt8 = 0
    var status: UInt8 = 0
    var data8: UInt8 = 0
    var data32: UInt32 = 0
    var bytes: (UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
                UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
                UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
                UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8) = (0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0)
}

private struct SMCKeyInfoData {
    var dataSize: IOByteCount = 0
    var dataType: UInt32 = 0
    var dataAttributes: UInt8 = 0
}

private func openSMC() -> Bool {
    let service = IOServiceGetMatchingService(kIOMainPortDefault,
        IOServiceMatching("AppleSMC"))
    guard service != 0 else { return false }
    let result = IOServiceOpen(service, mach_task_self_, 0, &smcConnection)
    IOObjectRelease(service)
    return result == kIOReturnSuccess
}

private func readSMCTemp(key: String) -> Double? {
    // Step 1: kSMCGetKeyInfo (data8 = 9) — get key metadata
    var inputStruct = SMCKeyData()
    var outputStruct = SMCKeyData()
    inputStruct.key = fourCharCode(key)
    inputStruct.data8 = 9 // kSMCGetKeyInfo

    var inputSize = MemoryLayout<SMCKeyData>.size
    var outputSize = MemoryLayout<SMCKeyData>.size

    let result = IOConnectCallStructMethod(
        smcConnection, 2,
        &inputStruct, inputSize,
        &outputStruct, &outputSize
    )

    guard result == kIOReturnSuccess else { return nil }

    // Step 2: kSMCReadKey (data8 = 5) — read actual value using retrieved keyInfo
    inputStruct.data8 = 5 // kSMCReadKey
    inputStruct.keyInfo.dataSize = outputStruct.keyInfo.dataSize
    inputStruct.keyInfo.dataType = outputStruct.keyInfo.dataType

    let result2 = IOConnectCallStructMethod(
        smcConnection, 2,
        &inputStruct, inputSize,
        &outputStruct, &outputSize
    )

    guard result2 == kIOReturnSuccess else { return nil }

    // sp78 type: signed 7.8 fixed point
    let intValue = Int(outputStruct.bytes.0) << 8 | Int(outputStruct.bytes.1)
    return Double(intValue) / 256.0
}

private func readFanSpeed(index: Int) -> (rpm: Int, min: Int, max: Int)? {
    let actualKey = String(format: "F%dAc", index)
    let minKey = String(format: "F%dMn", index)
    let maxKey = String(format: "F%dMx", index)

    // Simplified: read fpe2 type values
    // In production, would need proper SMC fpe2 decoding
    guard let actual = readSMCFpe2(key: actualKey),
          let min = readSMCFpe2(key: minKey),
          let max = readSMCFpe2(key: maxKey) else { return nil }

    return (rpm: Int(actual), min: Int(min), max: Int(max))
}

private func readSMCFpe2(key: String) -> Double? {
    // fpe2: unsigned 14.2 fixed point (used for fan speeds)
    var inputStruct = SMCKeyData()
    var outputStruct = SMCKeyData()
    inputStruct.key = fourCharCode(key)
    inputStruct.data8 = 9 // kSMCGetKeyInfo — get key metadata first

    var inputSize = MemoryLayout<SMCKeyData>.size
    var outputSize = MemoryLayout<SMCKeyData>.size

    let result = IOConnectCallStructMethod(
        smcConnection, 2,
        &inputStruct, inputSize,
        &outputStruct, &outputSize
    )
    guard result == kIOReturnSuccess else { return nil }

    // kSMCReadKey — read value with retrieved keyInfo
    inputStruct.data8 = 5
    inputStruct.keyInfo.dataSize = outputStruct.keyInfo.dataSize
    inputStruct.keyInfo.dataType = outputStruct.keyInfo.dataType

    let result2 = IOConnectCallStructMethod(
        smcConnection, 2,
        &inputStruct, inputSize,
        &outputStruct, &outputSize
    )
    guard result2 == kIOReturnSuccess else { return nil }

    let intValue = UInt(outputStruct.bytes.0) << 8 | UInt(outputStruct.bytes.1)
    return Double(intValue) / 4.0
}

// MARK: - Exported C ABI Functions

@_cdecl("smc_read_all")
public func smcReadAll() -> UnsafeMutablePointer<CChar> {
    var temps: [[String: Any]] = []
    var fans: [[String: Any]] = []

    if smcConnection == 0 {
        _ = openSMC()
    }

    if smcConnection != 0 {
        // Common temperature keys for Apple Silicon and Intel
        let tempKeys = [
            ("TC0P", "CPU Proximity"),
            ("TC0D", "CPU Die"),
            ("TC0E", "CPU Efficiency"),
            ("TG0P", "GPU Proximity"),
            ("TG0D", "GPU Die"),
            ("Tp09", "SSD"),
        ]

        for (key, name) in tempKeys {
            if let value = readSMCTemp(key: key), value > 0 && value < 120 {
                temps.append(["name": name, "value": round(value * 10) / 10])
            }
        }

        // Fans (up to 4)
        for i in 0..<4 {
            if let fan = readFanSpeed(index: i) {
                fans.append([
                    "name": "Fan \(i)",
                    "rpm": fan.rpm,
                    "min": fan.min,
                    "max": fan.max
                ])
            }
        }
    }

    let result: [String: Any] = ["temps": temps, "fans": fans]
    let jsonData = (try? JSONSerialization.data(withJSONObject: result)) ?? "{}".data(using: .utf8)!
    let jsonString = String(data: jsonData, encoding: .utf8) ?? "{}"

    return strdup(jsonString)!
}

@_cdecl("bt_get_devices")
public func btGetDevices() -> UnsafeMutablePointer<CChar> {
    // IOBluetooth requires the framework to be linked
    // For now, use IOKit to enumerate paired devices
    var devices: [[String: Any]] = []

    // Use system_profiler as a reliable fallback
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/system_profiler")
    process.arguments = ["SPBluetoothDataType", "-json"]
    let pipe = Pipe()
    process.standardOutput = pipe

    do {
        try process.run()
        process.waitUntilExit()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()

        if let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
           let btData = json["SPBluetoothDataType"] as? [[String: Any]] {

            for section in btData {
                // Connected devices
                if let connected = section["device_connected"] as? [[String: Any]] {
                    for device in connected {
                        for (name, info) in device {
                            guard let infoDict = info as? [String: Any] else { continue }
                            let batteryLevel = infoDict["device_batteryLevelMain"] as? String
                            let battery = batteryLevel.flatMap { Int($0.replacingOccurrences(of: "%", with: "")) }
                            devices.append([
                                "name": name,
                                "connected": true,
                                "battery": battery as Any
                            ])
                        }
                    }
                }
            }
        }
    } catch {
        // Return empty array on error
    }

    let jsonData = (try? JSONSerialization.data(withJSONObject: devices)) ?? "[]".data(using: .utf8)!
    let jsonString = String(data: jsonData, encoding: .utf8) ?? "[]"

    return strdup(jsonString)!
}

@_cdecl("free_string")
public func freeString(_ ptr: UnsafeMutablePointer<CChar>) {
    free(ptr)
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/swift/SystemMonitor.swift
git commit -m "feat: add Swift dylib for SMC temps/fans and Bluetooth devices"
```

---

### Task 11: Swift dylib build integration

**Files:**
- Modify: `src-tauri/build.rs`

- [ ] **Step 1: Update build.rs to compile Swift dylib**

```rust
use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    // Compile Swift dylib
    let swift_src = Path::new("swift/SystemMonitor.swift");
    if swift_src.exists() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let dylib_path = format!("{}/libsystem_monitor.dylib", out_dir);

        let status = Command::new("swiftc")
            .args([
                "-emit-library",
                "-o", &dylib_path,
                "-module-name", "SystemMonitor",
                "swift/SystemMonitor.swift",
                "-framework", "IOKit",
                "-framework", "Foundation",
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:rustc-link-lib=dylib=system_monitor");
                // Enable cfg flag so swift_bridge.rs compiles the FFI block
                println!("cargo:rustc-cfg=has_swift_dylib");
            }
            Ok(_) => {
                eprintln!("Warning: Swift dylib compilation failed. SMC/Bluetooth features will be unavailable.");
            }
            Err(e) => {
                eprintln!("Warning: swiftc not found ({e}). SMC/Bluetooth features will be unavailable.");
            }
        }
    }

    println!("cargo:rerun-if-changed=swift/SystemMonitor.swift");

    tauri_build::build()
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`

If `swiftc` is not available, the build should still succeed (with warning).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/build.rs
git commit -m "feat: build.rs compiles Swift dylib during cargo build"
```

---

### Task 12: Rust FFI bridge

**Files:**
- Create: `src-tauri/src/swift_bridge.rs`
- Modify: `src-tauri/src/system_monitor.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create swift_bridge.rs**

```rust
use crate::system_monitor::{TempSensor, FanInfo, BtDevice};

// Conditionally compiled — only when build.rs successfully compiled the Swift dylib
#[cfg(has_swift_dylib)]
mod ffi {
    use std::ffi::{c_char, CStr};
    use crate::system_monitor::{TempSensor, FanInfo, BtDevice};
    use serde::Deserialize;

    #[link(name = "system_monitor")]
    extern "C" {
        fn smc_read_all() -> *mut c_char;
        fn bt_get_devices() -> *mut c_char;
        fn free_string(ptr: *mut c_char);
    }

    #[derive(Deserialize, Default)]
    struct SmcData {
        temps: Vec<TempSensor>,
        fans: Vec<FanInfo>,
    }

    pub fn get_smc_data() -> (Vec<TempSensor>, Vec<FanInfo>) {
        unsafe {
            let ptr = smc_read_all();
            if ptr.is_null() {
                return (vec![], vec![]);
            }
            let json_owned = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            free_string(ptr);
            let data: SmcData = serde_json::from_str(&json_owned).unwrap_or_default();
            (data.temps, data.fans)
        }
    }

    pub fn get_bluetooth_devices() -> Vec<BtDevice> {
        unsafe {
            let ptr = bt_get_devices();
            if ptr.is_null() {
                return vec![];
            }
            let json_owned = CStr::from_ptr(ptr).to_string_lossy().into_owned();
            free_string(ptr);
            serde_json::from_str(&json_owned).unwrap_or_default()
        }
    }
}

/// Public API — falls back to empty data when Swift dylib is not available.
pub fn get_smc_data() -> (Vec<TempSensor>, Vec<FanInfo>) {
    #[cfg(has_swift_dylib)]
    { ffi::get_smc_data() }
    #[cfg(not(has_swift_dylib))]
    { (vec![], vec![]) }
}

pub fn get_bluetooth_devices() -> Vec<BtDevice> {
    #[cfg(has_swift_dylib)]
    { ffi::get_bluetooth_devices() }
    #[cfg(not(has_swift_dylib))]
    { vec![] }
}
```

- [ ] **Step 2: Add module declaration in lib.rs**

Add `mod swift_bridge;` after `mod system_monitor;`.

- [ ] **Step 3: Integrate swift_bridge into system polling loop**

In the system polling loop in `lib.rs`, replace the placeholder `temps`, `fans`, `bluetooth`:

```rust
// SMC: temps + fans every tick
let (temps, fans) = swift_bridge::get_smc_data();

// Bluetooth: every 10 seconds (slow-changing)
let bluetooth = if tick_count % (10 / sys_interval).max(1) == 0 || tick_count == 1 {
    swift_bridge::get_bluetooth_devices()
} else {
    if let Some(state) = sys_handle.try_state::<SystemState>() {
        state.0.read().ok().map(|m| m.bluetooth.clone()).unwrap_or_default()
    } else {
        vec![]
    }
};

let metrics = SystemMetrics {
    cpu,
    ram,
    disk,
    network: net,
    battery,
    temps,
    fans,
    bluetooth,
};
```

- [ ] **Step 4: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo build`
Expected: Compiles and links Swift dylib

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/swift_bridge.rs src-tauri/src/lib.rs
git commit -m "feat: FFI bridge to Swift dylib for SMC and Bluetooth"
```

---

### Task 13: Frontend temp and bluetooth components

**Files:**
- Create: `src/lib/TempGauge.svelte`
- Create: `src/lib/BluetoothList.svelte`
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Create TempGauge.svelte**

```svelte
<script lang="ts">
  import type { TempSensor, FanInfo } from '$lib/types';

  let { temps, fans }: {
    temps: TempSensor[];
    fans: FanInfo[];
  } = $props();

  function tempColor(value: number): string {
    if (value >= 80) return '#ef4444';
    if (value >= 50) return '#f59e0b';
    return '#34d399';
  }
</script>

<div class="hardware-metrics">
  {#if temps.length > 0}
    <div class="temps-row">
      {#each temps as sensor}
        <div class="temp-item">
          <span class="temp-label">{sensor.name}</span>
          <span class="temp-value" style="color: {tempColor(sensor.value)}">
            {sensor.value.toFixed(0)}°C
          </span>
        </div>
      {/each}
    </div>
  {/if}

  {#if fans.length > 0}
    <div class="fans-row">
      {#each fans as fan}
        <span class="fan-item">{fan.name}: {fan.rpm} rpm</span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .hardware-metrics {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .temps-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }
  .temp-item {
    display: flex;
    gap: 6px;
    align-items: baseline;
  }
  .temp-label {
    font-size: 12px;
    opacity: 0.5;
  }
  .temp-value {
    font-size: 15px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .fans-row {
    display: flex;
    gap: 12px;
    font-size: 12px;
    opacity: 0.5;
  }
</style>
```

- [ ] **Step 2: Create BluetoothList.svelte**

```svelte
<script lang="ts">
  import type { BtDevice } from '$lib/types';

  let { devices }: { devices: BtDevice[] } = $props();
</script>

{#if devices.length > 0}
  <div class="bt-list">
    {#each devices as device}
      <div class="bt-item">
        <span class="bt-dot" class:connected={device.connected}></span>
        <span class="bt-name">{device.name}</span>
        <span class="bt-battery">
          {device.battery != null ? `${device.battery}%` : '--'}
        </span>
      </div>
    {/each}
  </div>
{:else}
  <div class="bt-empty">No devices</div>
{/if}

<style>
  .bt-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .bt-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  .bt-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #555;
    flex-shrink: 0;
  }
  .bt-dot.connected {
    background: #34d399;
  }
  .bt-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .bt-battery {
    font-variant-numeric: tabular-nums;
    opacity: 0.5;
    font-size: 12px;
  }
  .bt-empty {
    font-size: 12px;
    opacity: 0.3;
  }
</style>
```

- [ ] **Step 3: Add TempGauge and BluetoothList to +page.svelte**

Import components:

```typescript
import TempGauge from '$lib/TempGauge.svelte';
import BluetoothList from '$lib/BluetoothList.svelte';
```

Update the Hardware section to include temps/fans:

```svelte
<SystemSection title="Hardware">
  <TempGauge temps={systemMetrics.temps} fans={systemMetrics.fans} />
  {#if systemMetrics.battery}
    <div class="battery-info">
      <!-- existing battery display -->
    </div>
  {/if}
</SystemSection>

{#if systemMetrics.bluetooth.length > 0}
  <SystemSection title="Bluetooth">
    <BluetoothList devices={systemMetrics.bluetooth} />
  </SystemSection>
{/if}
```

- [ ] **Step 4: Verify frontend builds**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build`
Expected: Build succeeds

- [ ] **Step 5: Commit**

```bash
git add src/lib/TempGauge.svelte src/lib/BluetoothList.svelte src/routes/+page.svelte
git commit -m "feat: add temperature, fan, and bluetooth UI components"
```

---

## Phase 4: Tray Icon

### Task 14: Configurable tray text

**Files:**
- Create: `src-tauri/src/tray_text.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create tray_text.rs**

```rust
use crate::settings::TraySettings;
use crate::system_monitor::SystemMetrics;
use crate::api::AllUsage;

/// Format tray title string based on user settings and current metrics.
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
```

- [ ] **Step 2: Add module and integrate into polling loops**

Add `mod tray_text;` to `lib.rs`.

Add tray title update at end of system polling loop:

```rust
// Update tray title
if let Some(settings_state) = sys_handle.try_state::<SettingsState>() {
    if let Ok(settings) = settings_state.0.read() {
        let ai = sys_handle.try_state::<UsageState>()
            .and_then(|s| s.0.lock().ok().map(|d| d.clone()))
            .flatten();
        let title = tray_text::format_tray_title(&settings.tray, &metrics, &ai);
        if let Some(tray) = sys_handle.tray_by_id(&tauri::tray::TrayIconId::new(TRAY_ID)) {
            let _ = tray.set_title(Some(&title));
        }
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo check`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray_text.rs src-tauri/src/lib.rs
git commit -m "feat: configurable text-based tray title from settings"
```

---

## Phase 5: Settings UI

### Task 15: Settings page component

**Files:**
- Create: `src/lib/SettingsPage.svelte`
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Create SettingsPage.svelte**

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { UserSettings } from '$lib/types';

  let { settings, onClose, onSave }: {
    settings: UserSettings;
    onClose: () => void;
    onSave: (s: UserSettings) => void;
  } = $props();

  // Local copy for editing
  let local = $state(structuredClone(settings));

  const availableItems = [
    { key: 'cpu', label: 'CPU %' },
    { key: 'ram', label: 'RAM' },
    { key: 'temp_cpu', label: 'CPU Temp' },
    { key: 'temp_gpu', label: 'GPU Temp' },
    { key: 'fan', label: 'Fan RPM' },
    { key: 'battery', label: 'Battery' },
    { key: 'net_down', label: 'Download' },
    { key: 'net_up', label: 'Upload' },
    { key: 'disk_free', label: 'Disk Free' },
    { key: 'ai_session', label: 'AI Session' },
    { key: 'ai_weekly', label: 'AI Weekly' },
  ];

  function toggleItem(key: string) {
    const idx = local.tray.items.indexOf(key);
    if (idx >= 0) {
      local.tray.items = local.tray.items.filter((k: string) => k !== key);
    } else {
      local.tray.items = [...local.tray.items, key];
    }
  }

  async function save() {
    try {
      await invoke('save_settings_cmd', { newSettings: local });
      onSave(local);
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }
</script>

<div class="settings-page">
  <div class="settings-header">
    <button class="back-btn" onclick={onClose}>← Back</button>
    <h2>Settings</h2>
  </div>

  <div class="settings-section">
    <h3>Tray Display</h3>
    <div class="tray-items">
      {#each availableItems as item}
        <label class="tray-item">
          <input
            type="checkbox"
            checked={local.tray.items.includes(item.key)}
            onchange={() => toggleItem(item.key)}
          />
          <span>{item.label}</span>
        </label>
      {/each}
    </div>

    <div class="tray-preview">
      <span class="preview-label">Preview:</span>
      <span class="preview-text">
        {local.tray.items.join(local.tray.separator)}
      </span>
    </div>
  </div>

  <div class="settings-section">
    <h3>Polling Interval</h3>
    <label class="slider-label">
      System metrics: {local.polling.system_interval_sec}s
      <input
        type="range"
        min="1"
        max="30"
        bind:value={local.polling.system_interval_sec}
      />
    </label>
  </div>

  <button class="save-btn" onclick={save}>Save</button>
</div>

<style>
  .settings-page {
    padding: 4px 0;
  }
  .settings-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
  }
  .settings-header h2 {
    font-size: 15px;
    margin: 0;
    font-weight: 600;
  }
  .back-btn {
    background: none;
    border: none;
    color: #60a5fa;
    cursor: pointer;
    font-size: 13px;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .back-btn:hover {
    background: rgba(96, 165, 250, 0.1);
  }
  .settings-section {
    margin-bottom: 16px;
  }
  .settings-section h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    opacity: 0.5;
    margin: 0 0 8px;
  }
  .tray-items {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .tray-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .tray-item input {
    accent-color: #34d399;
  }
  .tray-preview {
    margin-top: 10px;
    padding: 8px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 6px;
    font-size: 12px;
  }
  .preview-label {
    opacity: 0.4;
    margin-right: 6px;
  }
  .preview-text {
    font-family: 'SF Mono', monospace;
    color: #34d399;
  }
  .slider-label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
  }
  .slider-label input[type="range"] {
    width: 100%;
    accent-color: #34d399;
  }
  .save-btn {
    width: 100%;
    padding: 8px;
    background: #34d399;
    color: #0f0f1a;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
  }
  .save-btn:hover {
    background: #2ec48a;
  }
</style>
```

- [ ] **Step 2: Integrate SettingsPage into +page.svelte**

Add import and settings state:

```typescript
import SettingsPage from '$lib/SettingsPage.svelte';

let currentSettings: UserSettings | null = $state(null);
```

Load settings in `onMount`:

```typescript
invoke<UserSettings>('get_settings').then((s) => {
  currentSettings = s;
}).catch(() => {});
```

Add conditional rendering — when `showSettings` is true, show settings instead of metrics:

```svelte
{#if showSettings && currentSettings}
  <SettingsPage
    settings={currentSettings}
    onClose={() => showSettings = false}
    onSave={(s) => { currentSettings = s; showSettings = false; }}
  />
{:else}
  <!-- existing metrics content -->
{/if}
```

- [ ] **Step 3: Verify frontend builds**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add src/lib/SettingsPage.svelte src/routes/+page.svelte
git commit -m "feat: add settings page with tray config and polling interval"
```

---

### Task 16: Final integration test

- [ ] **Step 1: Full build**

Run: `cd /Users/vladislavkonovalov/aiUsagebar/src-tauri && cargo build`
Expected: Rust backend compiles with Swift dylib

- [ ] **Step 2: Frontend build**

Run: `cd /Users/vladislavkonovalov/aiUsagebar && npm run build`
Expected: SvelteKit builds static output

- [ ] **Step 3: Manual smoke test**

Run from native terminal (not Claude Code sandbox): `cd /Users/vladislavkonovalov/aiUsagebar && npx tauri dev`

Verify:
1. Tray icon appears with text title
2. Popup shows AI usage sections (Claude + Codex)
3. Popup shows system sections (CPU, RAM, Disk, Network)
4. Hardware section shows temperatures and fans (or N/A)
5. Battery section shows charge/health/cycles
6. Bluetooth section shows devices (or empty)
7. Settings gear opens settings page
8. Changing tray items and saving updates the tray text
9. WeeklyChart still works

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: VibeUsageBar v0.2.0 — unified AI + system monitor menubar app"
```
