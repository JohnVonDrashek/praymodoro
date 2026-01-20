//! User settings persistence using JSON storage.
//!
//! Settings are automatically saved to the platform-specific configuration directory:
//! - macOS: `~/Library/Application Support/com.praymodoro.Praymodoro/settings.json`
//! - Linux: `~/.config/praymodoro/settings.json`
//! - Windows: `%APPDATA%\praymodoro\Praymodoro\settings.json`

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Window positioning and scale settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowSettings {
    /// Window X position on screen.
    pub x: f32,
    /// Window Y position on screen.
    pub y: f32,
    /// Window scale factor (0.5 = 50%, 1.0 = 100%, 2.0 = 200%).
    pub scale: f32,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            scale: 1.0,
        }
    }
}

/// User preferences persisted between application sessions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Window positioning and scale preferences.
    pub window: WindowSettings,
    /// Selected saint character identifier.
    pub character: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window: WindowSettings::default(),
            character: "augustine-of-hippo".to_string(),
        }
    }
}

/// Returns the path to the settings file.
///
/// Uses the `directories` crate to determine the platform-specific config directory.
/// Returns `None` if the config directory cannot be determined.
fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "praymodoro", "Praymodoro").map(|dirs| {
        let config_dir = dirs.config_dir();
        config_dir.join("settings.json")
    })
}

/// Loads settings from disk, or returns defaults if the file doesn't exist.
///
/// This function silently handles errors (file not found, invalid JSON, etc.)
/// by returning default settings.
pub fn load_settings() -> Settings {
    if let Some(path) = settings_path() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&contents) {
                return settings;
            }
        }
    }
    Settings::default()
}

/// Saves settings to disk.
///
/// Creates the config directory if it doesn't exist. Errors are silently ignored
/// to avoid disrupting the application if settings cannot be saved.
pub fn save_settings(settings: &Settings) {
    if let Some(path) = settings_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = fs::write(&path, json);
        }
    }
}
