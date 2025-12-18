const { app, BrowserWindow } = require('electron');
const path = require('path');
const { TimeController } = require('./time-controller');
const { WindowManager } = require('./window-manager');
const { TrayManager } = require('./tray-manager');
const { SettingsManager } = require('./settings-manager');
const { registerIPCHandlers } = require('./ipc-handlers');

// Keep references to prevent garbage collection
let timeController = null;
let windowManager = null;
let trayManager = null;
let settingsManager = null;

function createApp() {
  // Initialize settings manager
  settingsManager = new SettingsManager();
  const settings = settingsManager.getSettings();

  // Paths for preload and renderer
  const preloadPath = path.join(__dirname, '../preload/preload.js');
  const rendererPath = path.join(__dirname, '../renderer/index.html');

  // Initialize window manager
  windowManager = new WindowManager(
    preloadPath,
    rendererPath,
    settings.window.x,
    settings.window.y,
    settings.window.scale
  );

  // Create the character window
  windowManager.createWindow();

  // Initialize time controller
  timeController = new TimeController();

  // Initialize tray manager
  const trayIconPath = path.join(__dirname, '../../assets/icons/tray-icon.png');
  trayManager = new TrayManager(trayIconPath, windowManager);
  trayManager.createTray();

  // Register IPC handlers
  registerIPCHandlers(windowManager, settingsManager);

  // Connect time controller to UI
  timeController.on('tick', (state) => {
    const formattedTime = timeController ? timeController.formatTime(state.remaining) : '00:00';

    // Update window
    if (windowManager) windowManager.sendTimeUpdate(state, formattedTime);

    // Update tray
    if (trayManager) trayManager.updateCountdown(formattedTime);
  });

  timeController.on('period-change', (state) => {
    // Update character mode on period transition
    if (windowManager) windowManager.updateCharacterMode(state.type);
  });

  // Start time tracking
  timeController.start();

  // Hide dock icon if setting is enabled
  if (settings.showInDock === false && app.dock) {
    app.dock.hide();
  }
}

// App lifecycle
app.whenReady().then(() => {
  createApp();

  // On macOS, re-create window when dock icon is clicked
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createApp();
    }
  });
});

// Quit when all windows are closed (except on macOS)
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// Cleanup on quit
app.on('before-quit', () => {
  if (timeController) timeController.destroy();
  if (windowManager) windowManager.destroy();
  if (trayManager) trayManager.destroy();
});

// Handle any uncaught errors
process.on('uncaughtException', (error) => {
  console.error('Uncaught exception:', error);
});
