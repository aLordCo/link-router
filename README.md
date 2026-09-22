# LinkRouter

Smart deep-link router & browser-selector prompter. A fast, lightweight, cross-platform (Linux, macOS, Windows) desktop app built with **Tauri 2** + **Svelte 5** (SvelteKit) + **Rust**.

When you click a link in any non-browser app (Slack, Discord, Teams, mail client…), LinkRouter intercepts it and either:

- opens it automatically in the right **browser and profile** based on your rules (URL patterns, source app, final destination after unshortening, cleaned of `utm_*` tracking params), or
- shows a fast keyboard-driven **prompter** (on-top, focused) to pick a browser/profile on the fly.

It stays in the **system tray**, hides on close, and relocates links with zero clicks when a rule matches.

## Tech Stack & Dependencies

- **GUI:** Tauri v2 (Rust, `tray-icon` feature) + [Tauri Plugins](https://github.com/tauri-apps/plugins-workspace): `deep-link`, `single-instance`, `clipboard-manager`
- **Frontend:** Svelte 5 / SvelteKit (TypeScript, Tailwind CSS)
- **Backend:** Rust 1.77+ (edition 2021) — `winreg` on Windows, `env_logger`/`log` for diagnostics, `url`/`regex`/`thiserror`/`serde`
- **Runtimes:** Node.js 20+ and npm (or pnpm) for the frontend toolchain

## Requirements

- [Rust](https://rustup.rs) toolchain >= 1.77 (stable)
- [Node.js](https://nodejs.org) >= 20 + npm
- Linux: GTK3, WebKit2GTK-4.1, libappindicator3 (tray), librsvg — install via your distro (`apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`)
- macOS: no extra deps; Windows: no extra deps

## Build & Run

```bash
git clone <repo-url>
cd link-router
npm install

# Dev (hot-reload + debug console)
npm run tauri dev

# Type-check the frontend
npm run check        # svelte-check + tsc

# Lint the Windows (Rust) code
(cd src-tauri && cargo check && cargo clippy --all-targets -- -D warnings)

# Production bundle (Linux .deb/.AppImage, macOS .dmg/.app, Windows .msi/.exe)
npm run tauri build
```

## IPC Map (Rust ⇄ UI)

| Event/Command | Direction | Payload |
|---|---|---|
| `dispatch://incoming-url` | Tauri → UI | `{ url, sourceApp }` |
| `open_in_browser` | UI → Tauri | `{ url, browserId, profileId? }` |
| `get_rules` / `create_rule` / `update_rule` / `delete_rule` | UI → Tauri | Rule CRUD |
| `list_browsers` | UI → Tauri | Detected browsers + profiles |

## Specs
See [`/spec`](./.spec/): Product Vision, Architecture & IPC, Functional Spec, Implementation Roadmap.
