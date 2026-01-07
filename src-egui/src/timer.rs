use crate::state::{AppState, PomodoroMode};
use chrono::{Local, Timelike};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

struct PomodoroSegment {
    start_minute: u32,
    end_minute: u32,
    mode: PomodoroMode,
}

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

fn format_time(seconds: i32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}

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
