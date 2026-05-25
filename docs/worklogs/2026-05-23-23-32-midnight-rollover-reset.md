---
when: 2026-05-23T23:32:01Z
why: Fix missing day rollover reset so tracked minutes do not continue indefinitely past midnight.
what: Added automatic midnight rollover handling that flushes the previous day and resets project totals for the new day.
model: github-copilot/gpt-5.3-codex
tags: [bugfix, rollover, data, timer, gtk4, worklog]
---
Updated `src/app.rs` with `rollover_if_new_day()` and `current_day` tracking to mirror the original app behavior of flushing and resetting at day boundaries. Added `day_file_for()` in `src/data.rs` and wired rollover calls in `src/ui.rs` autosave, minute tick, and close handlers so day transitions are handled automatically. Bumped the app version to `0.3.6`.