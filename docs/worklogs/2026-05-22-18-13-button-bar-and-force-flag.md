---
when: 2026-05-22T18:13:24Z
why: Replace right-click context menu with a button bar and wire up --force CLI flag test
what: Remove header label, add Edit Time/Delete/Mark/Move 5 min buttons, add force-flag unit test
model: github-copilot/claude-sonnet-4.6
tags: [ui, button-bar, context-menu, force-flag, test]
---

Removed the redundant in-body "TimeTracker" header label from `src/ui.rs` (title bar already shows it). Replaced the right-click context menu (`setup_context_menu`, `show_context_menu`) with four new buttons in the button bar — Edit Time, Delete, Mark, Move 5 min — each with correct sensitivity rules driven by a new `refresh_button_sensitivity` helper and a `selected_index: Rc<RefCell<Option<usize>>>` tracker updated via `connect_row_selected`. Verified `--force`/`-f` flag in `src/main.rs` already calls `fs::remove_file(lock_file_path())` before `acquire_lock()`; added unit test `test_force_flag_removes_stale_lock` in `src/data.rs` to cover this path. Version bumped from 0.2.0 to 0.3.0.
