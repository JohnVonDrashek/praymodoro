//! System tray icon management with context menu.
//!
//! Provides a system tray icon that allows users to:
//! - View the countdown timer
//! - Toggle character visibility
//! - Change character size (50% to 200%)
//! - Switch between saint characters
//! - Quit the application

use crate::state::{AppState, PomodoroMode, AVAILABLE_CHARACTERS};
use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use parking_lot::Mutex;
use std::sync::Arc;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Actions that can be triggered from the tray menu.
#[derive(Clone, Debug)]
pub enum TrayAction {
    /// No action.
    None,
    /// Toggle the visibility of the character window.
    ToggleVisibility,
    /// Change the selected saint character.
    SetCharacter(String),
    /// Change the window scale (0.5 to 2.0).
    SetScale(f32),
    /// Quit the application.
    Quit,
}

/// Manages the system tray icon and its context menu.
///
/// The tray icon displays a tomato icon and provides a context menu for
/// controlling the application. Menu items are automatically updated to
/// reflect the current application state.
pub struct TrayManager {
    _tray: TrayIcon,
    /// Menu item showing the countdown timer.
    countdown_item: MenuItem,
    /// Checkbox to show/hide the character window.
    show_check: CheckMenuItem,
    /// Size option checkboxes (50%, 75%, 100%, 125%, 150%, 200%).
    size_checks: Vec<(f32, CheckMenuItem)>,
    /// Character selection checkboxes.
    char_checks: Vec<(String, CheckMenuItem)>,
    /// Menu ID for the quit action.
    quit_id: muda::MenuId,
}

impl TrayManager {
    /// Creates a new tray icon with context menu.
    ///
    /// The menu is constructed with:
    /// - Countdown display (updates automatically)
    /// - Size submenu with percentage options
    /// - Character submenu with available saints
    /// - Show/hide checkbox
    /// - Quit option
    pub fn new() -> Self {
        // Create menu items
        let countdown_item = MenuItem::new("Work for: 25:00", false, None);
        let show_check = CheckMenuItem::new("Show Character", true, true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let quit_id = quit_item.id().clone();

        // Size submenu with check items
        let size_submenu = Submenu::new("Size", true);
        let sizes: Vec<f32> = vec![0.5, 0.75, 1.0, 1.25, 1.5, 2.0];
        let mut size_checks = Vec::new();
        for size in &sizes {
            let label = format!("{}%", (size * 100.0) as i32);
            let check = CheckMenuItem::new(&label, true, *size == 1.0, None);
            let _ = size_submenu.append(&check);
            size_checks.push((*size, check));
        }

        // Character submenu with check items
        let char_submenu = Submenu::new("Character", true);
        let mut char_checks = Vec::new();
        for (i, char_name) in AVAILABLE_CHARACTERS.iter().enumerate() {
            let display_name = format_character_name(char_name);
            let check = CheckMenuItem::new(&display_name, true, i == 0, None);
            let _ = char_submenu.append(&check);
            char_checks.push((char_name.to_string(), check));
        }

        // Build menu
        let menu = Menu::new();
        let _ = menu.append(&countdown_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&size_submenu);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&char_submenu);
        let _ = menu.append(&show_check);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        // Load tray icon
        let icon = load_tray_icon();

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_tooltip("Praymodoro")
            .with_menu_on_left_click(true)
            .build()
            .expect("Failed to create tray icon");

        Self {
            _tray: tray,
            countdown_item,
            show_check,
            size_checks,
            char_checks,
            quit_id,
        }
    }

    /// Polls for tray menu events and updates menu state.
    ///
    /// Should be called frequently (typically in the main UI update loop).
    /// Returns any action that should be taken in response to menu interactions.
    ///
    /// # Arguments
    ///
    /// * `state` - Current application state for updating menu checkboxes
    pub fn poll_events(&mut self, state: &Arc<Mutex<AppState>>) -> TrayAction {
        // Update countdown label
        {
            let s = state.lock();
            let mode_label = if s.mode == PomodoroMode::Work {
                "Work for:"
            } else {
                "Pray for:"
            };
            let _ = self.countdown_item.set_text(format!("{} {}", mode_label, s.formatted_time));

            // Update show check to match state
            let _ = self.show_check.set_checked(s.visible);

            // Update size checks
            for (size, check) in &self.size_checks {
                let _ = check.set_checked((*size - s.scale).abs() < 0.01);
            }

            // Update character checks
            for (char_name, check) in &self.char_checks {
                let _ = check.set_checked(*char_name == s.character);
            }
        }

        // Check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            // Check if quit
            if event.id == self.quit_id {
                return TrayAction::Quit;
            }

            // Check if show toggle
            if event.id == *self.show_check.id() {
                return TrayAction::ToggleVisibility;
            }

            // Check size items
            for (size, check) in &self.size_checks {
                if event.id == *check.id() {
                    return TrayAction::SetScale(*size);
                }
            }

            // Check character items
            for (char_name, check) in &self.char_checks {
                if event.id == *check.id() {
                    return TrayAction::SetCharacter(char_name.clone());
                }
            }
        }

        TrayAction::None
    }
}

/// Loads the tray icon from embedded assets.
///
/// Uses the `tray-iconTemplate@2x.png` which follows macOS naming conventions
/// for template images (automatically adapts to dark/light mode).
fn load_tray_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/tray-iconTemplate@2x.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load tray icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}

/// Formats a character identifier into a human-readable display name.
///
/// Converts kebab-case identifiers to Title Case, filtering out common words like "of".
///
/// # Examples
///
/// ```
/// assert_eq!(format_character_name("augustine-of-hippo"), "Augustine Hippo");
/// assert_eq!(format_character_name("thomas-aquinas"), "Thomas Aquinas");
/// ```
fn format_character_name(name: &str) -> String {
    name.split('-')
        .filter(|s| *s != "of")
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
