# 04 - Implementation Roadmap & Folder Structure

## 1. Directory Structure

```text
linkrouter/
├── .spec/                      # SDD Specifications
│   ├── 01-product-vision.md
│   ├── 02-architecture-ipc.md
│   ├── 03-functional-spec.md
│   └── 04-implementation-roadmap.md
├── src/                        # Frontend (Svelte 5 + TS)
│   ├── lib/
│   │   ├── components/         # Prompter, RuleEditor, BrowserCard, Settings panels
│   │   ├── i18n/               # NEW — locale dictionaries + registry
│   │   │   ├── index.ts        # supported locales, t() helper, system-locale resolution
│   │   │   ├── en.ts           # English dictionary (default / fallback)
│   │   │   └── es.ts           # Spanish dictionary
│   │   ├── services/           # IPC wrappers (tauri.ts, config.ts)
│   │   ├── stores/             # Runes stores (appState.svelte.ts: locale + theme + settings)
│   │   └── utils/              # URL formatters, visual helpers, theme resolver
│   ├── routes/                 # Views: Prompter (/), Settings (/settings)
│   ├── app.css                 # CSS custom properties + [data-theme] light/dark tokens
│   └── App.svelte
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── adapters/
│   │   │   ├── config/         # NEW — config.json loader/saver (atomic write, validation)
│   │   │   │   ├── mod.rs      # AppConfig, ConfigError, load/save/watch
│   │   │   │   └── path.rs     # XDG config path resolution (~/.config/link-router/)
│   │   │   ├── detectors/      # Per-OS modules (linux.rs, macos.rs, windows.rs)
│   │   │   ├── dispatcher.rs   # show_prompt (always-on-top), launch routing
│   │   │   ├── launchers/
│   │   │   ├── commands/       # IPC command handlers (+ get_config/save_settings/system_locale)
│   │   │   └── persistence/json.rs
│   │   ├── core/               # domain, rules engine, route_service, sanitizer, error
│   │   ├── lib.rs              # tray menu (Mostrar/Cerrar/Salir), close-to-hide, setup
│   │   └── main.rs
│   ├── capabilities/           # Tauri v2 Security Capabilities
│   └── tauri.conf.json
└── ~/.config/link-router/config.json   # runtime user config (not in repo)
```

## 2. Roadmap phases (SDD: spec → implement → verify)

### Phase 0 — v1 (DONE)
- Deep-link/single-instance interception, browser detection, rules engine, URL cleaner/unshortener.
- Prompter window (always-on-top when no rule matches), system tray (Mostrar/Cerrar/Salir), close-to-hide.
- IPC: `dispatch://incoming-url`, `open_in_browser`, rule CRUD, `list_browsers`.
- Packaging: GitHub Actions release workflow (`.github/workflows/release.yml`) → Linux `.deb/.rpm/.AppImage`, Windows `.msi/.exe`, macOS universal `.dmg`.

### Phase 1 — Multi-language (Feature 5)
1. `src/lib/i18n/{en,es}.ts` dictionaries + `t()` helper + supported-locale registry.
2. `locale` Rune in `appState.svelte.ts` with resolution order: setting → system → `en`.
3. `system_locale` invoke (Rust) returning OS locale tag; map `es*` → `es`, else `en`.
4. Settings UI: language selector (`system | Español | English`), persisted via `save_settings`.
5. **Verify:** AC of Feature 5 (system-es, unsupported→en, explicit override, no raw keys).

### Phase 2 — Themes (Feature 6)
1. CSS custom-property tokens in `app.css` + `[data-theme="light|dark"]`.
2. `theme` Rune resolving `system` via `prefers-color-scheme`; apply attribute on root.
3. Settings UI: theme selector (`system | light | dark`), persisted.
4. **Verify:** AC of Feature 6 (reactive system switch, explicit override, no FOUC).

### Phase 3 — Config file (Feature 7)
1. Rust `adapters/config/`: path resolution (XDG → `~/.config/link-router/config.json`), load/validate/atomic-save, `version` migrations.
2. IPC: `get_config`, `save_config`, `get_settings`, `save_settings`.
3. Wire rules engine + settings UI to config-backed persistence (replace in-memory-only store).
4. Optional v1.1: file watcher → `config://updated` event.
5. **Verify:** AC + scenario table of Feature 7 (missing/invalid/atomic/external edits).

### Phase 4 — Hardening & release
- i18n coverage audit (no missing keys in `es`/`en`), theme contrast checks, config migration test.
- Update README (config path, language/theme settings), bump version, tag → CI release.

## 3. Definition of Done (per feature)
- Acceptance Criteria in `.spec/03-functional-spec.md` are all green.
- Specs updated first (SDD), then code, then `cargo check` + `clippy` + `npm run check` clean.
- No regression on Phase 0 behaviors (tray, prompter, rules).
