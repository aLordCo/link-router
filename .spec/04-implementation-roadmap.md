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
│   │   ├── components/         # Prompter, RuleEditor, BrowserCard
│   │   ├── services/           # IPC wrappers (tauri.ts)
│   │   ├── stores/             # Stores / Runes (appState.svelte.ts)
│   │   └── utils/              # URL formatters, visual helpers
│   ├── routes/                 # Views: Prompter (/), Settings (/settings)
│   └── App.svelte
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── browser_detector/   # Per-OS modules (linux.rs, macos.rs, windows.rs)
│   │   ├── rules_engine/       # Evaluator, wildcards, regex
│   │   ├── url_cleaner/        # UTM removal & unshortener
│   │   ├── commands/           # IPC command handlers
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── capabilities/           # Tauri v2 Security Capabilities
│   └── tauri.conf.json
```
