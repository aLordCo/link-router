# 01 - Product Vision & Scope: LinkRouter

## 1. Executive Summary
LinkRouter is a fast, lightweight, cross-platform (Linux, macOS, Windows) smart router and browser selector. It intercepts links opened outside any browser application and instantly routes each URL to the correct browser or specific profile, based on user-defined rules or a fast visual selector (_popup prompter_).

## 2. Competitive Benchmarks & Inspiration
- **Velja & Choosy:** Visual polish, browser-profile support (Chrome/Firefox/Brave), URL shortener and tracking-parameter (UTM) cleaning.
- **Browserosaurus & Junction:** Clean centered popup selector UI with fast keyboard shortcuts (1, 2, 3...) and native integration with Linux environments (XDG/desktop files) and macOS.
- **Finicky:** Flexible rules engine based on pattern matching (domain patterns, source app, regex).

## 3. Platform & Target Specifications
- **Operating Systems:** macOS (11+), Linux (X11/Wayland with XDG), Windows (10/11).
- **Tech Stack:**
  - **Frontend:** Svelte 5 (TypeScript, Tailwind CSS), Svelte 5 Runes for reactive state.
  - **Backend:** Rust (Tauri v2 Core + OS integration crates).
- **Performance:**
  - Startup / window-teardown time: < 150ms.
  - Background RAM: < 45 MB.
  - Final binary size: < 18 MB.

## 4. Key Value Features
1. **Auto-Discovery of Browsers:** Automatic detection of installed browsers and their profiles (including Flatpak/Snap on Linux, non-standard paths on Windows/macOS).
2. **Ultra-Fast Prompter:** Borderless, centered, keyboard-navigable popup selector.
3. **Smart Rules Engine:** URL-pattern matching (`*.company.com/*`), source-app matching (e.g. "if the link comes from Slack → open in Chrome Work Profile"), and regex.
4. **URL Sanitizer:** Automatic removal of tracking parameters (`utm_*`, `fbclid`, `gclid`, `ref_`) via silent Rust HTTP calls.
5. **URL Unshortener:** Detect `t.co`, `bit.ly`, `tinyurl.com` → silent HTTP HEAD request in Rust to evaluate the rule with the final destination URL.
6. **Multi-language (i18n):** UI available in **English** and **Spanish** (neutral Latin American Spanish — Colombian reference; no voseo/regionalisms). The language follows the OS system locale by default (`es*` → Spanish, anything else → English); unsupported locales fall back to **English**. Users can override the language explicitly in LinkRouter Settings. Implemented as a lightweight reactive store based on Svelte 5 Runes; a missing key in either dictionary is a compile-time error.
7. **Theme support (dark/light):** The app follows the system light/dark preference by default, and the user can override it from LinkRouter Settings. Applied instantly across the prompter, settings and system-tray surfaces.
8. **External config file:** All rules and application settings are read from `~/.config/link-router/config.json` (XDG config home), so users can edit, version and sync their configuration with plain files. Legacy v0.1 configs (`~/.config/linkrouter/config.json`) migrate automatically on first run.
9. **Tray-first presence:** LinkRouter runs from the system tray; on Linux the main window is excluded from the OS taskbar (X11 backend + skip-taskbar, as native Wayland ignores the hint), and the tray context menu offers exactly two actions: *Show LinkRouter* and *Quit*.

## 5. Non-Goals (v1)
- Does not manage tabs within a browser.
- Is not an ad-blocker, does not modify page content.
- No cloud sync (v1 is 100% local).
- No RTL/exotic locales yet (only `es` + `en` shipped initially; architecture must allow adding locales without code changes).
