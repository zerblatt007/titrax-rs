---
when: 2026-05-22T17:55:16Z
why: Verify the GTK4/Rust rewrite compiles cleanly after initial implementation
what: Release build of titrax v0.2.0 confirmed successful; 2 dead-code warnings noted
model: github-copilot/claude-sonnet-4.6
tags: [rust, gtk4, build, titrax]
---

Ran `cargo build --release` on the titrax v0.2.0 Rust codebase. Build completed successfully with no errors; binary produced at `target/release/titrax`. Two dead-code warnings were emitted for `AppState::move_project` (src/app.rs:187) and the helper `adjust_index` (src/app.rs:197), which are implemented but not yet wired to any UI action.
