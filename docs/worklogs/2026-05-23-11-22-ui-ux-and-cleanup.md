---
when: 2026-05-23T11:22:40Z
why: Record the UI/layout cleanup and persistence changes that were finalized in this session.
what: Side-panel controls, compact list styling, font scaling persistence, and removal of unstable window position storage.
model: github-copilot/gpt-5.3-codex
tags: [ui, gtk4, layout, config, cleanup, worklog]
---
Updated `src/ui.rs`, `src/data.rs`, `src/config.rs`, and `Cargo.toml` to keep the project list compact, move controls to a side panel, persist font size and window size in `~/.config/titrax/config.toml`, and remove the unreliable window position persistence. Also cleaned up the unused transfer dialog helper and bumped the app version to `0.3.1`.