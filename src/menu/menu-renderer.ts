import { MenuState, CharacterName } from '../shared/types';

function formatCharacterName(name: CharacterName): string {
  return name
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

async function updateUI(): Promise<void> {
  const state: MenuState = await window.menuApi.getState();

  // Update countdown
  const countdownEl = document.getElementById('countdown');
  if (countdownEl) {
    countdownEl.querySelector('.menu-label')!.textContent = `Time Remaining: ${state.countdown}`;
  }

  // Update toggle character button
  const toggleEl = document.getElementById('toggle-character');
  if (toggleEl) {
    toggleEl.textContent = state.isCharacterVisible ? 'Hide' : 'Show';
  }

  // Update size label
  const sizeEl = document.getElementById('size-label');
  if (sizeEl) {
    const scalePercent = Math.round(state.scale * 100);
    sizeEl.textContent = `Size: ${scalePercent}%`;
  }

  // Update character label
  const characterEl = document.getElementById('character-label');
  if (characterEl) {
    const label = characterEl.querySelector('.menu-label');
    if (label) {
      label.textContent = `Character: ${formatCharacterName(state.character)}`;
    }
  }
}

function updateCountdown(countdown: string): void {
  const countdownEl = document.getElementById('countdown');
  if (countdownEl) {
    const label = countdownEl.querySelector('.menu-label');
    if (label) {
      label.textContent = `Time Remaining: ${countdown}`;
    }
  }
}

function setupEventListeners(): void {
  // Toggle character
  document.getElementById('toggle-character')?.addEventListener('click', async () => {
    window.menuApi.sendAction({ type: 'toggle-character' });
    await updateUI();
  });

  // Increase size - stays open
  document.getElementById('increase-size')?.addEventListener('click', async () => {
    window.menuApi.sendAction({ type: 'increase-size' });
    await updateUI();
  });

  // Decrease size - stays open
  document.getElementById('decrease-size')?.addEventListener('click', async () => {
    window.menuApi.sendAction({ type: 'decrease-size' });
    await updateUI();
  });

  // Next character - stays open
  document.getElementById('next-character')?.addEventListener('click', async () => {
    window.menuApi.sendAction({ type: 'next-character' });
    await updateUI();
  });

  // Quit - closes app
  document.getElementById('quit')?.addEventListener('click', () => {
    window.menuApi.sendAction({ type: 'quit' });
  });
}

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
  await updateUI();
  setupEventListeners();

  // Listen for real-time countdown updates
  window.menuApi.onTimeUpdate((countdown) => {
    updateCountdown(countdown);
  });
});
