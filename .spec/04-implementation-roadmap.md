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
│   │   ├── i18n/               # locale dictionaries + registry
│   │   │   ├── index.ts        # supported locales, t() helper, system-locale resolution
│   │   │   ├── en.ts           # English dictionary (default / fallback)
│   │   │   └── es.ts           # Spanish dictionary (neutral LatAm) — typed as Record<MessageKey,string>
│   │   ├── services/           # IPC wrappers (tauri.ts: rules, settings, config, browsers)
│   │   ├── stores/             # Runes stores (appState.svelte.ts: settings + locale + theme)
│   │   └── theme.ts            # theme resolver (system/light/dark) + data-theme apply
│   ├── routes/                 # Views: Prompter (/), Settings (/settings)
│   ├── app.css                 # CSS custom properties + [data-theme] light/dark tokens
│   └── App.svelte
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── adapters/
│   │   │   ├── detectors/      # Per-OS modules (linux.rs, macos.rs, windows.rs)
│   │   │   ├── dispatcher.rs   # show_prompt (always-on-top), launch routing
│   │   │   ├── launchers/
│   │   │   ├── commands/       # IPC handlers (get_config/save_config/get_settings/save_settings/system_locale, rule CRUD, …)
│   │   │   └── persistence/json.rs    # config.json loader/saver: XDG path, atomic write,
│   │   │                         #         validation, legacy (linkrouter/) migration, tests
│   │   ├── core/               # domain, rules engine, route_service, sanitizer, error
│   │   ├── lib.rs              # tray menu (Mostrar/Salir), close-to-hide, setup, window icon/taskbar
│   │   └── main.rs             # Linux: WEBKIT_DISABLE_DMABUF_RENDERER + GDK_BACKEND=x11
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

### Phase 1 — Multi-language (Feature 5) (DONE)
1. `src/lib/i18n/{en,es}.ts` dictionaries + `t()` helper + supported-locale registry.
2. `locale` Rune in `appState.svelte.ts` with resolution order: setting → system → `en`.
3. `system_locale` invoke (Rust) returning OS locale tag; map `es*` → `es`, else `en`.
4. Settings UI: language selector (`system | Español | English`), persisted via `save_settings`.
5. Spanish is neutral Latin American (Colombian reference) — `tú` forms, no voseo.
6. **Verify:** AC of Feature 5 (system-es, unsupported→en, explicit override, no raw keys). ✔

### Phase 2 — Themes (Feature 6) (DONE)
1. CSS custom-property tokens in `app.css` + `[data-theme="light|dark"]`.
2. `theme` Rune resolving `system` via `prefers-color-scheme`; apply attribute on root (pre-mount to avoid FOUC).
3. Settings UI: theme selector (`system | light | dark`), persisted.
4. **Verify:** AC of Feature 6 (reactive system switch, explicit override, no FOUC). ✔

### Phase 3 — Config file (Feature 7) (DONE)
1. Config adapter: XDG path resolution (~/.config/link-router/config.json), load/validate/atomic-save (tmp+rename), `version` migrations — implemented in `adapters/persistence/json.rs`.
2. IPC: `get_config`, `save_config`, `get_settings`, `save_settings` (+ `system_locale`).
3. Rules engine + settings UI wired to config-backed persistence.
4. Legacy `~/.config/linkrouter/config.json` (v0.1 rule array) migrated on first run; legacy file removed.
5. Optional v1.1: file watcher → `config://updated` event (not implemented, per AC non-goal).
6. **Verify:** AC + scenario table of Feature 7 (missing/invalid/atomic/external edits). ✔

### Phase 4 — Hardening & release (DONE)
- i18n coverage audit: `es.ts` typed as `Record<MessageKey, string>` → missing keys are compile errors.
- Theme contrast pass: light text-accent → `#047857` (~5.5:1), dark text-faint → `#808795` (~4.8:1) for small text.
- Config migration test (`linkrouter/` → `link-router/`), missing-file and invalid-JSON tests.
- README: config contract + full IPC map.
- Version bump 0.1.0 → 0.2.0 (package.json, Cargo.toml, tauri.conf.json); tag `v0.2.0`.
- CI release: Linux (.deb/.rpm/.AppImage), Windows (.msi/.exe), macOS (.dmg) — triggered by `v*` tag push.
- Post-release fixes from CI/QA: full bundle icon set (`.ico`/`.icns`) added; workflow actions bumped to v5 (Node 24); main window hidden from taskbar (skip-taskbar + X11/XWayland backend on Linux); tray menu simplified to *Show/Quit*.

## 3. Definition of Done (per feature)
- Acceptance Criteria in `.spec/03-functional-spec.md` are all green.
- Specs updated first (SDD), then code, then `cargo check` + `clippy` + `npm run check` clean.
- No regression on Phase 0 behaviors (tray, prompter, rules).
