#![forbid(unsafe_code)]

mod app;
mod config;
mod data;
mod sort;
mod ui;

use gtk4::prelude::*;
use gtk4::{glib, Application};

const APP_ID: &str = "io.github.zerblatt007.titrax-rs";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        "titrax-rs {} — TimeTracker\n\
         \n\
         USAGE:\n\
         \ttitrax-rs [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \t--help,    -h   Print this help message and exit\n\
         \t--version, -v   Print version and exit\n\
         \n\
         DATA:\n\
         \tDay files and project list are stored in ~/.TimeTracker/\n\
         \tConfiguration is stored in ~/.config/titrax-rs/config.toml\n",
        VERSION
    );
}

fn main() -> glib::ExitCode {
    // Parse arguments before acquiring the lock or initializing GTK.
    // This prevents --help and --version from creating a LOCK file.
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return glib::ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("titrax-rs {}", VERSION);
        return glib::ExitCode::SUCCESS;
    }

    let gtk_args: Vec<String> = args.clone();

    // Acquire the PID lock file. Exit immediately if another instance holds it.
    // Stale locks (from crashed processes) are detected and removed automatically
    // by checking whether the stored PID is still alive.
    let _lock = match data::acquire_lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!(
                "titrax-rs: cannot acquire lock ({:?}): {}\n\
                 titrax-rs: another instance may already be running.",
                data::lock_file_path(),
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
    let exit_code = app.run_with_args(&gtk_args);

    // Belt-and-suspenders: explicitly remove the LOCK file after the main loop
    // exits, in case the Drop on LockGuard did not run (e.g. if GTK internals
    // called a non-unwinding exit path).
    let _ = data::remove_lock_file_if_exists();

    exit_code
}

// Numeric signal constants — avoids pulling in the `libc` crate.
const fn libc_sigterm() -> i32 {
    15
}
const fn libc_sigint() -> i32 {
    2
}
const fn libc_sighup() -> i32 {
    1
}
