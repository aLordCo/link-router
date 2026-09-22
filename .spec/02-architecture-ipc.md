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
   │       └─► Exact match without prompt?
   │             ├─► YES ─► [Launch target browser (Process Launcher)]
   │             │               └─► Window remains hidden / background
   │             │
   │             └─► NO ── (3) If visual selector required:
   │                         ├─► Shows / focuses Tauri window (`show`, `set_always_on_top`, `set_focus`)
   │                         └─► Emits IPC event `dispatch://incoming-url`
   │                               │
   │                               ▼
   │                      [ Svelte 5 UI Prompter ]
   │                      (Selección manual por Teclado / Mouse)
   │                               │
   │                               ▼
   │                      [ Rust Command: `open_in_browser` ]
   │                               │
   │                               ▼
   │                      [ Hides window (`hide`) ]
   │
   ▼
[ System Tray / Lifecycle Manager ]
   (Proceso persistente minimizado al reloj)
   ├─► Catch `WindowEvent::CloseRequested` ─► `api.prevent_close()` + `window.hide()`
   ├─► Left-click / "Show LinkRouter"       ─► `window.show()` + `set_always_on_top(true)` + `set_focus()`
   └─► "Close" menu item                    ─► `app.exit(0)`
```

## 2. IPC Channel Map

| Event / Command                        | Direction       | Payload                                             |
| -------------------------------------- | --------------- | --------------------------------------------------- |
| `dispatch://incoming-url`              | Tauri → UI      | `{ url, sourceApp }`                                |
| `open_in_browser` (invoke)             | UI → Tauri      | `{ url, browserId, profileId? }`                    |
| `get_rules` / `create_rule` / ...      | UI → Tauri      | Rule CRUD                                           |
| `list_browsers`                        | UI → Tauri      | Detected browsers + profiles                        |
| Tray "show"/"hide" menu events         | OS → Tauri      | `WindowEvent::CloseRequested` intercept             |
