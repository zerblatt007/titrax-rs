# Titrax GTK4/Rust — Blueprint

## Overview

Full rewrite of the original 1996 Athena-widget C program (titrax) into a modern GTK4/Rust desktop application. The data format (`~/.TimeTracker/YYYY-MM-DD`) is preserved for backward compatibility. The project-list file (`~/.TimeTracker/projectlist`) is preserved verbatim.

---

## Toolchain Installation (Builder must execute first)

```
apt-get update
apt-get install -y rustc cargo libgtk-4-dev pkg-config build-essential
```

- Verify: `rustc --version` must be ≥ 1.70. If apt provides an older version, install via rustup instead:
  ```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
  ```
- Verify GTK4: `pkg-config --modversion gtk4`
  - If version < 4.12, use `gtk4 = "0.7"` in Cargo.toml instead of `"0.9"`.

---

## Project File Structure

```
/work/
  src/
    main.rs        # Entry point: GApplication init, signal handlers, CLI args
    app.rs         # AppState struct, core business logic, timer callbacks
    data.rs        # File I/O: dayfile read/write, projectlist read/write, lock file
    config.rs      # config.toml read/write (~/.TimeTracker/config.toml)
    ui.rs          # GTK4 widget construction, signal connections, context menu
    sort.rs        # Norwegian sort: Æ, Ø, Å collate after Z
  Cargo.toml
  Cargo.lock
  build.rs         # (optional) only if needed for resource compilation
  .gitignore
  README.md
  docs/
    worklogs/
  scripts/
    bump-version.sh
```

---

## Cargo.toml

```toml
[package]
name = "titrax"
version = "0.1.0"
edition = "2021"

[dependencies]
gtk4 = "0.9"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
```

> **GTK4 version note:** If system GTK4 < 4.12, the Builder MUST downgrade to `gtk4 = "0.7"` and remove any `v4_12` feature flags.

---

## Data Models

### `Project` (in `app.rs`)
| Field         | Type            | Description                        |
|---------------|-----------------|------------------------------------|
| `name`        | `String`        | Project display name               |
| `minutes`     | `u32`           | Minutes worked today               |
| `is_dead`     | `bool`          | From dayfile but not in projectlist|

### `AppState` (in `app.rs`)
| Field              | Type                    | Description                              |
|--------------------|-------------------------|------------------------------------------|
| `projects`         | `Vec<Project>`          | Ordered list of all projects             |
| `active_index`     | `Option<usize>`         | Currently selected/ticking project       |
| `marked_index`     | `Option<usize>`         | Marked project for Transfer              |
| `total_minutes`    | `u32`                   | Sum of all project minutes               |
| `adjusted_minutes` | `i32`                   | Delta from manual edits                  |
| `data_dir`         | `PathBuf`               | `~/.TimeTracker` or override             |
| `date_override`    | `Option<String>`        | CLI arg: edit a specific date's file     |
| `last_tick_epoch`  | `i64`                   | Unix epoch minutes at last tick          |
| `last_date`        | `u32`                   | Day-of-month at last tick (midnight detect) |
| `paused`           | `bool`                  | No active project                        |

### `Config` (in `config.rs`)
| Field              | Type            | Default              |
|--------------------|-----------------|----------------------|
| `start_paused`     | `bool`          | `false`              |
| `auto_pause`       | `Option<String>`| `None` (`"HH:MM"`)  |
| `data_dir`         | `Option<String>`| `None`               |
| `inc_plus`         | `u32`           | `10`                 |
| `inc_minus`        | `u32`           | `10`                 |

---

## Day-File Format (`~/.TimeTracker/YYYY-MM-DD`)

```
# TIMETRACKER log saved at <timestamp>
# <reason>
HH:MM ProjectName
HH:MM AnotherProject
```

- Lines beginning with `#` are comments and are skipped on read.
- Time is stored as `HH:MM` (hours:minutes, not zero-padded hours required but written with `%2d:%02d`).
- Blank lines are ignored.

## Project-List File (`~/.TimeTracker/projectlist`)

- One project name per line, plain text.
- Blank lines ignored.
- If missing or empty, created with a single entry `"empty"`.

## Lock File (`~/.TimeTracker/LOCK`)

- Created on startup (normal mode only, not date-edit mode).
- Contains: `<hostname> <pid>\n`
- Removed on clean exit.
- If present at startup: show error dialog and exit.

---

## Module Responsibilities

### `main.rs`
- Parse CLI args: optional positional arg = date string (editor mode).
- Resolve `data_dir` from env `TIMETRACKDIR` → `TIMEXDIR` → config → default `~/.TimeTracker`.
- Create `GApplication`, connect `activate` signal.
- Register UNIX signal handlers (SIGTERM, SIGINT, SIGHUP, SIGQUIT) via `glib::unix_signal_add` — on receipt, save and quit.
- Bootstrap `AppState`, call `ui::build_window`.

### `app.rs`
- `AppState::new()` — initialise state, load projectlist, load today's dayfile.
- `AppState::tick()` — called every 60 s; increment active project; detect midnight rollover (save yesterday, clear, reload).
- `AppState::save(reason: &str)` — delegate to `data::write_dayfile`.
- `AppState::add_project(name, after_index)` — insert into projectlist and state.
- `AppState::delete_project(index)` — remove from projectlist and state.
- `AppState::edit_time(index, minutes)` — set project minutes, recalculate totals.
- `AppState::transfer(from, to, minutes)` — move minutes between projects.
- `AppState::increment(index, delta)` — adjust minutes by delta, floor at 0.
- `AppState::recalculate_totals()` — recompute `total_minutes` and `adjusted_minutes`.
- `AppState::detect_auto_pause(hour, minute)` — compare against config `auto_pause`.

