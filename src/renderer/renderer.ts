import { PomodoroMode, CharacterName } from '../shared/types';

// Get DOM elements
const characterImg = document.getElementById('character') as HTMLImageElement;
const countdownEl = document.getElementById('countdown') as HTMLDivElement;

let currentMode: PomodoroMode = 'work';
let currentCharacter: CharacterName = 'augustine-of-hippo';
let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;

// Listen for time updates from main process
window.api.onTimeUpdate((data) => {
  // Update countdown display
  countdownEl.textContent = data.formattedTime;

  // Update mode if changed
  if (data.type !== currentMode) {
    currentMode = data.type;
    updateMode(currentMode);
  }
});

// Listen for period changes
window.api.onPeriodChange((mode) => {
  currentMode = mode;
  updateMode(mode);
});

// Listen for character changes
window.api.onCharacterChange((character) => {
  currentCharacter = character;
  updateCharacterImage();
});

// Listen for scale changes
window.api.onScaleChange((scale) => {
  document.documentElement.style.setProperty('--scale', scale.toString());
});

// Update character image based on current mode and character
function updateCharacterImage(): void {
  const imageName = currentMode === 'work' ? 'work.png' : 'quick-break.png';
  const imagePath = `assets/characters/${currentCharacter}/${imageName}`;
  characterImg.src = imagePath;
}

// Update character image and styling based on mode
function updateMode(mode: PomodoroMode): void {
  updateCharacterImage();

  // Update body class for styling
  document.body.className = `${mode}-mode`;
}

// Window dragging
// Note: Basic dragging is handled by -webkit-app-region: drag in CSS
// But we want to save position after drag
let savePositionTimeout: NodeJS.Timeout | null = null;

// Listen for mouse moves to detect when dragging ends
document.addEventListener('mousedown', (e) => {
  if (e.target === countdownEl) {
    return; // Don't start drag on countdown
  }
  isDragging = true;
  dragStartX = e.screenX;
  dragStartY = e.screenY;
});

document.addEventListener('mouseup', () => {
  if (isDragging) {
    isDragging = false;

    // Debounce position saving
    if (savePositionTimeout) {
      clearTimeout(savePositionTimeout);
    }

    savePositionTimeout = setTimeout(() => {
      saveCurrentPosition();
    }, 500);
  }
});

// Save current window position
function saveCurrentPosition(): void {
  // Get window position via Electron API (requires additional IPC)
  // For now, we'll rely on the window manager to track position
  // The main process can save position when window is moved
}

// Initialize - get settings and set up character
async function init() {
  const settings = await window.api.getSettings();
  currentCharacter = settings.character;
  // Set initial scale
  document.documentElement.style.setProperty('--scale', settings.window.scale.toString());
  updateMode(currentMode);
  console.log('Praymodoro renderer loaded');
}

init();
