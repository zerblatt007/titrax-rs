---
when: 2026-05-23T11:34:02Z
why: Restore the legacy Titrax icon for modern desktop launchers and window titlebars.
what: Added the old Titrax icon asset and a desktop entry, then wired the GTK window to use the titrax icon name.
model: github-copilot/gpt-5.3-codex
tags: [ui, icon, desktop, gtk4, packaging, worklog]
---
Restored the legacy Titrax icon by converting the original `titrax.xbm` into a PNG under `share/icons/hicolor/64x64/apps/` and adding `share/applications/titrax.desktop`. Updated `src/ui.rs` so the GTK window uses the `titrax` icon name, and documented the launcher/icon setup in `README.md`. Bumped the app version to `0.3.2`.