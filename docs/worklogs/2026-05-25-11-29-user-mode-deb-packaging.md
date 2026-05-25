---
when: 2026-05-25T11:29:48Z
why: Make it easy for a normal user to build and install a .deb package without system-wide installation.
what: Added a user-mode .deb build script with optional ~/.local install path and documented the workflow in README.
model: github-copilot/gpt-5.3-codex
tags: [packaging, deb, install, user-mode, docs, worklog]
---
Added `scripts/build-user-deb.sh` to build a local `.deb` package from the current development binary (debug-first) and optionally install it to `~/.local` with `--install`. Updated `README.md` with user-focused build/install instructions and script usage, and updated `.gitignore` to exclude local package artifacts (`dist/` and `*.deb`). Bumped the app version to `0.3.7`.