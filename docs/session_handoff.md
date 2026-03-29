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
