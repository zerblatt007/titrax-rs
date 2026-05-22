# Titrax — Project Context

## What It Is

A GTK4/Rust desktop time-tracker. The user maintains a list of projects; clicking one starts accumulating minutes. Data is saved daily to `~/.TimeTracker/YYYY-MM-DD`. This is a full rewrite of the original 1996 Athena-widget C program by Harald Tveit Alvestrand.

## Status

- **Phase:** Implementation complete (v0.x). Four UI/UX changes planned for next build cycle (see `BLUEPRINT.md § Planned Changes`).
- **Old C source** (`titrax.c`, `projectlist.c`, etc.) remains in repo root, untouched.
- **Rust source** lives under `src/`.

## Pending Changes Summary

1. Remove redundant "TimeTracker" label from window body (`ui.rs`).
2. Replace right-click context menu with a persistent button bar (`ui.rs`).
3. Add "Move 5 min" button: moves 5 min from active → selected project (`ui.rs`).
4. Verify `--force` / `-f` flag correctly removes stale LOCK before `acquire_lock()` (`main.rs` — likely already wired, needs test).

## Runtime Dependencies

| Dependency     | Version  | Purpose                          |
|----------------|----------|----------------------------------|
| `gtk4` (crate) | `0.9`    | GUI (requires GTK 4.12+ system lib) |
| `serde`        | `1`      | Config serialisation             |
| `toml`         | `0.8`    | Config file format               |
| `chrono`       | `0.4`    | Date/time handling               |

## System Build Dependencies (apt)

`rustc`, `cargo`, `libgtk-4-dev`, `pkg-config`, `build-essential`

> If apt `rustc` < 1.70, use rustup. If system GTK4 < 4.12, downgrade crate to `gtk4 = "0.7"`.

## Data Directory

`~/.TimeTracker/` (overridable via `TIMETRACKDIR` env var, `TIMEXDIR` env var, or `config.toml`)

## Key Files

| Path                          | Role                        |
|-------------------------------|-----------------------------|
| `src/main.rs`                 | Entry point, GApplication   |
| `src/app.rs`                  | AppState, business logic    |
| `src/data.rs`                 | File I/O, lock management   |
| `src/config.rs`               | Config load/save            |
| `src/ui.rs`                   | GTK4 widgets & signals      |
| `src/sort.rs`                 | Norwegian collation         |
| `~/.TimeTracker/projectlist`  | Project name list           |
| `~/.TimeTracker/YYYY-MM-DD`   | Daily time log              |
| `~/.TimeTracker/LOCK`         | Single-instance lock        |
| `~/.TimeTracker/config.toml`  | User config                 |

