# aiUsageBar

macOS menubar app for tracking AI coding assistant usage limits. Shows session and weekly utilization with notifications before hitting limits.

![screenshot](https://github.com/user-attachments/assets/placeholder.png)

## Features

- Real-time session (5-hour) and weekly (7-day) usage tracking
- Extra usage credits display
- Color-coded progress bars (green / yellow / red)
- Native macOS notifications at 80% and 95% thresholds
- 7-day usage history chart
- SQLite storage for usage history
- OAuth token auto-refresh

## Requirements

- macOS 14+
- Claude Code CLI installed and logged in (`claude login`)
- Rust toolchain
- Node.js 18+

## Development

```bash
npm install
npx tauri dev
```

## Build

```bash
npx tauri build
```

The `.app` bundle will be in `src-tauri/target/release/bundle/macos/`.

## Tech Stack

- **Tauri v2** — native macOS app runtime
- **Svelte 5** — frontend UI
- **Rust** — backend (API polling, keychain, SQLite, notifications)
- **Anthropic OAuth API** — usage data source

## How It Works

1. Reads OAuth token from macOS Keychain (stored by `claude login`)
2. Polls `api.anthropic.com/api/oauth/usage` every 60 seconds
3. Displays utilization in menubar tray icon and popup window
4. Fires native notifications when approaching limits
