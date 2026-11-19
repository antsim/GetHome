import { DestroyRef, Injectable, inject, signal } from '@angular/core';

import { TauriCommands } from './tauri-commands';
import { isTauri } from './tauri-env';
import { TauriEvent, type TimerViewModel } from './models';

/**
 * Exposes the backend's `timer-state` event as a signal. The backend emits
 * once per second; this service does no timing of its own.
 */
@Injectable({ providedIn: 'root' })
export class TimerStateService {
  private readonly commands = inject(TauriCommands);
  private readonly state = signal<TimerViewModel | null>(null);

  /** Latest view model, or `null` before the first update. */
  readonly current = this.state.asReadonly();

  constructor() {
    if (!isTauri()) {
      return;
    }
    const unlisten = this.commands.listen<TimerViewModel>(TauriEvent.TimerState, (vm) =>
      this.state.set(vm),
    );
    inject(DestroyRef).onDestroy(() => void unlisten.then((fn) => fn()));
    void this.commands.getState().then((vm) => this.state.set(vm));
  }
}
