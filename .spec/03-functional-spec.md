# 03 - Functional Specifications & Use Cases

## Feature 1: Browser & Profile Discovery

### Acceptance Criteria
- **GIVEN** that LinkRouter launches for the first time,
- **WHEN** a system scan runs on Linux, macOS, or Windows,
- **THEN** the app identifies browsers installed (including Flatpak/Snap installs on Linux, non-standard paths on Windows/macOS) and their user profiles.

---

## Feature 2: Rule Engine & URL Patterns

### Matching Scenarios
1. **Wildcard Domain Pattern:**
   - Rule: `*.company.com/*` → Open in `Google Chrome (Work Profile)`.
2. **Source App:**
   - Rule: If the app that emits the click is `Slack` or `Microsoft Teams` → Open in `Brave (Work)`.
3. **Tracking Parameter Cleaning (URL Cleaner):**
   - Incoming URL: `https://example.com/product?id=123&utm_source=newsletter&fbclid=xyz`
   - Processed URL: `https://example.com/product?id=123`
4. **Short Link Unshortening:**
   - Detect `t.co`, `bit.ly`, `tinyurl.com` → silent HTTP HEAD request in Rust to evaluate the rule with the final destination URL.

---

## Feature 3: Default Handler State

### Acceptance Criteria
- **GIVEN** that the user enters the LinkRouter settings panel,
- **WHEN** the user clicks "Set as default browser",
- **THEN** on macOS calls `LSSetDefaultHandlerForURLScheme`, on Linux updates `~/.config/mimeapps.list` natively (fallback to `xdg-settings`), and on Windows opens the system associations dialog/module.

---

## Feature 4: Lifecycle, Background Execution (System Tray) & Foreground Window

### Acceptance Criteria
- **GIVEN** that LinkRouter is running,
- **WHEN** the user presses the window close button or the `Esc` key,
- **THEN** the window hides transparently by intercepting `WindowEvent::CloseRequested` with `api.prevent_close()`, keeping the process active in the background.
- **AND** the app shows a notification-area icon (System Tray) next to the system clock.
- **WHEN** the user clicks a link from an external app and no automatic rule matches,
- **THEN** the LinkRouter window is brought to the foreground above all other active apps (`set_always_on_top(true)`, `show()`, `set_focus()`).
- **WHEN** the user left-clicks the tray icon or selects "Show LinkRouter" in the context menu,
- **THEN** the main window is restored and focused on screen (`show()` and `set_focus()`).
- **WHEN** the user selects "Close" from the tray context menu,
- **THEN** the app exits cleanly (`app.exit(0)`).

---

## Feature 5: Multi-language (i18n) — Spanish & English

### Acceptance Criteria
- **GIVEN** that LinkRouter launches for the first time and `settings.locale` is `"system"`,
- **WHEN** the OS system locale is Spanish (e.g. `es-AR`, `es-ES`),
- **THEN** the UI renders in **Spanish**.
- **GIVEN** that the system locale is **not** supported (e.g. `fr-FR`, `de-DE`),
- **WHEN** LinkRouter resolves the effective locale,
- **THEN** it falls back to **English** (`en`), never showing untranslated keys or raw key names.
- **GIVEN** that the user is in LinkRouter Settings,
- **WHEN** the user selects an explicit language (`Español` or `English`),
- **THEN** the change applies **immediately** to the whole UI (prompter, settings, tray-related strings) without restarting the app, and persists to `~/.config/link-router/config.json`.
- **GIVEN** that `settings.locale` is `"system"`,
- **WHEN** the user changes the OS language while LinkRouter is running,
- **THEN** LinkRouter re-resolves the locale on next launch (v1: on restart; file watcher for live OS-locale changes is a v1.1 non-goal).
- **Non-functional:** locale dictionaries live in `src/lib/i18n/{en,es}.ts`; adding a new locale must not require code changes outside the dictionary file + a registry entry (open/closed for new languages).

### Matching / UX scenarios
1. Default → system Spanish → all labels in Spanish (`Abrir en…`, `Configuración`, `Cerrar`).
2. Explicit English override → UI in English even if the OS is Spanish.
3. Unsupported system locale → English.

---

## Feature 6: Theme support (dark / light)

### Acceptance Criteria
- **GIVEN** that `settings.theme` is `"system"` (default),
- **WHEN** the OS is in dark mode,
- **THEN** LinkRouter renders in **dark** theme; when the OS switches to light, the app switches too (reactively, no restart).
- **GIVEN** that the user selects an explicit theme in LinkRouter Settings,
- **WHEN** the user picks `Light` or `Dark`,
- **THEN** the app ignores the OS preference and applies the chosen theme immediately, persisting it to `config.json`.
- **AND** the theme is consistent across: prompter window, settings panel, system-tray tooltip/menu surfaces where applicable, and the window background (no flash of the wrong theme on startup).
- **Non-functional:** implementation uses CSS custom properties + a `data-theme="light|dark"` attribute on the root element; the theme Rune store resolves `system` via `prefers-color-scheme`.

---

## Feature 7: External configuration file (`~/.config/link-router/config.json`)

### Acceptance Criteria
- **GIVEN** that LinkRouter starts,
- **WHEN** `~/.config/link-router/config.json` exists and is valid,
- **THEN** all rules and application settings are loaded from that file and used by the rules engine and UI.
- **GIVEN** that the config file does **not** exist,
- **WHEN** the app starts,
- **THEN** it runs with built-in defaults (empty rules, `locale: "system"`, `theme: "system"`) and creates the file on the first settings/rules save.
- **GIVEN** that the config file is **invalid** (malformed JSON / wrong schema),
- **WHEN** LinkRouter parses it,
- **THEN** it logs a warning, falls back to the last-good or default config, and **never crashes**; the user can fix the file manually and reload.
- **GIVEN** that the user edits rules or settings in the LinkRouter UI,
- **WHEN** the change is saved,
- **THEN** the file is written **atomically** (no partial/corrupt file on crash) and other running readers never observe an intermediate state.
- **GIVEN** that the file is edited externally (e.g. by hand or by a dotfiles sync),
- **WHEN** LinkRouter is running,
- **THEN** (v1) the next launch picks up the changes; (v1.1, optional) a file watcher emits `config://updated` and the UI refreshes live.
- **Non-functional:** the file path is exactly `~/.config/link-router/config.json` (XDG), UTF-8, pretty-printed JSON with `version` for forward migrations.

### Scenario table
| # | Scenario | Expected |
|---|----------|----------|
| 1 | First run, no file | Defaults in memory; file created on first save |
| 2 | Valid file with 3 rules | Rules engine uses those 3 rules at launch |
| 3 | File contains invalid JSON | Warning + defaults; app stays alive |
| 4 | User adds a rule in UI | `config.json` updated atomically; rule active immediately |
| 5 | Hand-edited `locale: "es"` | Next launch renders UI in Spanish |
| 6 | Hand-edited `theme: "dark"` | Next launch renders dark theme |
