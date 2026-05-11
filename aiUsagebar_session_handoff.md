# VibeUsageBar — Session Handoff (2026-05-11)

## macOS 26 (Tahoe) tray + notifications fix

### Symptom
On macOS 26.3.1 (MacBook Air M4) the menubar icon never appeared, even though
`TrayIconBuilder::build()` returned `Ok` and the app process was alive. Push
notifications worked but clicking them did nothing. Affects Tauri 2.10.x and
2.11.x equally; not a regression in our code.

### Root cause
1. Tauri's `tray-icon`/`muda` crate creates the `NSStatusItem` synchronously
   inside `setup()` — i.e. *before* the NSApp event loop is pumping. On macOS
   26 ControlCenter's new `appStatusItems` registry never picks those items
   up (no `Host properties initialized` log line for our bundle id).
2. `tauri-plugin-notification` on desktop has no public click handler API;
   the "Click to open" body text was misleading.
3. macOS 26 also gates *all* newly-registered third-party status items behind
   a per-app visibility toggle in System Settings → Control Center → Menu Bar
   ("blocked host" state on first registration).

### Fix — bypass Tauri's tray/notification path on macOS via Swift dylib

Implemented a native AppKit/UserNotifications layer inside the existing
`libsystem_monitor.dylib` (already linked for SMC). All menu-bar and
notification calls now go through this dylib via FFI; the Tauri tray-icon
crate is still in the dependency tree but no longer instantiated.

