//! Pomodoro timer logic synchronized with system clock.
//!
//! The timer follows a fixed hourly schedule (30/5/25/5 pattern) that aligns
//! with the system clock, ensuring consistency across application restarts.
//!
//! # Schedule
//!
//! Each hour is divided into four periods:
//! - **00:00-25:00** - Work (25 minutes)
//! - **25:00-30:00** - Rest/Prayer (5 minutes)
//! - **30:00-55:00** - Work (25 minutes)
//! - **55:00-60:00** - Rest/Prayer (5 minutes)

use crate::state::{AppState, PomodoroMode};
use chrono::{Local, Timelike};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

/// Represents a time period within the Pomodoro schedule.
struct PomodoroSegment {
    /// Start minute within the hour (0-59).
    start_minute: u32,
    /// End minute within the hour (0-60, where 60 = start of next hour).
    end_minute: u32,
    /// Mode for this time period (Work or Rest).
    mode: PomodoroMode,
}

/// The fixed hourly Pomodoro schedule.
///
/// These segments repeat every hour, synchronized with the system clock.
const POMODORO_SEGMENTS: &[PomodoroSegment] = &[
    PomodoroSegment {
        start_minute: 0,
        end_minute: 25,
        mode: PomodoroMode::Work,
    },
    PomodoroSegment {
        start_minute: 25,
        end_minute: 30,
        mode: PomodoroMode::Rest,
    },
    PomodoroSegment {
        start_minute: 30,
        end_minute: 55,
        mode: PomodoroMode::Work,
    },
    PomodoroSegment {
        start_minute: 55,
        end_minute: 60,
        mode: PomodoroMode::Rest,
    },
];

/// Determines the current Pomodoro period based on system time.
///
/// Returns the current mode (Work/Rest) and remaining seconds in that period.
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

    (segment.mode, remaining)
}

/// Formats seconds into MM:SS display format.
///
/// # Examples
///
/// ```
/// assert_eq!(format_time(90), "01:30");
/// assert_eq!(format_time(3661), "61:01");
/// ```
fn format_time(seconds: i32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Runs the timer loop in a background thread.
///
/// Updates the shared application state every second with the current mode
/// and remaining time. This function never returns and should be spawned
/// in a separate thread.
///
/// # Arguments
///
/// * `state` - Shared application state wrapped in `Arc<Mutex<_>>`
///
/// # Example
///
/// ```no_run
/// let state = Arc::new(Mutex::new(AppState::new()));
/// let state_for_timer = Arc::clone(&state);
/// std::thread::spawn(move || {
///     run_timer(state_for_timer);
/// });
/// ```
pub fn run_timer(state: Arc<Mutex<AppState>>) {
    loop {
        let (mode, remaining) = get_current_period();
        let formatted = format_time(remaining);

        {
            let mut s = state.lock();
            s.mode = mode;
            s.remaining_seconds = remaining;
            s.formatted_time = formatted;
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
