import { EventEmitter } from 'events';
import { powerMonitor } from 'electron';
import { PomodoroState, PomodoroMode } from '../shared/types';
import { POMODORO_SEGMENTS } from '../shared/constants';

export class TimeController extends EventEmitter {
  private intervalId: NodeJS.Timeout | null = null;
  private lastMode: PomodoroMode | null = null;

  constructor() {
    super();
    this.setupPowerMonitor();
  }

  /**
   * Calculate current pomodoro period based on system time
   */
  getCurrentPeriod(): PomodoroState {
    const now = new Date();
    const minutes = now.getMinutes();
    const seconds = now.getSeconds();

    // Find current segment based on current minutes
    const segment = POMODORO_SEGMENTS.find(
      s => minutes >= s.startMinute && minutes < s.endMinute
    );

    if (!segment) {
      throw new Error(`Invalid time state: ${minutes}:${seconds}`);
    }

    // Calculate remaining time in seconds
    const currentSecond = minutes * 60 + seconds;
    const endSecond = segment.endMinute * 60;
    const remaining = endSecond - currentSecond;

    return {
      type: segment.type,
      remaining: remaining,
    };
  }

  /**
   * Format seconds as MM:SS
   */
  formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  /**
   * Start the time tracking interval
   */
  start(): void {
    if (this.intervalId) {
      return; // Already running
    }

    // Initial tick
    this.tick();

    // Update every second
    this.intervalId = setInterval(() => {
      this.tick();
    }, 1000);
  }

  /**
   * Stop the time tracking interval
   */
  stop(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * Called every second to check time and emit events
   */
  private tick(): void {
    const state = this.getCurrentPeriod();

    // Emit tick event with current state
    this.emit('tick', state);

    // Check if period changed
    if (this.lastMode !== null && this.lastMode !== state.type) {
      this.emit('period-change', state);
    }

    this.lastMode = state.type;
  }

  /**
   * Setup power monitor to handle system sleep/wake
   */
  private setupPowerMonitor(): void {
    // Recalculate immediately on system wake
    powerMonitor.on('resume', () => {
      this.tick();
    });

    // Optionally handle other power events
    powerMonitor.on('suspend', () => {
      // System going to sleep - could log or cleanup if needed
    });
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    this.stop();
    this.removeAllListeners();
  }
}
