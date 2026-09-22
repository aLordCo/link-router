# Changelog

All notable changes to LinkRouter are documented in this file, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Each release's
section is picked up by the CI to fill the GitHub release notes.

## [0.2.1] - 2026-09-22

Maintenance release focused entirely on macOS.

### Fixed

- **macOS: browsers not detected.** The detector joined application
  directories with the display name instead of the `.app` bundle name
  (`/Applications/Safari` instead of `/Applications/Safari.app`), so the
  browser list was always empty. Now also scans `/System/Applications`,
  `/Network/Applications` and `~/Applications`.
- **macOS: set LinkRouter as the default browser.** Previously unsupported
  on macOS. “Set as default” now re-registers the bundle with
  LaunchServices and opens **System Settings → Desktop & Dock** so you can
  pick LinkRouter as the http/https handler (macOS has no API to switch it
  programmatically). The “Check” button reads the current default handler
  back from LaunchServices.

### Notes

- Version reported by the installers was bumped to 0.2.1.
- Installers: Linux (`.deb`/`.rpm`/AppImage), Windows (MSI + NSIS), macOS
  (universal `.dmg`, Apple Silicon + Intel).

> Tip: if LinkRouter doesn't appear in the “Default web browser” dropdown,
> click “Set as default” (it force-registers the app) and select
> “LinkRouter” from the list.

## [0.2.0] - 2026-09-22

First public release. LinkRouter is a tray-first desktop app that routes
each link you open to your preferred browser or browser profile, based on
configurable rules.

### New features

- **Internationalization (ES/EN)** — auto-detects the system locale,
  switchable in settings and persisted. Spanish uses a neutral LatAm tone.
- **Dark & light themes** — follows the system preference with live in-app
  switching; contrast tuned (WCAG AA) for both palettes.
- **External JSON configuration** — full config (rules, locale, theme) is
  persisted to an editable file, validated on load/save, with atomic writes
  and automatic migration from the legacy storage layout.
- **Tray-first UX** — the app lives in the system tray. The main window
  stays out of the taskbar; the tray menu offers a single Show / Quit entry,
  and closing the window hides it instead of quitting.
- **Browser detection on all platforms** — Linux (desktop entries + fallback
  paths), Windows (registry + install-location fallback) and macOS
  (application bundles, including Chrome/Brave/Edge/Vivaldi/Arc/Orion/Zen
  profiles).
- **New app icon** applied to every platform format.

### Fixed

- Windows/macOS CI packaging: missing `.ico`/`.icns` prevented installer
  builds — icon set generated for all formats and bundler config updated.
- Legacy GitHub Actions (`checkout@v4`, `setup-node@v4`) replaced with v5
  (Node 20 deprecation).
- Linux: main-window icon set explicitly at startup; skips the taskbar
  reliably via X11 backend.
- Windows: registry browser detection now de-duplicates entries across
  registry hives.
- Build warnings eliminated on Windows and macOS (dead-code gating for
  platform-specific helpers, type fixes). Clippy passes with `-D warnings`.

### Notes

- Installers: Linux (`.deb`/`.rpm`/AppImage), Windows (MSI + NSIS), macOS
  (universal `.dmg`, Apple Silicon + Intel).