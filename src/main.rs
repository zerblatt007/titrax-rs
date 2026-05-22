#![forbid(unsafe_code)]

mod app;
mod config;
mod data;
mod sort;
mod ui;

use gtk4::prelude::*;
use gtk4::{glib, Application};

const APP_ID: &str = "no.uninett.titrax";

// UNIX signal numbers (POSIX)
const SIGTERM: i32 = 15;
const SIGINT: i32 = 2;
const SIGHUP: i32 = 1;
const SIGQUIT: i32 = 3;

fn main() -> glib::ExitCode {
    // Acquire the lock file before building the application.
    // If another instance holds the lock, exit immediately.
    let _lock = match data::acquire_lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!(
                "titrax: cannot acquire lock file (~/.TimeTracker/LOCK): {}",
                e
            );
            eprintln!("titrax: another instance may already be running.");
            return glib::ExitCode::FAILURE;
        }
    };

    let app = Application::builder().application_id(APP_ID).build();

    // Register UNIX signal handlers so the app saves and exits cleanly
    // on SIGTERM, SIGINT, SIGHUP, and SIGQUIT.
    {
        let app_clone = app.clone();
        glib::unix_signal_add_local(SIGTERM, move || {
            app_clone.quit();
            glib::ControlFlow::Break
        });
    }
    {
        let app_clone = app.clone();
        glib::unix_signal_add_local(SIGINT, move || {
            app_clone.quit();
            glib::ControlFlow::Break
        });
    }
    {
        let app_clone = app.clone();
        glib::unix_signal_add_local(SIGHUP, move || {
            app_clone.quit();
            glib::ControlFlow::Break
        });
    }
    {
        let app_clone = app.clone();
        glib::unix_signal_add_local(SIGQUIT, move || {
            app_clone.quit();
            glib::ControlFlow::Break
        });
    }

    app.connect_activate(ui::build_window);
    app.run()
}
