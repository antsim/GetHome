import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';

import { SettingsService } from '../core/settings.service';
import { TauriCommands, toAppError } from '../core/tauri-commands';
import { TimerStateService } from '../core/timer-state.service';
import { toTimerView } from './timer-view';

const ERROR_VISIBLE_MS = 5000;

/** The small always-on-top window showing the remaining working time. */
@Component({
  selector: 'app-timer-window',
  templateUrl: './timer-window.html',
  styleUrl: './timer-window.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  host: {
    '[style.--surface-alpha]': 'surfaceAlpha()',
  },
})
export class TimerWindow {
  private readonly commands = inject(TauriCommands);
  private readonly timerState = inject(TimerStateService);
  private readonly settings = inject(SettingsService);

  protected readonly view = computed(() => {
    const vm = this.timerState.current();
    return vm ? toTimerView(vm) : null;
  });
  protected readonly surfaceAlpha = computed(
    () => (this.settings.current()?.windowOpacity ?? 100) / 100,
  );
  protected readonly pending = signal(false);
  protected readonly error = signal<string | null>(null);
  protected readonly Math = Math;
  private errorTimer: ReturnType<typeof setTimeout> | undefined;

  protected async toggleTimer(): Promise<void> {
    const view = this.view();
    if (!view || this.pending()) {
      return;
    }
    this.pending.set(true);
    try {
      await (view.running ? this.commands.stopTimer() : this.commands.resumeTimer());
      this.showError(null);
    } catch (err) {
      this.showError(toAppError(err).message);
    } finally {
      this.pending.set(false);
    }
  }

  protected openSettings(): void {
    void this.commands.openSettings();
  }

  private showError(message: string | null): void {
    clearTimeout(this.errorTimer);
    this.error.set(message);
    if (message) {
      this.errorTimer = setTimeout(() => this.error.set(null), ERROR_VISIBLE_MS);
    }
  }
}
