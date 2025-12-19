use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_store::StoreExt;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

// Constants
const WINDOW_WIDTH: f64 = 200.0;
const WINDOW_HEIGHT: f64 = 450.0;
const AVAILABLE_CHARACTERS: &[&str] = &["augustine-of-hippo", "thomas-aquinas", "saint-patrick", "thomas-more"];

// Pomodoro segments aligned to hourly clock
struct PomodoroSegment {
    start_minute: u32,
    end_minute: u32,
    segment_type: PomodoroMode,
}

const POMODORO_SEGMENTS: &[PomodoroSegment] = &[
    PomodoroSegment { start_minute: 0, end_minute: 25, segment_type: PomodoroMode::Work },
    PomodoroSegment { start_minute: 25, end_minute: 30, segment_type: PomodoroMode::Rest },
    PomodoroSegment { start_minute: 30, end_minute: 55, segment_type: PomodoroMode::Work },
    PomodoroSegment { start_minute: 55, end_minute: 60, segment_type: PomodoroMode::Rest },
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PomodoroMode {
    Work,
    Rest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeUpdate {
    #[serde(rename = "type")]
    pub mode_type: PomodoroMode,
    pub remaining: i32,
    #[serde(rename = "formattedTime")]
    pub formatted_time: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowSettings {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub opacity: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub window: WindowSettings,
    pub character: String,
    pub launch_at_startup: bool,
    pub show_in_dock: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window: WindowSettings {
                x: 100.0,
                y: 100.0,
                scale: 1.0,
                opacity: 1.0,
            },
            character: "augustine-of-hippo".to_string(),
            launch_at_startup: false,
            show_in_dock: false,
        }
    }
}

// Menu state for frontend
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuState {
    pub countdown: String,
    pub mode: String,
    pub is_character_visible: bool,
    pub scale: f64,
    pub character: String,
}

// App state
pub struct AppState {
    last_mode: Mutex<Option<PomodoroMode>>,
    current_countdown: Mutex<String>,
    current_mode: Mutex<PomodoroMode>,
    settings: Mutex<Settings>,
    character_visible: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_mode: Mutex::new(None),
            current_countdown: Mutex::new("25:00".to_string()),
            current_mode: Mutex::new(PomodoroMode::Work),
            settings: Mutex::new(Settings::default()),
            character_visible: Mutex::new(true),
        }
    }
}

// Time calculation
fn get_current_period() -> (PomodoroMode, i32) {
    let now = Local::now();
    let minutes = now.minute();
    let seconds = now.second();

    let segment = POMODORO_SEGMENTS
        .iter()
        .find(|s| minutes >= s.start_minute && minutes < s.end_minute)
        .unwrap_or(&POMODORO_SEGMENTS[0]);

    let current_second = (minutes * 60 + seconds) as i32;
    let end_second = (segment.end_minute * 60) as i32;
    let remaining = end_second - current_second;

    (segment.segment_type, remaining)
}

