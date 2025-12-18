import { BrowserWindow, ipcMain, app } from 'electron';
import { WindowManager } from './window-manager';
import { SettingsManager } from './settings-manager';
import { IPC_CHANNELS, MenuState, MenuAction } from '../shared/types';
import { AVAILABLE_CHARACTERS } from '../shared/constants';

// Webpack provides these constants
declare const MENU_WINDOW_WEBPACK_ENTRY: string;
declare const MENU_WINDOW_PRELOAD_WEBPACK_ENTRY: string;

const MENU_WIDTH = 220;
const MENU_HEIGHT = 188;

export class CustomMenuManager {
  private menuWindow: BrowserWindow | null = null;
  private currentCountdown = '25:00';

  constructor(
    private windowManager: WindowManager,
    private settingsManager: SettingsManager
  ) {
    this.setupIPC();
  }

  private setupIPC(): void {
    // Handle get state request
    ipcMain.handle(IPC_CHANNELS.MENU_GET_STATE, (): MenuState => {
      return {
        countdown: this.currentCountdown,
        isCharacterVisible: this.windowManager.isVisible(),
        scale: this.windowManager.getScale(),
        character: this.settingsManager.getCharacter(),
      };
    });

    // Handle menu actions
    ipcMain.on(IPC_CHANNELS.MENU_ACTION, (_event, action: MenuAction) => {
      this.handleAction(action);
    });

    // Handle close request
    ipcMain.on(IPC_CHANNELS.MENU_CLOSE, () => {
      this.hideMenu();
    });
  }

  private handleAction(action: MenuAction): void {
    switch (action.type) {
      case 'toggle-character':
        this.windowManager.toggleWindow();
        break;

      case 'increase-size': {
        const currentScale = this.windowManager.getScale();
        const newScale = currentScale + 0.1;
        this.windowManager.setScale(newScale);
        this.settingsManager.saveScale(newScale);
        break;
      }

      case 'decrease-size': {
        const currentScale = this.windowManager.getScale();
        const newScale = currentScale - 0.1;
        this.windowManager.setScale(newScale);
        this.settingsManager.saveScale(newScale);
        break;
      }

      case 'next-character': {
        const currentCharacter = this.settingsManager.getCharacter();
        const currentIndex = AVAILABLE_CHARACTERS.indexOf(currentCharacter);
        const nextIndex = (currentIndex + 1) % AVAILABLE_CHARACTERS.length;
        const nextCharacter = AVAILABLE_CHARACTERS[nextIndex];
        this.settingsManager.saveCharacter(nextCharacter);
        this.windowManager.updateCharacter(nextCharacter);
        break;
      }

      case 'quit':
        app.quit();
        break;
    }
  }

  updateCountdown(countdown: string): void {
    this.currentCountdown = countdown;
    // Send real-time update to menu if it's open
    if (this.menuWindow && !this.menuWindow.isDestroyed()) {
      this.menuWindow.webContents.send(IPC_CHANNELS.MENU_TIME_UPDATE, countdown);
    }
  }

  showMenu(trayBounds: Electron.Rectangle): void {
    if (this.menuWindow && !this.menuWindow.isDestroyed()) {
      this.menuWindow.focus();
      return;
    }

    // Calculate position - below tray icon, centered
    const x = Math.round(trayBounds.x + trayBounds.width / 2 - MENU_WIDTH / 2);
    const y = trayBounds.y + trayBounds.height + 4;

    this.menuWindow = new BrowserWindow({
      width: MENU_WIDTH,
      height: MENU_HEIGHT,
      x,
      y,
      frame: false,
      transparent: true,
      resizable: false,
      movable: false,
      minimizable: false,
      maximizable: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      show: false,
      vibrancy: 'menu',
      visualEffectState: 'active',
      webPreferences: {
        preload: MENU_WINDOW_PRELOAD_WEBPACK_ENTRY,
        nodeIntegration: false,
        contextIsolation: true,
      },
    });

    this.menuWindow.loadURL(MENU_WINDOW_WEBPACK_ENTRY);

    // Show when ready
    this.menuWindow.once('ready-to-show', () => {
      this.menuWindow?.show();
    });

    // Close when clicking outside
    this.menuWindow.on('blur', () => {
      this.hideMenu();
    });

    this.menuWindow.on('closed', () => {
      this.menuWindow = null;
    });
  }

  hideMenu(): void {
    if (this.menuWindow && !this.menuWindow.isDestroyed()) {
      this.menuWindow.close();
      this.menuWindow = null;
    }
  }

  isMenuVisible(): boolean {
    return this.menuWindow !== null && !this.menuWindow.isDestroyed() && this.menuWindow.isVisible();
  }

  destroy(): void {
    this.hideMenu();
    ipcMain.removeHandler(IPC_CHANNELS.MENU_GET_STATE);
    ipcMain.removeAllListeners(IPC_CHANNELS.MENU_ACTION);
    ipcMain.removeAllListeners(IPC_CHANNELS.MENU_CLOSE);
  }
}
