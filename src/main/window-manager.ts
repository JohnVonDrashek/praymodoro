import { BrowserWindow, screen } from 'electron';
import { PomodoroState, PomodoroMode, CharacterName, IPC_CHANNELS } from '../shared/types';
import { WINDOW_WIDTH, WINDOW_HEIGHT } from '../shared/constants';

export class WindowManager {
  private window: BrowserWindow | null = null;
  private currentMode: PomodoroMode = 'work';

  constructor(
    private preloadPath: string,
    private rendererPath: string,
    private initialX: number,
    private initialY: number,
    private scale: number = 1.0
  ) {}

  /**
   * Create the character window
   */
  createWindow(): void {
    const scaledWidth = Math.floor(WINDOW_WIDTH * this.scale);
    const scaledHeight = Math.floor(WINDOW_HEIGHT * this.scale);

    this.window = new BrowserWindow({
      width: scaledWidth,
      height: scaledHeight,
      x: this.initialX,
      y: this.initialY,
      transparent: true,
      frame: false,
      alwaysOnTop: true,
      resizable: false,
      hasShadow: false,
      skipTaskbar: true, // Don't show in task switcher
      webPreferences: {
        preload: this.preloadPath,
        nodeIntegration: false,
        contextIsolation: true,
      },
    });

    // Make window draggable
    this.window.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });
    this.window.setAlwaysOnTop(true, 'floating');

    // Load the renderer HTML (use loadURL for Webpack, loadFile for file paths)
    if (this.rendererPath.startsWith('http://') || this.rendererPath.startsWith('file://')) {
      this.window.loadURL(this.rendererPath);
    } else {
      this.window.loadFile(this.rendererPath);
    }

    // Handle window moved (for saving position)
    this.window.on('moved', () => {
      // Position is saved via IPC from renderer
    });

    // Cleanup on close
    this.window.on('closed', () => {
      this.window = null;
    });
  }

  /**
   * Show the window
   */
  showWindow(): void {
    if (this.window) {
      this.window.show();
    }
  }

  /**
   * Hide the window
   */
  hideWindow(): void {
    if (this.window) {
      this.window.hide();
    }
  }

  /**
   * Toggle window visibility
   */
  toggleWindow(): void {
    if (this.window) {
      if (this.window.isVisible()) {
        this.hideWindow();
      } else {
        this.showWindow();
      }
    }
  }

  /**
   * Check if window is visible
   */
  isVisible(): boolean {
    return this.window?.isVisible() ?? false;
  }

  /**
   * Update character mode (work/rest)
   */
  updateCharacterMode(mode: PomodoroMode): void {
    this.currentMode = mode;
    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.send(IPC_CHANNELS.PERIOD_CHANGE, mode);
    }
  }

  /**
   * Update character (switch between characters)
   */
  updateCharacter(character: CharacterName): void {
    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.send(IPC_CHANNELS.CHARACTER_CHANGE, character);
    }
  }

  /**
   * Send time update to renderer
   */
  sendTimeUpdate(state: PomodoroState, formattedTime: string): void {
    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.send(IPC_CHANNELS.TIME_UPDATE, {
        ...state,
        formattedTime,
      });
    }
  }

  /**
   * Set window position
   */
  setPosition(x: number, y: number): void {
    if (this.window) {
      // Ensure window stays on screen
      const bounds = this.ensureOnScreen(x, y);
      this.window.setPosition(bounds.x, bounds.y);
    }
  }

  /**
   * Get current window position
   */
  getPosition(): { x: number; y: number } {
    if (this.window) {
      const [x, y] = this.window.getPosition();
      return { x, y };
    }
    return { x: this.initialX, y: this.initialY };
  }

  /**
   * Ensure window position is on screen
   */
  private ensureOnScreen(x: number, y: number): { x: number; y: number } {
    const displays = screen.getAllDisplays();
    const scaledWidth = Math.floor(WINDOW_WIDTH * this.scale);
    const scaledHeight = Math.floor(WINDOW_HEIGHT * this.scale);

    // Find if position is within any display
    let onScreen = false;
    for (const display of displays) {
      const { x: dx, y: dy, width: dw, height: dh } = display.bounds;
      if (
        x >= dx &&
        x + scaledWidth <= dx + dw &&
        y >= dy &&
        y + scaledHeight <= dy + dh
      ) {
        onScreen = true;
        break;
      }
    }

    // If off-screen, place on primary display
    if (!onScreen) {
      const primaryDisplay = screen.getPrimaryDisplay();
      const { x: dx, y: dy } = primaryDisplay.bounds;
      return { x: dx + 100, y: dy + 100 };
    }

    return { x, y };
  }

  /**
   * Set window scale
   */
  setScale(scale: number): void {
    this.scale = Math.max(0.5, Math.min(3.0, scale)); // Clamp between 0.5x and 3.0x

    if (this.window && !this.window.isDestroyed()) {
      const scaledWidth = Math.floor(WINDOW_WIDTH * this.scale);
      const scaledHeight = Math.floor(WINDOW_HEIGHT * this.scale);

      // Resize window (keeps top-left corner in same position)
      this.window.setSize(scaledWidth, scaledHeight);
    }
  }

  /**
   * Get current scale
   */
  getScale(): number {
    return this.scale;
  }

  /**
   * Destroy the window
   */
  destroy(): void {
    if (this.window && !this.window.isDestroyed()) {
      this.window.close();
      this.window = null;
    }
  }
}
