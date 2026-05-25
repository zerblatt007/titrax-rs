---
when: 2026-05-23T11:39:06Z
why: Make the restored legacy icon visible when running the app directly from the repo without a desktop entry.
what: Added runtime icon theme search paths so GTK can resolve the legacy Titrax icon locally.
model: github-copilot/gpt-5.3-codex
tags: [ui, icon, gtk4, packaging, runtime, worklog]
---
Updated `src/ui.rs` to register local icon theme search paths on startup, which lets GTK resolve `titrax.png` from `share/icons/` even when the app is launched directly from the repository. This keeps the restored legacy icon visible without requiring `titrax.desktop` for source-tree runs, while the desktop entry remains useful for normal launcher integration. Bumped the app version to `0.3.3`.