fn format_time(seconds: i32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

// Tauri commands
#[tauri::command]
fn get_settings(state: tauri::State<Arc<AppState>>) -> Settings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_position(x: f64, y: f64, state: tauri::State<Arc<AppState>>, app: AppHandle) {
    let mut settings = state.settings.lock().unwrap();
    settings.window.x = x;
    settings.window.y = y;

    // Persist to store
    if let Ok(store) = app.store("settings.json") {
        let _ = store.set("window", serde_json::to_value(&settings.window).unwrap());
        let _ = store.save();
    }
}

#[tauri::command]
fn save_scale(scale: f64, state: tauri::State<Arc<AppState>>, app: AppHandle) {
    let mut settings = state.settings.lock().unwrap();
    settings.window.scale = scale.clamp(0.5, 3.0);

    if let Ok(store) = app.store("settings.json") {
        let _ = store.set("window", serde_json::to_value(&settings.window).unwrap());
        let _ = store.save();
    }
}

#[tauri::command]
fn save_character(character: String, state: tauri::State<Arc<AppState>>, app: AppHandle) {
    let mut settings = state.settings.lock().unwrap();
    settings.character = character.clone();

    if let Ok(store) = app.store("settings.json") {
        let _ = store.set("character", serde_json::json!(character));
        let _ = store.save();
    }
}

#[tauri::command]
fn hide_window(window: WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
fn show_window(window: WebviewWindow) {
    let _ = window.show();
}

#[tauri::command]
fn toggle_window(window: WebviewWindow) -> bool {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        false
    } else {
        let _ = window.show();
        true
    }
}

#[tauri::command]
fn get_menu_state(state: tauri::State<Arc<AppState>>) -> MenuState {
    let settings = state.settings.lock().unwrap();
    let countdown = state.current_countdown.lock().unwrap().clone();
    let mode = state.current_mode.lock().unwrap();
    let visible = *state.character_visible.lock().unwrap();

    MenuState {
        countdown,
        mode: match *mode {
            PomodoroMode::Work => "work".to_string(),
            PomodoroMode::Rest => "rest".to_string(),
        },
        is_character_visible: visible,
        scale: settings.window.scale,
        character: settings.character.clone(),
    }
}

#[tauri::command]
fn menu_action(action: String, state: tauri::State<Arc<AppState>>, app: AppHandle) {
    match action.as_str() {
        "toggle-character" => {
            let mut visible = state.character_visible.lock().unwrap();
            *visible = !*visible;
            let is_visible = *visible;
            drop(visible);

            if let Some(window) = app.get_webview_window("main") {
                if is_visible {
                    let _ = window.show();
                    // Refocus menu to prevent it from closing
                    if let Some(menu) = app.get_webview_window("menu") {
                        let _ = menu.set_focus();
                    }
                } else {
                    let _ = window.hide();
                }
            }
        }
        "increase-size" => {
            if let Some(window) = app.get_webview_window("main") {
                let mut settings = state.settings.lock().unwrap();
                settings.window.scale = (settings.window.scale + 0.1).min(3.0);
                let scale = settings.window.scale;
                drop(settings);

                let new_width = (WINDOW_WIDTH * scale) as u32;
                let new_height = (WINDOW_HEIGHT * scale) as u32;
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(new_width, new_height)));
                let _ = app.emit("scale-change", scale);

                if let Ok(store) = app.store("settings.json") {
                    let s = state.settings.lock().unwrap();
                    let _ = store.set("window", serde_json::to_value(&s.window).unwrap());
                    let _ = store.save();
                }
            }
        }
        "decrease-size" => {
            if let Some(window) = app.get_webview_window("main") {
                let mut settings = state.settings.lock().unwrap();
                settings.window.scale = (settings.window.scale - 0.1).max(1.0);
                let scale = settings.window.scale;
                drop(settings);

                let new_width = (WINDOW_WIDTH * scale) as u32;
                let new_height = (WINDOW_HEIGHT * scale) as u32;
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(new_width, new_height)));
                let _ = app.emit("scale-change", scale);

                if let Ok(store) = app.store("settings.json") {
                    let s = state.settings.lock().unwrap();
                    let _ = store.set("window", serde_json::to_value(&s.window).unwrap());
                    let _ = store.save();
                }
            }
        }
        "next-character" => {
            let mut settings = state.settings.lock().unwrap();
            let current_idx = AVAILABLE_CHARACTERS
                .iter()
                .position(|&c| c == settings.character)
                .unwrap_or(0);
            let next_idx = (current_idx + 1) % AVAILABLE_CHARACTERS.len();
            settings.character = AVAILABLE_CHARACTERS[next_idx].to_string();
            let character = settings.character.clone();
            drop(settings);

            let _ = app.emit("character-change", &character);

            if let Ok(store) = app.store("settings.json") {
                let _ = store.set("character", serde_json::json!(character));
                let _ = store.save();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

#[tauri::command]
fn close_menu(app: AppHandle) {
    if let Some(menu_window) = app.get_webview_window("menu") {
        let _ = menu_window.hide();
    }
}

fn load_settings(app: &AppHandle) -> Settings {
    let store = app.store("settings.json").ok();

    let mut settings = Settings::default();

    if let Some(store) = store {
        if let Some(window) = store.get("window") {
            if let Ok(w) = serde_json::from_value::<WindowSettings>(window.clone()) {
                settings.window = w;
            }
        }
        if let Some(character) = store.get("character") {
            if let Some(c) = character.as_str() {
                settings.character = c.to_string();
            }
        }
    }

    settings
}

fn show_menu_at_tray(app: &AppHandle, position: tauri::PhysicalPosition<f64>) {
    // Check if menu window exists, if not create it
    if let Some(menu_window) = app.get_webview_window("menu") {
        // Toggle visibility
        if menu_window.is_visible().unwrap_or(false) {
            let _ = menu_window.hide();
        } else {
            // Position menu below tray icon
            let _ = menu_window.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition::new(
                    (position.x - 125.0) as i32, // Center menu under icon
                    position.y as i32,
                )
            ));
            let _ = menu_window.show();
            let _ = menu_window.set_focus();
        }
    } else {
        // Create menu window
        let menu_window = WebviewWindowBuilder::new(
            app,
            "menu",
            WebviewUrl::App("menu.html".into()),
        )
        .title("Menu")
        .inner_size(250.0, 200.0)
        .position((position.x - 125.0) as f64, position.y as f64)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .build();

        if let Ok(window) = menu_window {
            // Apply native macOS vibrancy effect (menu material)
            #[cfg(target_os = "macos")]
            let _ = apply_vibrancy(&window, NSVisualEffectMaterial::Menu, None, Some(6.0));

            let _ = window.set_focus();

            // Hide menu when it loses focus
            let app_handle = app.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    if let Some(menu) = app_handle.get_webview_window("menu") {
                        let _ = menu.hide();
                    }
                }
            });
        }
    }
}

