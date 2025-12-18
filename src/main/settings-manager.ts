import Store from 'electron-store';
import { Settings, CharacterName } from '../shared/types';
import { DEFAULT_SETTINGS } from '../shared/constants';

export class SettingsManager {
  private store: Store<Settings>;

  constructor() {
    this.store = new Store<Settings>({
      defaults: DEFAULT_SETTINGS,
      name: 'praymodoro-settings',
    });
  }

  /**
   * Get all settings
   */
  getSettings(): Settings {
    return this.store.store;
  }

  /**
   * Get a specific setting value
   */
  getSetting<K extends keyof Settings>(key: K): Settings[K] {
    return this.store.get(key);
  }

  /**
   * Save partial settings (deep merge)
   */
  saveSettings(settings: Partial<Settings>): void {
    // Deep merge for nested window settings
    if (settings.window) {
      const currentWindow = this.store.get('window');
      this.store.set('window', { ...currentWindow, ...settings.window });
    }

    // Set other top-level settings
    if (settings.character !== undefined) {
      this.store.set('character', settings.character);
    }
    if (settings.launchAtStartup !== undefined) {
      this.store.set('launchAtStartup', settings.launchAtStartup);
    }
    if (settings.showInDock !== undefined) {
      this.store.set('showInDock', settings.showInDock);
    }
  }

  /**
   * Get current character
   */
  getCharacter(): CharacterName {
    return this.store.get('character');
  }

  /**
   * Save character setting
   */
  saveCharacter(character: CharacterName): void {
    this.store.set('character', character);
  }

  /**
   * Save window position
   */
  savePosition(x: number, y: number): void {
    const window = this.store.get('window');
    this.store.set('window', { ...window, x, y });
  }

  /**
   * Save window scale
   */
  saveScale(scale: number): void {
    const window = this.store.get('window');
    this.store.set('window', { ...window, scale });
  }

  /**
   * Save window opacity
   */
  saveOpacity(opacity: number): void {
    const window = this.store.get('window');
    this.store.set('window', { ...window, opacity });
  }

  /**
   * Reset to default settings
   */
  reset(): void {
    this.store.clear();
  }
}
