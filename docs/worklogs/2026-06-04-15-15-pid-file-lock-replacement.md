---
when: 2026-06-04T15:15:26Z
why: Replace fragile create_new(true) lock with PID file that auto-detects stale locks
tags: [lockfile, pid, single-instance, refactor]
model: huggingface-local/local-model
what: Replace LOCK file with PID-based lock, remove --force flag

---

Replaced the `create_new(true)` single-instance lock with a PID file approach (`~/.TimeTracker/LOCK` now contains the process PID). On startup, the stored PID is checked for liveness via `kill(pid, 0)` — dead processes are detected automatically and their stale locks removed, eliminating the need for the `--force` / `-f` flag. Added `libc = "0.2"` dependency for the `kill` syscall. Updated tests to verify PID-based locking and stale lock detection. Updated all documentation (BLUEPRINT.md, PROJECT_RULES.md, CONTEXT.md). Version bumped 0.4.0 → 0.5.0.

Files changed: `Cargo.toml`, `src/data.rs`, `src/main.rs`, `BLUEPRINT.md`, `docs/PROJECT_RULES.md`, `CONTEXT.md`
