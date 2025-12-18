import { ipcMain } from 'electron';
import { IPC_CHANNELS } from '../shared/types';
import { WindowManager } from './window-manager';
import { SettingsManager } from './settings-manager';

export function registerIPCHandlers(
  windowManager: WindowManager,
  settingsManager: SettingsManager
): void {
  // Hide window
  ipcMain.on(IPC_CHANNELS.HIDE_WINDOW, () => {
    windowManager.hideWindow();
  });

  // Save position
  ipcMain.on(IPC_CHANNELS.SAVE_POSITION, (event, x: number, y: number) => {
    settingsManager.savePosition(x, y);
  });

  // Get settings
  ipcMain.handle(IPC_CHANNELS.GET_SETTINGS, () => {
    return settingsManager.getSettings();
  });

  // Save settings
  ipcMain.on(IPC_CHANNELS.SAVE_SETTINGS, (event, settings) => {
    settingsManager.saveSettings(settings);
  });
}
