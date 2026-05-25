---
when: 2026-05-23T11:44:07Z
why: Add a per-user installation path so Titrax can be integrated without system-wide installation.
what: Added a user-mode install script that installs the binary, icon, and desktop entry under XDG user directories.
model: github-copilot/gpt-5.3-codex
tags: [install, desktop, icon, gtk4, xdg, worklog]
---
Added `scripts/install-user.sh` to install Titrax into `~/.local/bin`, `~/.local/share/icons/`, and `~/.local/share/applications/` without requiring root. Updated `README.md` to document the user install flow and the new script. Bumped the app version to `0.3.4`.