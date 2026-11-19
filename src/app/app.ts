import { ChangeDetectionStrategy, Component } from '@angular/core';

import { currentWindowKind } from './core/tauri-env';
import { SettingsWindow } from './settings/settings-window';
import { TimerWindow } from './timer/timer-window';

/** Root component: each Tauri window renders one view, chosen by its label. */
@Component({
  selector: 'app-root',
  imports: [TimerWindow, SettingsWindow],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @switch (kind) {
      @case ('settings') {
        <app-settings-window />
      }
      @default {
        <app-timer-window />
      }
    }
  `,
})
export class App {
  protected readonly kind = currentWindowKind();
}
