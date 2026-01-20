use crate::settings::Settings;

pub const AVAILABLE_CHARACTERS: &[&str] = &[
    "augustine-of-hippo",
    "thomas-aquinas",
    "saint-patrick",
    "thomas-more",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PomodoroMode {
    Work,
    Rest,
}

impl PomodoroMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PomodoroMode::Work => "work",
            PomodoroMode::Rest => "rest",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub mode: PomodoroMode,
    pub remaining_seconds: i32,
    pub formatted_time: String,
    pub character: String,
    pub scale: f32,
    pub visible: bool,
    pub settings: Settings,
    pub should_quit: bool,
    pub window_position: Option<(f32, f32)>,
}

impl AppState {
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
