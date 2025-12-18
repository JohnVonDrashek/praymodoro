import { contextBridge, ipcRenderer } from 'electron';
import { IPC_CHANNELS, MenuState, MenuAction, PomodoroMode } from '../shared/types';

contextBridge.exposeInMainWorld('menuApi', {
  getState: (): Promise<MenuState> => {
    return ipcRenderer.invoke(IPC_CHANNELS.MENU_GET_STATE);
  },

  sendAction: (action: MenuAction): void => {
    ipcRenderer.send(IPC_CHANNELS.MENU_ACTION, action);
  },

  close: (): void => {
    ipcRenderer.send(IPC_CHANNELS.MENU_CLOSE);
  },

  onTimeUpdate: (callback: (countdown: string) => void): void => {
    ipcRenderer.on(IPC_CHANNELS.MENU_TIME_UPDATE, (_event, countdown) => callback(countdown));
  },

  onModeUpdate: (callback: (mode: PomodoroMode) => void): void => {
    ipcRenderer.on(IPC_CHANNELS.MENU_MODE_UPDATE, (_event, mode) => callback(mode));
  },
});

declare global {
  interface Window {
    menuApi: {
      getState: () => Promise<MenuState>;
      sendAction: (action: MenuAction) => void;
      close: () => void;
      onTimeUpdate: (callback: (countdown: string) => void) => void;
      onModeUpdate: (callback: (mode: PomodoroMode) => void) => void;
    };
  }
}
