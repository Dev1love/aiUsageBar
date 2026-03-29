# VibeUsageBar — Session Handoff (2026-03-24)

## What was done this session

### 1. SparkChart component (DONE)
- New `src/lib/SparkChart.svelte` — SVG area sparkline chart component
- Ring buffer of last 60 samples for CPU, RAM, network down/up
- SVG `<path>` with gradient area fill, color from CSS variables
- Threshold colors: green → yellow (80%) → red (95%)
- Supports `formattedValue` prop for custom display (e.g. network speeds)
- Replaced UsageBar for CPU and RAM in Compute section
- Replaced NetworkSpeed with two SparkCharts (download/upload) in Storage & Network
- Disk stays as UsageBar (static metric, sparkline not useful)

### 2. Section visibility & ordering (DONE)
- Sections now render dynamically via `sortedSectionKeys` from settings
- Split "AI Usage" into separate sections: `ai_claude` and `ai_codex` (independently toggleable)
- Added `weekly_chart` and `bluetooth` as separate sections
- Settings UI: "Popup Sections" with toggles and up/down arrows for reorder
- Rust defaults updated with 7 sections (was 5)
- Settings persist to `settings.json` via existing save mechanism

### 3. Collapsible sections (DONE)
- SystemSection now has `collapsible` prop (default: true)
- Click section header to collapse/expand
- Chevron indicator (▸/▾)
- Proper `<button>` element for a11y
- Hover effect on header

## Known issues
- Same as before: SMC temps blocked on M4, Bluetooth shows only connected devices
- Pre-existing warning in SettingsPage.svelte (settings initial capture) — cosmetic, doesn't affect behavior
- Old `settings.json` files will get sections migrated via `ensureSections()` in SettingsPage

## Next session priorities

### 1. Polish
- Glass theme: chart bars / sparklines contrast (white on translucent)
- Settings: drag & drop for tray item reorder (Phase 5 from spec)
- Smooth sparkline animation on new data points

### 2. Deferred from spec
- GPU utilization (IOReport framework, undocumented Apple API — Phase 5)
- System metrics SQLite history (currently real-time only, sparklines reset on restart)
- Settings polling interval runtime update (currently requires restart)
- Per-core CPU sparkline (data available in `cpu.per_core`)

## Key files

| File | Purpose |
|---|---|
| `src/lib/SparkChart.svelte` | **NEW** — SVG area sparkline component |
| `src/lib/SystemSection.svelte` | **UPDATED** — collapsible sections |
| `src/lib/SettingsPage.svelte` | **UPDATED** — popup sections toggle/reorder UI |
| `src/routes/+page.svelte` | **UPDATED** — dynamic section ordering, sparkline history |
| `src-tauri/src/settings.rs` | **UPDATED** — 7 default sections |
| `src-tauri/src/lib.rs` | Main app: polling loops, tray, window |
| `src-tauri/src/system_monitor.rs` | sysinfo + battery metrics |
| `src/lib/UsageBar.svelte` | Still used for AI usage bars and Disk |
| `src/lib/themes.ts` | 6 theme definitions |
