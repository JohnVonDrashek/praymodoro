import { PomodoroSegment, CharacterName } from './types';

// Pomodoro time segments (aligned to hourly clock)
// :00-:25 = Work (25 min)
// :25-:30 = Rest (5 min)
// :30-:55 = Work (25 min)
// :55-:00 = Rest (5 min)
export const POMODORO_SEGMENTS: PomodoroSegment[] = [
  { startMinute: 0,  endMinute: 25, type: 'work' },
  { startMinute: 25, endMinute: 30, type: 'rest' },
  { startMinute: 30, endMinute: 55, type: 'work' },
  { startMinute: 55, endMinute: 60, type: 'rest' },
];

// Available characters
export const AVAILABLE_CHARACTERS: CharacterName[] = ['augustine-of-hippo', 'thomas-aquinas', 'saint-patrick'];

// Default settings
export const DEFAULT_SETTINGS = {
  window: {
    x: 100,
    y: 100,
    scale: 1.0,
    opacity: 1.0,
  },
  character: 'augustine-of-hippo' as CharacterName,
  launchAtStartup: false,
  showInDock: false,
};

// Window dimensions (base size, affected by scale setting)
// Aspect ratio matches trimmed character images (590:1455 ≈ 1:2.47)
export const WINDOW_WIDTH = 160;
export const WINDOW_HEIGHT = 395;