- `swift/SystemMonitor.swift`
  - `+import AppKit, UserNotifications`
  - `TrayDelegate` (NSObject) — click handler dispatches left-click → Rust
    callback (toggle popup), right-click / Ctrl-click → context menu (Quit)
  - `tray_init(callback)` — async-dispatches to main thread so it runs AFTER
    NSApp event loop starts (the critical change vs. Tauri's approach)
  - `tray_set_title(cstr)`, `tray_set_icon_rgba(bytes, w, h)` — main-thread
    async updates to button title/image
  - `NotificationDelegate` (UNUserNotificationCenterDelegate) — taps invoke
    the same Rust callback (toggle popup), so notification clicks work
  - `notification_show(title, body)` — replaces all `app.notification()...`
    sites
- `build.rs` — `+-framework AppKit`
- `src/swift_bridge.rs` — `tray` and `notification` modules with cfg-gated
  FFI declarations
- `src/lib.rs`
  - Removed `TrayIconBuilder`, removed `NotificationExt` import
  - `static APP_HANDLE: OnceLock<AppHandle>` + `extern "C" fn on_tray_click`
    callback registered with Swift
  - All 5 notification call sites use `swift_bridge::notification::show(...)`
  - `update_tray_icon` / `set_tray_error_icon` / system metrics loop's
    title update all route through `swift_bridge::tray::*`
- `src/tray_icon.rs` — empty-bar background colour bumped from dark navy
  `0x3a3a4a` to mid-grey `0x90909a` so the 0%-utilization initial icon is
  legible on both light and dark menubars
- Tauri stack bumped: 2.10.3 → 2.11.1, `tray-icon` 0.21.3 → 0.23.1
  (`Cargo.toml`, `Cargo.lock`, `package.json`, `package-lock.json`).
  This did not fix the issue on its own but is required for future Tauri
  compatibility and brings in unrelated wry/runtime fixes.

### One-time user action required after install
macOS 26 marks new third-party status items as "blocked host" on first
registration. The user must approve VibeUsageBar once via:

  **System Settings → Control Center → Menu Bar → enable VibeUsageBar**

(or drag it out from the hidden tray in the menubar customization view).
After that the icon shows on every subsequent launch automatically.

### Launch quirk
`open -a VibeUsageBar` (LaunchServices flow) is required for the
`NSStatusItem` to actually attach to ControlCenter. Running the binary
directly (`/Applications/VibeUsageBar.app/Contents/MacOS/vibeusagebar`)
skips LaunchServices init and the icon never registers — useful to know
when debugging from a terminal.

### Files touched
- `Cargo.toml`, `Cargo.lock`
- `package.json`, `package-lock.json`
- `src-tauri/build.rs`
- `src-tauri/swift/SystemMonitor.swift`
- `src-tauri/src/swift_bridge.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/tray_icon.rs`
- `src-tauri/capabilities/default.json` (global-shortcut permission, from
  the earlier ⇧⌥D feature; left unchanged this session)

### Verified
- Menubar icon: two coloured bars + `CPU XX%` text — visible
- Left-click on icon → popup opens
- Right-click on icon → context menu with Quit
- ⇧⌥D global shortcut → popup opens
- Native push notifications fire; clicking a banner opens popup

---

# VibeUsageBar — Session Handoff (2026-03-30)

## Current version: v0.3.0

Unified macOS menubar utility: AI usage tracking (Claude Code + Codex CLI) + full system monitoring + history.

## What was done (v0.2.0 → v0.3.0)

### SparkChart
- `src/lib/SparkChart.svelte` — SVG area sparkline with monotone cubic splines
- Ring buffer of 60 samples for CPU, RAM, network
- Gradient area fill, threshold colors (green → yellow 80% → red 95%)
- `formattedValue` prop for custom display (network speeds)
- Sanitized SVG gradient IDs (fixed black charts on special chars)

### Per-core CPU
- `src/lib/CpuCores.svelte` — mini sparkline grid for all cores (1-10 on M4)
- Keyed `{#each}` to prevent flickering on updates
- Toggleable via Settings → "Show per-core CPU"

### Chart mode toggle
- Settings UI: Sparkline vs Progress Bar buttons
- Affects CPU, RAM, Network sections
- Stored in `popup.chart_mode` ("spark" | "bar")

### Section management
- Sections render dynamically via `sortedSectionKeys` from settings
- Split "AI Usage" into `ai_claude` and `ai_codex` (independently toggleable)
- 8 sections: ai_claude, ai_codex, weekly_chart, compute, storage_network, hardware, bluetooth, history
- Reorder with ▲▼ arrows in Settings
- Toggle visibility with checkboxes

### Collapsible sections
- `SystemSection` has `collapsible` prop (default: true)
- `<button>` element for a11y, chevron indicator

### History view
- `src/lib/HistoryView.svelte` — two tabs: Battery and Network
- Battery: sparkline of health_percent (30 days), current health %, cycle count
- Network: bar chart of daily download/upload (14 days), period totals

### SQLite persistence
- `battery_history` table: percent, health_percent, cycle_count, charging (every 30 min)
- `network_daily` table: date, total_download_bytes, total_upload_bytes (accumulated each tick)
- Tauri commands: `get_battery_history`, `get_network_daily`

### Live polling interval
- System metrics loop uses `tokio::time::sleep` + reads SettingsState each iteration
- Slider changes take effect immediately without restart

### Settings migration
- `migrate_settings()` merges missing section keys from defaults
- Old configs auto-upgraded on load

### Backdrop-filter fix
- Moved to `body::before` pseudo-element to prevent flickering on DOM changes
- Sections use `contain: layout style` to isolate repaints

## Known issues
- **SMC temperatures**: Apple Silicon M4 blocks IOKit SMC access. Temperature section empty.
- **Bluetooth**: only connected devices shown (system_profiler limitation)
- **Fans**: MacBook Air M4 is fanless — 0 fans is correct
- **DMG bundling**: sometimes fails, .app always builds fine
- **History**: needs data to accumulate (battery every 30min, network each tick)

## Next priorities

### Polish
- Glass theme: sparkline contrast on translucent background
- Smooth animation on new sparkline data points
- Network sparkline scaling (auto-scale to peak)

### Deferred
- GPU utilization (IOReport framework, undocumented Apple API)
- System metrics SQLite history for CPU/RAM (currently real-time only, sparklines reset on restart)
- Global hotkey to open popup
- Per-core CPU: show P-core vs E-core labels

## Key files

| File | Purpose |
|---|---|
| `src/lib/SparkChart.svelte` | SVG area sparkline component |
| `src/lib/CpuCores.svelte` | Per-core CPU mini sparkline grid |
| `src/lib/HistoryView.svelte` | Battery/Network history tabs |
| `src/lib/SystemSection.svelte` | Collapsible section wrapper |
| `src/lib/SettingsPage.svelte` | Settings: theme, chart mode, sections, polling |
| `src/lib/UsageBar.svelte` | Progress bar (AI usage, disk) |
| `src/lib/WeeklyChart.svelte` | 7-day AI usage bar chart |
| `src/routes/+page.svelte` | Main UI: dynamic section ordering, sparkline history |
| `src-tauri/src/lib.rs` | App setup, polling loops, tray, window |
| `src-tauri/src/db.rs` | SQLite: usage, battery, network tables |
| `src-tauri/src/settings.rs` | Settings load/save/migrate/defaults |
| `src-tauri/src/system_monitor.rs` | sysinfo + battery metrics |
| `src-tauri/src/swift_bridge.rs` | FFI to Swift dylib (SMC/Bluetooth) |
| `src-tauri/src/tray_text.rs` | Format tray title string |
| `src/lib/themes.ts` | 6 themes + applyTheme() |

## Build & run

```bash
cd /Users/vladislavkonovalov/aiUsagebar
npm run dev          # dev mode (Vite + Electron) — use native terminal!
npx tauri build      # release build → src-tauri/target/release/bundle/macos/VibeUsageBar.app
```

Deploy:
```bash
pkill -f vibeusagebar; sleep 1
rm -rf /Applications/VibeUsageBar.app
cp -R src-tauri/target/release/bundle/macos/VibeUsageBar.app /Applications/
open /Applications/VibeUsageBar.app
```
