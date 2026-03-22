# VibeUsageBar — Session Handoff (2026-03-22)

## What was done this session

Transformed aiUsageBar (AI-only usage tracker) into **VibeUsageBar v0.2.0** — a unified macOS menubar app with AI usage tracking + system monitoring + themes.

### Completed
- Renamed project aiUsageBar → VibeUsageBar
- Settings system (JSON + Rust state + Svelte settings page)
- Data migration from old com.aiusagebar.app
- System metrics: CPU, RAM, disk, network via `sysinfo` crate
- Battery via `battery` crate (with `system_profiler` fallback for health on Apple Silicon)
- Swift dylib for SMC temps/fans + Bluetooth (compiles, but SMC blocked on M4 by macOS permissions)
- Rust FFI bridge with `cfg(has_swift_dylib)` conditional compilation
- Configurable tray text (`tray.set_title()`)
- Popup closes on focus loss (standard menubar behavior)
- 6 themes: glass (default), glass-blue, hacker, midnight, cyberpunk, frost
- Theme live preview in settings
- Extra usage: fixed cents→dollars, shows real % (142% not capped 100%)
- Extra usage notifications at 80% and 100%
- WeeklyChart moved after AI Usage section (not at bottom)

### Known issues
- **SMC temperatures**: Apple Silicon M4 blocks IOKit SMC access without entitlements. Tried `AppleSMCKeysEndpoint`, `IOHIDEventSystemClient` — all return 0. Only `sudo powermetrics` works (needs passwordless sudo). Temperature section currently empty.
- **Bluetooth**: `system_profiler SPBluetoothDataType` works but shows only connected devices. If nothing connected → empty.
- **Fans**: MacBook Air M4 is fanless — `0 fans` is correct behavior.
- **Extra usage credits**: API returns values in cents (2848 = $28.48). Fixed in ExtraUsage.svelte.

## Next session priorities

### 1. SparkChart component (user requested)
Replace UsageBar progress bars for system metrics (CPU, RAM, network) with real-time area sparkline charts — small SVG or canvas charts that draw continuously, similar to Activity Monitor / Stats app. Need:
- `src/lib/SparkChart.svelte` — new component
- Ring buffer of last N values (e.g. 60 samples)
- SVG `<path>` with area fill
- Color from CSS variables
- Replace UsageBar in Compute and Storage & Network sections

### 2. Polish
- Glass theme: chart bars don't contrast well (white on translucent)
- Settings: drag & drop for tray item reorder (Phase 5 from spec)
- Collapsible sections in popup

### 3. Deferred from spec
- GPU utilization (IOReport framework, undocumented Apple API — Phase 5)
- System metrics SQLite history (currently real-time only)
- Settings polling interval runtime update (currently requires restart)

## Key files

| File | Purpose |
|---|---|
| `src-tauri/src/lib.rs` | Main app: polling loops, tray, window, notifications |
| `src-tauri/src/system_monitor.rs` | sysinfo + battery metrics |
| `src-tauri/src/swift_bridge.rs` | FFI to Swift dylib (conditional) |
| `src-tauri/src/settings.rs` | Settings load/save/defaults |
| `src-tauri/src/tray_text.rs` | Format tray title string |
| `src-tauri/swift/SystemMonitor.swift` | SMC + Bluetooth via IOKit |
| `src/lib/themes.ts` | 6 theme definitions + applyTheme() |
| `src/routes/+page.svelte` | Main UI with all sections |
| `src/lib/SettingsPage.svelte` | Settings with theme picker |

## Spec & Plan docs
- `docs/superpowers/specs/2026-03-22-vibeusagebar-system-monitor-design.md`
- `docs/superpowers/plans/2026-03-22-vibeusagebar-system-monitor.md`
