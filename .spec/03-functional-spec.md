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
