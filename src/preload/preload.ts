import { contextBridge, ipcRenderer } from 'electron';
import { IPC_CHANNELS, PomodoroState, PomodoroMode, Settings, CharacterName } from '../shared/types';

// Expose protected methods via contextBridge
contextBridge.exposeInMainWorld('api', {
  // Listen for time updates from main process
  onTimeUpdate: (callback: (data: PomodoroState & { formattedTime: string }) => void) => {
    ipcRenderer.on(IPC_CHANNELS.TIME_UPDATE, (_event, data) => callback(data));
  },

  // Listen for period changes (work/rest transition)
  onPeriodChange: (callback: (mode: PomodoroMode) => void) => {
    ipcRenderer.on(IPC_CHANNELS.PERIOD_CHANGE, (_event, mode) => callback(mode));
  },

  // Listen for character changes
  onCharacterChange: (callback: (character: CharacterName) => void) => {
    ipcRenderer.on(IPC_CHANNELS.CHARACTER_CHANGE, (_event, character) => callback(character));
  },

  // Listen for scale changes
  onScaleChange: (callback: (scale: number) => void) => {
    ipcRenderer.on(IPC_CHANNELS.SCALE_CHANGE, (_event, scale) => callback(scale));
  },

  // Send hide window request
  hideWindow: () => {
    ipcRenderer.send(IPC_CHANNELS.HIDE_WINDOW);
  },

  // Save window position
  savePosition: (x: number, y: number) => {
    ipcRenderer.send(IPC_CHANNELS.SAVE_POSITION, x, y);
  },

  // Get settings
  getSettings: (): Promise<Settings> => {
    return ipcRenderer.invoke(IPC_CHANNELS.GET_SETTINGS);
  },

  // Save settings
  saveSettings: (settings: Partial<Settings>) => {
    ipcRenderer.send(IPC_CHANNELS.SAVE_SETTINGS, settings);
  },
});

// Type declaration for TypeScript
declare global {
  interface Window {
    api: {
      onTimeUpdate: (callback: (data: PomodoroState & { formattedTime: string }) => void) => void;
      onPeriodChange: (callback: (mode: PomodoroMode) => void) => void;
      onCharacterChange: (callback: (character: CharacterName) => void) => void;
      onScaleChange: (callback: (scale: number) => void) => void;
      hideWindow: () => void;
      savePosition: (x: number, y: number) => void;
      getSettings: () => Promise<Settings>;
      saveSettings: (settings: Partial<Settings>) => void;
    };
  }
}
