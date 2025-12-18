import { Tray, Menu, nativeImage, app } from 'electron';
import { WindowManager } from './window-manager';
import { SettingsManager } from './settings-manager';
import { CharacterName } from '../shared/types';
import { AVAILABLE_CHARACTERS } from '../shared/constants';

export class TrayManager {
  private tray: Tray | null = null;
  private currentCountdown: string = '25:00';

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

    // Initial menu
    this.updateContextMenu();
  }

  /**
   * Update countdown display
   */
  updateCountdown(countdown: string): void {
    this.currentCountdown = countdown;
    if (this.tray) {
      this.tray.setToolTip(`Praymodoro - ${countdown}`);
      // Update menu to reflect new countdown
      this.updateContextMenu();
    }
  }

  /**
   * Update context menu
   */
  private updateContextMenu(): void {
    if (!this.tray) return;

    const isVisible = this.windowManager.isVisible();
    const currentScale = this.windowManager.getScale();
    const scalePercent = Math.round(currentScale * 100);

    const menu = Menu.buildFromTemplate([
      {
        label: `Time Remaining: ${this.currentCountdown}`,
        enabled: false,
      },
      { type: 'separator' },
      {
        label: isVisible ? 'Hide Character' : 'Show Character',
        click: () => {
          this.windowManager.toggleWindow();
          // Update menu after toggle
          setTimeout(() => this.updateContextMenu(), 100);
        },
      },
      { type: 'separator' },
      {
        label: `Size: ${scalePercent}%`,
        enabled: false,
      },
      {
        label: 'Increase Size',
        click: () => {
          const newScale = currentScale + 0.1;
          this.windowManager.setScale(newScale);
          this.settingsManager.saveScale(newScale);
          this.updateContextMenu();
          // Reopen menu so user can continue resizing
          if (this.tray) {
            setTimeout(() => this.tray?.popUpContextMenu(), 50);
          }
        },
      },
      {
        label: 'Decrease Size',
        click: () => {
          const newScale = currentScale - 0.1;
          this.windowManager.setScale(newScale);
          this.settingsManager.saveScale(newScale);
          this.updateContextMenu();
          // Reopen menu so user can continue resizing
          if (this.tray) {
            setTimeout(() => this.tray?.popUpContextMenu(), 50);
          }
        },
      },
      { type: 'separator' },
      {
        label: `Character: ${this.formatCharacterName(this.settingsManager.getCharacter())}`,
        enabled: false,
      },
      {
        label: 'Next Character',
        click: () => {
          this.cycleCharacter();
          this.updateContextMenu();
          if (this.tray) {
            setTimeout(() => this.tray?.popUpContextMenu(), 50);
          }
        },
      },
      { type: 'separator' },
      {
        label: 'Quit',
        click: () => {
          app.quit();
        },
      },
    ]);

    this.tray.setContextMenu(menu);
  }

  /**
   * Show context menu
   */
  private showContextMenu(): void {
    this.updateContextMenu();
  }

  /**
   * Format character name for display
   */
  private formatCharacterName(name: CharacterName): string {
    return name
      .split('-')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }

  /**
   * Cycle to next character
   */
  private cycleCharacter(): void {
    const currentCharacter = this.settingsManager.getCharacter();
    const currentIndex = AVAILABLE_CHARACTERS.indexOf(currentCharacter);
    const nextIndex = (currentIndex + 1) % AVAILABLE_CHARACTERS.length;
    const nextCharacter = AVAILABLE_CHARACTERS[nextIndex];

    this.settingsManager.saveCharacter(nextCharacter);
    this.windowManager.updateCharacter(nextCharacter);
  }

  /**
   * Destroy tray
   */
  destroy(): void {
    if (this.tray) {
      this.tray.destroy();
      this.tray = null;
    }
  }
}
