#![forbid(unsafe_code)]

mod app;
mod config;
mod data;
mod sort;
mod ui;

use gtk4::prelude::*;
use gtk4::{glib, Application};

const APP_ID: &str = "no.uninett.titrax";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        "titrax {} — TimeTracker\n\
         \n\
         USAGE:\n\
         \ttitrax [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \t--help,    -h   Print this help message and exit\n\
         \t--version, -v   Print version and exit\n\
         \t--force,   -f   Remove stale LOCK file before starting\n\
         \n\
         DATA:\n\
         \tDay files and project list are stored in ~/.TimeTracker/\n\
         \tConfiguration is stored in ~/.config/titrax/config.toml\n",
        VERSION
    );
}

fn main() -> glib::ExitCode {
    // Parse arguments before acquiring the lock or initializing GTK.
    // This prevents --help and --version from creating a LOCK file.
    let args: Vec<String> = std::env::args().collect();
    let force = args.iter().any(|a| a == "--force" || a == "-f");

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return glib::ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("titrax {}", VERSION);
        return glib::ExitCode::SUCCESS;
    }

    // Remove stale LOCK file if --force was passed.
    if force {
        let _ = std::fs::remove_file(data::lock_file_path());
    }

    // Acquire the lock file. Exit immediately if another instance holds it.
    let _lock = match data::acquire_lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!(
                "titrax: cannot acquire lock (~/.TimeTracker/LOCK): {}\n\
                 titrax: another instance may already be running.\n\
                 titrax: use --force to remove a stale lock.",
                e
            );
            return glib::ExitCode::FAILURE;
        }
    };

    let app = Application::builder().application_id(APP_ID).build();

    // Register UNIX signal handlers so the app saves and exits cleanly.
    // NOTE: GLib only supports SIGHUP, SIGINT, SIGTERM, SIGUSR1, SIGUSR2, SIGWINCH.
    // SIGQUIT is intentionally omitted — GLib will assert-fail if you register it.
    for signum in [libc_sigterm(), libc_sigint(), libc_sighup()] {
        let app_clone = app.clone();
        glib::unix_signal_add_local(signum, move || {
            app_clone.quit();
            glib::ControlFlow::Break
        });
    }

    app.connect_activate(ui::build_window);
    let exit_code = app.run();

    // Belt-and-suspenders: explicitly remove the LOCK file after the main loop
    // exits, in case the Drop on LockGuard did not run (e.g. if GTK internals
    // called a non-unwinding exit path).
    let _ = std::fs::remove_file(data::lock_file_path());

    exit_code
}

// Numeric signal constants — avoids pulling in the `libc` crate.
const fn libc_sigterm() -> i32 { 15 }
const fn libc_sigint()  -> i32 {  2 }
const fn libc_sighup()  -> i32 {  1 }
