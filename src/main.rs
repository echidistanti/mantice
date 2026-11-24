#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! Mantice: a text expander written in Rust, inspired by Beeftext.
//!
//! Architecture:
//! * `platform`: keyboard/mouse monitoring, keystroke synthesis, clipboard, active application (per OS).
//! * `engine`: a thread that tracks the typed text, detects keywords and renders snippets.
//! * `ui`: the egui user interface, the tray icon, the combo picker and the input dialog.
//! * `library`, `variables`, `fragments`, `qtdate`: the platform-independent core (Beeftext-compatible).

// first, so that the `tr!` macro is visible in the other modules
#[macro_use]
mod i18n;
mod app_state;
mod backup;
mod emoji;
mod engine;
mod espanso;
mod espanso_source;
mod fragments;
mod icon;
mod library;
mod logger;
mod paths;
mod platform;
mod prefs;
mod process_list;
mod qtdate;
mod shortcut;
mod sound;
mod ui;
mod variables;

use std::sync::Arc;

use app_state::{AppShared, UiMsg};
use engine::{Engine, EngineMsg};
use paths::AppPaths;
use prefs::Preferences;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let start_hidden = args.iter().any(|a| a == "--background" || a == "--minimized");

    let paths = AppPaths::detect();
    // before anything reads the data folder; the log is not open yet, so the outcome is written below
    let legacy_migration = paths.migrate_legacy_data_dir();
    paths.ensure_exists();
    let prefs = Preferences::load(&paths.settings_file());
    logger::init(&paths.log_file(), prefs.write_debug_log);
    i18n::set_language(prefs.language);
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panic: {info}");
    }));
    match legacy_migration {
        Some(Ok(old)) => log::info!("The data of ExpanderCraft was moved from '{}' to '{}'.", old.display(), paths.data_dir.display()),
        Some(Err(e)) => log::error!("The data of ExpanderCraft could not be moved to '{}': {e}", paths.data_dir.display()),
        None => {}
    }
    log::info!(
        "Mantice {} starting ({}, data folder: {}{})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        paths.data_dir.display(),
        if paths.portable { ", portable edition" } else { "" }
    );

    let (engine_tx, engine_rx) = crossbeam_channel::unbounded();
    let (shared, ui_rx) = AppShared::new(paths.clone(), prefs, engine_tx);
    let shared = Arc::new(shared);

    let s = shared.clone();
    if !platform::acquire_single_instance(&paths.instance_socket(), move || s.ui.send(UiMsg::ShowMainWindow)) {
        log::info!("Mantice is already running: the existing instance was asked to show its window.");
        return;
    }

    shared.load_library();
    if shared.prefs.read().auto_start_at_login && !paths.portable {
        // keep the startup entry pointing to the current executable
        if let Err(e) = platform::set_autostart(true) {
            log::warn!("Could not update the automatic startup entry: {e:#}");
        }
    }

    // reload the combo list when another computer sharing its folder (OneDrive, Dropbox...) changes it
    let s = shared.clone();
    let _ = std::thread::Builder::new().name("mantice-library-watch".into()).spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            s.reload_library_if_changed();
        }
    });

    let engine = Engine::new(shared.clone(), engine_rx, platform::new_keyboard(), platform::new_clipboard()).spawn();
    let sink: platform::SharedSink = shared.clone();
    if let Err(e) = platform::start_input_monitor(sink) {
        log::error!("Keyboard monitoring could not be started: {e:#}");
    }

    if let Err(e) = ui::run(shared.clone(), ui_rx, start_hidden) {
        log::error!("The user interface failed: {e}");
    }

    let _ = shared.engine_tx.send(EngineMsg::Quit);
    let _ = engine.join();
    log::info!("Mantice stopped.");
    log::logger().flush();
    std::process::exit(0);
}
