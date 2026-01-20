//! Application state management for the Praymodoro timer.
//!
//! This module defines the core state structures including timer mode,
//! character selection, window positioning, and user preferences.

use crate::settings::Settings;

/// List of available saint characters for the desktop companion.
///
/// Each character has corresponding sprite assets in the `assets/characters/` directory
/// with idle, work, and quick-break animations.
pub const AVAILABLE_CHARACTERS: &[&str] = &[
    "augustine-of-hippo",
    "thomas-aquinas",
    "saint-patrick",
    "thomas-more",
];

/// Represents the current mode of the Pomodoro timer.
///
/// The timer alternates between [`Work`] sessions for focused productivity
/// and [`Rest`] sessions for prayer and reflection.
///
/// # Timer Schedule
///
/// Each hour follows a 30/5/25/5 pattern:
/// - 0-25 minutes: Work
/// - 25-30 minutes: Rest (prayer)
/// - 30-55 minutes: Work
/// - 55-60 minutes: Rest (prayer)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PomodoroMode {
    /// Work mode - time for focused productivity.
    Work,
    /// Rest mode - time for prayer and reflection.
    Rest,
}

impl PomodoroMode {
    /// Returns the string representation of the mode.
    ///
    /// Used for asset loading and display purposes.
    pub fn as_str(&self) -> &'static str {
        match self {
            PomodoroMode::Work => "work",
            PomodoroMode::Rest => "rest",
        }
    }
}

/// The main application state shared between threads.
///
/// This state is wrapped in `Arc<Mutex<_>>` to allow safe concurrent access
/// between the UI thread, timer thread, and tray icon handler.
#[derive(Clone, Debug)]
pub struct AppState {
    /// Current timer mode (Work or Rest).
    pub mode: PomodoroMode,
    /// Remaining seconds in the current period.
    pub remaining_seconds: i32,
    /// Pre-formatted time string (MM:SS) for display.
    pub formatted_time: String,
    /// Currently selected saint character identifier.
    pub character: String,
    /// Window scale factor (0.5 to 2.0).
    pub scale: f32,
    /// Whether the companion window is visible.
    pub visible: bool,
    /// User settings persisted to disk.
    pub settings: Settings,
    /// Signal flag to quit the application.
    pub should_quit: bool,
    /// Last known window position (x, y) in screen coordinates.
    pub window_position: Option<(f32, f32)>,
}

impl AppState {
    /// Creates a new application state with default values.
    ///
    /// Initializes with:
    /// - Work mode
    /// - 25 minutes remaining
    /// - Augustine of Hippo as the default character
    /// - 100% scale (1.0)
    /// - Window visible
    pub fn new() -> Self {
        Self {
            mode: PomodoroMode::Work,
            remaining_seconds: 25 * 60,
            formatted_time: "25:00".to_string(),
            character: "augustine-of-hippo".to_string(),
            scale: 1.0,
            visible: true,
            settings: Settings::default(),
            should_quit: false,
            window_position: None,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
