import { DestroyRef, Injectable, inject, signal } from '@angular/core';

import { TauriCommands } from './tauri-commands';
import { isTauri } from './tauri-env';
import { TauriEvent, type Settings } from './models';

/** Current settings as a signal, kept up to date via `settings-changed`. */
@Injectable({ providedIn: 'root' })
export class SettingsService {
  private readonly commands = inject(TauriCommands);
  private readonly state = signal<Settings | null>(null);

  readonly current = this.state.asReadonly();

  constructor() {
    if (!isTauri()) {
      return;
    }
    const unlisten = this.commands.listen<Settings>(TauriEvent.SettingsChanged, (settings) =>
      this.state.set(settings),
    );
    inject(DestroyRef).onDestroy(() => void unlisten.then((fn) => fn()));
    void this.commands.getSettings().then((view) => this.state.set(view.settings));
  }
}
