# Project Rules — Titrax-RS GTK4/Rust

1. **State sharing:** All mutable app state lives in `AppState` wrapped as `Rc<RefCell<AppState>>`. Never introduce `Arc`, `Mutex`, or threads; GTK4 callbacks are single-threaded.

2. **GTK version guard:** Before building, verify `pkg-config --modversion gtk4`. Use crate `gtk4 = "0.9"` for GTK ≥ 4.12; downgrade to `"0.7"` for GTK ≥ 4.6. Document the chosen version in `README.md`.

3. **Data-file backward compatibility:** Day-file format (`# comment`, `HH:MM name`) and projectlist format (one name per line) must remain byte-for-byte compatible with the original C program's output.

4. **Lock-file RAII:** The lock file (`~/.TimeTracker/LOCK`) must be managed via a `LockGuard` struct that removes the file on `Drop`. Never leave the lock file behind on any exit path.

5. **No unsafe code:** `#![forbid(unsafe_code)]` must appear at the top of `main.rs`. All GTK4 and system interactions must go through safe Rust crate APIs only.
