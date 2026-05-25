---
when: 2026-05-23T11:47:46Z
why: Make the restored legacy icon readable in both dark and light desktop themes.
what: Rebuilt the Titrax icon with a neutral background and border, keeping the old logo centered.
model: github-copilot/gpt-5.3-codex
tags: [ui, icon, gtk4, desktop, theme, worklog]
---
Updated `share/icons/hicolor/64x64/apps/titrax.png` so the legacy icon now has a visible neutral background and border instead of transparent edges. This makes the app icon easier to see in both dark and light desktop themes while preserving the old Titrax logo. The build still passes after the icon asset update.