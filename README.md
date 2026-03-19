# aiUsageBar

macOS menubar app for tracking AI coding assistant usage limits. Shows real-time session and weekly utilization for Claude Code and Codex CLI.

## Features

- **Claude Code** — 5-hour session + 7-day weekly usage, extra credits
- **Codex CLI** — 5-hour session + weekly usage, credits balance
- Color-coded progress bars (green → yellow → red)
- Native macOS notifications at 80% and 95% thresholds
- 7-day usage history chart
- OAuth token auto-refresh
- No dock icon — lives entirely in the menubar

## Requirements

- macOS 14+
- [Rust toolchain](https://rustup.rs/)
- Node.js 18+
- Claude Code logged in (`claude login`)
- Codex CLI logged in (`codex login`) — optional

## Install

```bash
git clone https://github.com/Dev1love/aiUsageBar.git
cd aiUsageBar
npm install
npx tauri build
```

The `.app` bundle will be in `src-tauri/target/release/bundle/macos/`. Drag it to `/Applications`.

## Development

```bash
npm install
npx tauri dev
```

## How It Works

| Provider | Data Source | Auth |
|----------|-----------|------|
| Claude Code | `api.anthropic.com/api/oauth/usage` | OAuth token from macOS Keychain |
| Codex CLI | `chatgpt.com/backend-api/wham/usage` | JWT from `~/.codex/auth.json` |

Polls every 5 minutes. Each provider is independent — if one fails, the other still shows.

## Tech Stack

- **Tauri v2** — native macOS runtime
- **Svelte 5** — frontend
- **Rust** — backend (API, keychain, SQLite, notifications)
- **SQLite** — usage history storage

## License

MIT
