// Shared TypeScript types for the Praymodoro app

export type PomodoroMode = 'work' | 'rest';

export interface PomodoroSegment {
  startMinute: number;
  endMinute: number;
  type: PomodoroMode;
}

export interface PomodoroState {
  type: PomodoroMode;
  remaining: number; // seconds remaining in current period
}

export type CharacterName = 'augustine-of-hippo' | 'thomas-aquinas';

export interface Settings {
  window: {
    x: number;
    y: number;
    scale: number;    // 0.5 to 2.0
    opacity: number;  // 0.5 to 1.0
  };
  character: CharacterName;
  launchAtStartup: boolean;
  showInDock: boolean;
}

// IPC channel names
export const IPC_CHANNELS = {
  // Main -> Renderer
  TIME_UPDATE: 'time-update',
  PERIOD_CHANGE: 'period-change',
  CHARACTER_CHANGE: 'character-change',

  // Renderer -> Main
  HIDE_WINDOW: 'hide-window',
  SAVE_POSITION: 'save-position',
  GET_SETTINGS: 'get-settings',
  SAVE_SETTINGS: 'save-settings',
} as const;
