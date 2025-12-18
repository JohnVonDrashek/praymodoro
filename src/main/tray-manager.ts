import { Tray, nativeImage } from 'electron';
import { WindowManager } from './window-manager';
import { SettingsManager } from './settings-manager';
import { CustomMenuManager } from './custom-menu-manager';
import { PomodoroMode } from '../shared/types';

export class TrayManager {
  private tray: Tray | null = null;
  private currentCountdown = '25:00';
  private currentMode: PomodoroMode = 'work';
  private customMenuManager: CustomMenuManager | null = null;

  constructor(
    private iconPath: string,
    private windowManager: WindowManager,
    private settingsManager: SettingsManager
  ) {}

  /**
   * Create the system tray icon
   */
  createTray(): void {
    // Load icon (use template image for dark mode support on macOS)
    let icon: Electron.NativeImage;
    try {
      icon = nativeImage.createFromPath(this.iconPath);
      // Only set template image if filename contains "Template"
      if (this.iconPath.includes('Template')) {
        icon.setTemplateImage(true); // Adapts to dark mode
      }
    } catch (error) {
      console.error('Failed to load tray icon:', error);
      // Fallback to a simple emoji if icon not found
      icon = nativeImage.createFromDataURL(
        'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=='
      );
    }

    this.tray = new Tray(icon);
    this.tray.setToolTip('Praymodoro');

    // Initialize custom menu manager
    this.customMenuManager = new CustomMenuManager(this.windowManager, this.settingsManager);

    // Show custom menu on click
    this.tray.on('click', (_event, bounds) => {
      if (this.customMenuManager?.isMenuVisible()) {
        this.customMenuManager.hideMenu();
      } else {
        this.customMenuManager?.showMenu(bounds);
      }
    });
  }

  /**
   * Update countdown display
   */
  updateCountdown(countdown: string, mode: PomodoroMode): void {
    this.currentCountdown = countdown;
    const modeChanged = this.currentMode !== mode;
    this.currentMode = mode;

    if (this.tray) {
      this.tray.setToolTip(`Praymodoro - ${countdown}`);
    }
    if (this.customMenuManager) {
      this.customMenuManager.updateCountdown(countdown);
      if (modeChanged) {
        this.customMenuManager.updateMode(mode);
      }
    }
  }

  /**
   * Destroy tray
   */
  destroy(): void {
    if (this.customMenuManager) {
      this.customMenuManager.destroy();
      this.customMenuManager = null;
    }
    if (this.tray) {
      this.tray.destroy();
      this.tray = null;
    }
  }
}
