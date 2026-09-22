# 02 - Architecture & IPC Specification

## 1. Architecture Overview

```text
[ Native System OS Event ]
   (Clic en enlace en Slack/Discord/Browser)
                     │
                     ▼
[ Single-Instance / DeepLink Handler ]
                     │
   ├── (1) Sanitizes URL (URL Cleaner)
   │
   ├── (2) Evaluates rules engine (Rules Engine - Fast path)
   │       └─► Rules loaded from ~/.config/link-router/config.json
   │             ├─► Exact match without prompt?
   │             │        ├─► YES ─► [Launch target browser (Process Launcher)]
   │             │        │              └─► Window remains hidden / background
   │             │        └─► NO ── (3) visual selector required
   │             │
   │             └─► NO / missing file ─► Built-in defaults + prompt
   │
   └── (3) Shows / focuses Tauri window (`show`, `set_always_on_top`, `set_focus`)
           └─► Emits IPC event `dispatch://incoming-url`
                     │
                     ▼
            [ Svelte 5 UI Prompter ]
            (i18n Rune store + theme Rune store applied)
                     │
                     ▼
            [ Rust Command: `open_in_browser` ]
                     │
                     ▼
            [ Hides window (`hide`) ]

[ Config Loader (Rust) ]
   ├─► Reads ~/.config/link-router/config.json at startup (XDG config home)
   ├─► Validates + deserializes into AppConfig { settings, rules }
   ├─► On parse error: keep last-good / defaults, log warning, do not crash
   └─► Watcher (optional v1.1): reload on file change → emit `config://updated`

[ System Tray / Lifecycle Manager ]
   ├─► Catch `WindowEvent::CloseRequested` ─► `api.prevent_close()` + `window.hide()`
   ├─► "Show LinkRouter" menu item           ─► `window.unminimize()` + `show()` + `set_focus()`
   └─► "Quit" menu item (only exit option)   ─► `app.exit(0)`
```

## 2. IPC Channel Map

| Event / Command                        | Direction       | Payload                                             |
| -------------------------------------- | --------------- | --------------------------------------------------- |
| `dispatch://incoming-url`              | Tauri → UI      | `{ url, sourceApp }`                                |
| `open_in_browser` (invoke)             | UI → Tauri      | `{ url, browserId, profileId? }`                    |
| `get_rules` / `create_rule` / ...      | UI → Tauri      | Rule CRUD (backed by config.json)                   |
| `list_browsers`                        | UI → Tauri      | Detected browsers + profiles                        |
| `get_config` (invoke)                  | UI → Tauri      | → `AppConfig { settings, rules }`                   |
| `save_config` (invoke)                 | UI → Tauri      | Full config (validated) → writes config.json (atomic write: tmp + rename) |
| `get_settings` (invoke)                | UI → Tauri      | → `{ locale, theme, ... }`                          |
| `save_settings` (invoke)               | UI → Tauri      | Partial settings update → persists to config.json   |
| `system_locale` (invoke)               | UI → Tauri      | → OS locale tag (e.g. `es-AR`, `en-US`)             |
| `config://updated`                     | Tauri → UI      | `AppConfig` (file watcher / after save) — v1.1 optional |
| Tray menu events ("show" / "quit")     | OS → Tauri      | Show-window / `app.exit(0)`                          |
| `WindowEvent::CloseRequested`          | OS → Tauri      | intercept → hide window (close-to-tray)             |

## 3. Config file contract

**Path:** `$XDG_CONFIG_HOME/link-router/config.json` → defaults to `~/.config/link-router/config.json`
(Cross-platform note: on Windows/macOS the same XDG-style path is used for consistency; if the platform provides no XDG support, fall back to `~/.config/link-router/config.json`.)

```jsonc
{
  "version": 1,
  "settings": {
    "locale": "system",          // "system" | "es" | "en"
    "theme": "system",           // "system" | "light" | "dark"
    "launchAtLogin": false,
    "prompterPosition": "center"
  },
  "rules": [
    { "id": "r1", "pattern": "*.company.com/*", "browserId": "google-chrome", "profileId": "work" },
    { "id": "r2", "pattern": "https://github.com/*", "browserId": "firefox", "profileId": null }
  ]
}
```

**Behavior:**
- Missing file → create with defaults on first save (or run with in-memory defaults).
- Invalid JSON / schema → log + fall back to defaults; never crash the app.
- Writes are **atomic** (write to `config.json.tmp`, then rename over `config.json`).
- `version` field allows future migrations.
- Legacy v0.1 config at `~/.config/linkrouter/config.json` (plain rule array) is recognized, migrated to the new location/format, and the legacy file removed (first run only).

**Window/tray icon:** window and tray use the same embedded bundle icon set (PNG + `.ico` + `.icns`). On Linux the window sets its icon at startup and is excluded from the taskbar via `skip-taskbar` (the app runs under the X11/XWayland backend, because native Wayland has no skip-taskbar hint and Mutter ignores it); the tray-carrying strings are rebuilt when the locale changes.

## 4. Frontend state (Svelte 5 Runes)

```ts
// src/lib/stores/appState.svelte.ts (extended)
// locale: 'system' | 'es' | 'en';  theme: 'system' | 'light' | 'dark'
// Both are $state runes; effective locale/theme resolved reactively:
//   locale = settings.locale === 'system' ? systemLocale : settings.locale
//   if (!SUPPORTED.includes(effectiveLocale)) effectiveLocale = 'en'
//   theme   = settings.theme   === 'system' ? systemTheme   : settings.theme
// A `t(key)` helper resolves keys from dictionaries (es.ts / en.ts).
```

**Resolution order (i18n):** explicit setting → system locale → `en` fallback.
**Resolution order (theme):** explicit setting → OS preference (`prefers-color-scheme`) → `light`.