### `data.rs`
- `read_projectlist(dir) -> Vec<String>` — parse projectlist file.
- `write_projectlist(dir, names)` — overwrite projectlist file.
- `read_dayfile(path) -> Vec<(String, u32)>` — parse `HH:MM name` lines.
- `write_dayfile(path, projects, reason)` — write dayfile with header.
- `acquire_lock(dir) -> Result<LockGuard>` — create LOCK file; `LockGuard` removes it on drop.
- `ensure_data_dir(dir)` — create directory if absent (mode 0700).

### `config.rs`
- `Config::load(dir) -> Config` — read `config.toml`; fall back to defaults on missing/parse error.
- `Config::save(dir)` — write `config.toml`.

### `ui.rs`
- `build_window(app, state_rc)` — construct all GTK4 widgets, connect signals, return `ApplicationWindow`.
- Widget tree:
  ```
  ApplicationWindow
    Box (vertical)
      Box (horizontal, toolbar)
        Button: Quit
        Button: +10
        Button: -10
        Button: Pause  (hidden in editor mode)
        Button: Add
        Button: Edit
        Button: Save   (editor mode only)
        Button: Load   (editor mode only)
        Label: total time display  (e.g. " 3:45+0")
      ScrolledWindow
        ListView (projects: time | name columns)
  ```
- Context menu (right-click on list row): Edit time, Delete project, Mark, Transfer.
- `build_edit_dialog(parent, project, minutes) -> Dialog` — modal dialog with `Entry` for time and +/- buttons.
- `build_add_dialog(parent) -> Dialog` — modal dialog with `Entry` for project name.
- Timer: `glib::timeout_add_seconds(60, tick_closure)` — fires every 60 s.
- Auto-save timer: `glib::timeout_add_seconds(600, save_closure)` — fires every 10 min.

### `sort.rs`
- `norwegian_sort_key(s: &str) -> String` — maps Æ→ZA, Ø→ZB, Å→ZC (and lowercase equivalents) so standard lexicographic sort places them after Z.
- `sort_projects(projects: &mut Vec<Project>)` — sorts by `norwegian_sort_key(name)`.
- Note: sort is applied only when explicitly requested (Add/Delete), not on every tick.

---

## Signal & Timer Architecture

| Event                        | Handler location | Action                                      |
|------------------------------|------------------|---------------------------------------------|
| Row selected in ListView     | `ui.rs`          | Set `active_index`, update title bar        |
| Quit button / window close   | `ui.rs`          | Save, release lock, `app.quit()`            |
| +10 / -10 buttons            | `ui.rs`          | Call `AppState::increment`                  |
| Pause button                 | `ui.rs`          | Set `active_index = None`                   |
| Add button                   | `ui.rs`          | Show add dialog; on confirm call `add_project` |
| Edit (context menu / button) | `ui.rs`          | Show edit dialog; on confirm call `edit_time` |
| Delete (context menu)        | `ui.rs`          | Call `delete_project`                       |
| Mark (context menu)          | `ui.rs`          | Set `marked_index`                          |
| Transfer (context menu)      | `ui.rs`          | Call `AppState::transfer`                   |
| 60 s timer                   | `app.rs`         | `AppState::tick()`, refresh UI              |
| 600 s timer                  | `app.rs`         | `AppState::save("Periodic save")`           |
| SIGTERM / SIGINT / SIGHUP    | `main.rs`        | Save, release lock, exit                    |
| Midnight detected in tick    | `app.rs`         | Save yesterday (offset -1), clear, reload   |

---

## State Sharing Pattern

`AppState` is wrapped in `Rc<RefCell<AppState>>` and cloned into each GTK signal closure. All UI mutations go through `state.borrow_mut()`. No multi-threading is used; GTK4 main loop is single-threaded.

---

## Editor Mode (date-override CLI arg)

When a date string is passed as a CLI argument:
- Lock file is NOT created.
- Save/Load buttons are shown; Pause button is hidden.
- Timer callbacks are NOT registered.
- `data_dir/date_string` is used as the dayfile path directly.

---

## Cleanup Plan for Old C Files

The following C-era files are superseded and must be moved to an `## Archive` note in this document once the Rust build is confirmed working. They must NOT be deleted until QA signs off:

- `titrax.c`, `titrax.h`, `projectlist.c`, `patchlevel.h`
- `Makefile`, `Makefile.bak`, `Imakefile`
- `Titrax.ad`, `titrax.xbm`
- `udping.c`, `ad2c`, `titrat`, `sumtitra`, `sumtitra.man`, `titrax.man`
- `weekno.perl`, `.indent.pro`, `CONTRIBUTIONS`, `TODO`

They remain in the repo root untouched until the Builder's cleanup task.

---

## .gitignore Entries (Builder must ensure)

```
/target/
Cargo.lock   # omit this line if publishing as binary crate — keep lock for binaries
*.swp
.env
```

> For a binary application, `Cargo.lock` SHOULD be committed. Do not add it to `.gitignore`.

---

## Archive

_(empty — no prior architecture to archive)_
