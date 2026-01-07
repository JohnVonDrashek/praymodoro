use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowSettings {
    pub x: f32,
    pub y: f32,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub window: WindowSettings,
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

fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "praymodoro", "Praymodoro").map(|dirs| {
        let config_dir = dirs.config_dir();
        config_dir.join("settings.json")
    })
}

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
