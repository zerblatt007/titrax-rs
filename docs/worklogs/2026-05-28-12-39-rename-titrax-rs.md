---
when: 2026-05-28T12:39:56Z
why: Disambiguate the Rust rewrite from the original 1996 C program, which shares the same binary name.
what: Renamed app from titrax to titrax-rs across binary, desktop entry, icon, config dir, APP_ID, scripts, and docs.
model: github-copilot/claude-sonnet-4.6
tags: [rename, titrax-rs, app-id, desktop, packaging]
---
Renamed the Rust application from `titrax` to `titrax-rs` throughout the repository. Changes include: `Cargo.toml` package name, binary name in all scripts (`install-user.sh`, `uninstall-user.sh`, `build-user-deb.sh`), icon asset (`titrax.png` → `titrax-rs.png`), desktop entry (`titrax.desktop` → `titrax-rs.desktop`), GTK `APP_ID` (`no.uninett.titrax` → `io.github.zerblatt007.titrax-rs`), config directory (`~/.config/titrax/` → `~/.config/titrax-rs/`), and all references in `README.md`, `BLUEPRINT.md`, and `docs/PROJECT_RULES.md`. The `TITRAXDIR` env var and `~/.TimeTracker` data directory are unchanged for backward compatibility. Bumped version to 0.4.0.
