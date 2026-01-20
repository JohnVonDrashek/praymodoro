//! Praymodoro - A Pomodoro timer with Catholic saints as desktop companions.
//!
//! This is the main entry point for the Praymodoro desktop application.
//! The application uses egui for the UI, runs a background timer thread,
//! and provides a system tray icon for control.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod settings;
mod state;
mod timer;
mod tray;

use app::PrayomodoroApp;
use parking_lot::Mutex;
use state::AppState;
use std::sync::Arc;

/// Hides the application from the macOS Dock.
///
/// This makes the app behave as a menu bar utility rather than a regular application.
/// The window remains functional, but there's no Dock icon.
#[cfg(target_os = "macos")]
fn hide_dock_icon() {
    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
    unsafe {
        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory);
    }
}

#[cfg(not(target_os = "macos"))]
fn hide_dock_icon() {}

/// Loads the application icon from embedded assets.
///
/// Returns icon data in RGBA format that egui can use for the window icon.
fn load_app_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../assets/icons/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load app icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    egui::IconData {
        rgba,
        width,
        height,
    }
}

/// Application entry point.
///
/// Initializes the application state, spawns the timer thread, and launches
/// the egui window with a transparent, draggable interface.
fn main() {
    // Initialize shared state
    let state = Arc::new(Mutex::new(AppState::new()));

    // Load settings
    {
        let mut s = state.lock();
        s.settings = settings::load_settings();
        s.character = s.settings.character.clone();
        s.scale = s.settings.window.scale;
    }

    // Start timer thread
    let state_for_timer = Arc::clone(&state);
    std::thread::spawn(move || {
        timer::run_timer(state_for_timer);
    });

    // Load app icon
    let icon = load_app_icon();

    // Run the egui app (tray will be created inside the app on the main thread)
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([160.0, 395.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_has_shadow(false) // Prevents ghosting on macOS transparent windows
            .with_always_on_top()
            .with_resizable(false)
            .with_title("Praymodoro")
            .with_icon(icon),
        ..Default::default()
    };

    let state_for_app = Arc::clone(&state);
    eframe::run_native(
        "Praymodoro",
        native_options,
        Box::new(move |cc| {
            // Hide dock icon on macOS (must be after eframe init)
            hide_dock_icon();

            // Install image loaders for egui_extras
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // Load custom serif font for timer
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "serif".to_owned(),
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/NotoSerif-Bold.ttf"
                ))),
            );
            // Add serif as a new font family
            fonts
                .families
                .insert(egui::FontFamily::Name("serif".into()), vec!["serif".to_owned()]);
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(PrayomodoroApp::new(state_for_app)))
        }),
    )
    .expect("Failed to run eframe");
}