fn setup_tray(app: &AppHandle, state: Arc<AppState>) -> tauri::Result<()> {
    // Load tray icon template from embedded PNG
    let icon_bytes = include_bytes!("../icons/tray-iconTemplate.png");
    let icon = Image::from_bytes(icon_bytes)
        .unwrap_or_else(|_| app.default_window_icon().unwrap().clone());

    let _ = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event {
                show_menu_at_tray(_tray.app_handle(), position);
            }
        })
        .build(app)?;

    // Also emit time updates to menu window
    let app_for_timer = app.clone();
    let state_for_timer = state.clone();
    thread::spawn(move || {
        loop {
            // Emit to menu window if visible
            if let Some(menu) = app_for_timer.get_webview_window("menu") {
                if menu.is_visible().unwrap_or(false) {
                    let countdown = state_for_timer.current_countdown.lock().unwrap().clone();
                    let _ = menu.emit("menu-time-update", &countdown);
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    Ok(())
}

fn start_timer(app: AppHandle, state: Arc<AppState>) {
    thread::spawn(move || {
        loop {
            let (mode, remaining) = get_current_period();
            let formatted = format_time(remaining);

            // Update stored countdown and mode
            *state.current_countdown.lock().unwrap() = formatted.clone();
            *state.current_mode.lock().unwrap() = mode;

            // Check for period change
            let mut last_mode = state.last_mode.lock().unwrap();
            let mode_changed = last_mode.map(|m| m != mode).unwrap_or(false);
            *last_mode = Some(mode);
            drop(last_mode);

            // Emit time update
            let update = TimeUpdate {
                mode_type: mode,
                remaining,
                formatted_time: formatted,
            };
            let _ = app.emit("time-update", &update);

            // Emit period change if needed
            if mode_changed {
                let _ = app.emit("period-change", &mode);
                // Also emit to menu
                let mode_str = match mode {
                    PomodoroMode::Work => "work",
                    PomodoroMode::Rest => "rest",
                };
                let _ = app.emit("menu-mode-update", mode_str);
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::default());
    let state_clone = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(state)
        .setup(move |app| {
            // Load settings
            let settings = load_settings(app.handle());
            *state_clone.settings.lock().unwrap() = settings.clone();

            // Apply window position and scale
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition::new(settings.window.x as i32, settings.window.y as i32)
                ));

                let new_width = (WINDOW_WIDTH * settings.window.scale) as u32;
                let new_height = (WINDOW_HEIGHT * settings.window.scale) as u32;
                let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(new_width, new_height)));
            }

            // Hide dock icon (menu bar app)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Setup tray
            setup_tray(app.handle(), state_clone.clone())?;

            // Start timer
            start_timer(app.handle().clone(), state_clone.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_position,
            save_scale,
            save_character,
            hide_window,
            show_window,
            toggle_window,
            get_menu_state,
            menu_action,
            close_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
