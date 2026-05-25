---
when: 2026-05-25T11:33:49Z
why: Add a clear uninstall path for user-mode installs extracted into ~/.local.
what: Added uninstall-user.sh and README guidance for user-mode uninstall behavior.
model: github-copilot/gpt-5.3-codex
tags: [packaging, uninstall, docs, version]
---
Added `scripts/uninstall-user.sh` with safe file removal and `--dry-run` support for user-mode installs in `~/.local`. Updated `README.md` to document the uninstall command and clarify that `dpkg remove` does not manage extraction-based `--install` mode. Bumped version to `0.3.8` in `Cargo.toml`.
