import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type WindowKind = 'timer' | 'settings';

/** Which view to render, based on the label of the Tauri window. */
export function currentWindowKind(): WindowKind {
  if (isTauri() && getCurrentWindow().label === 'settings') {
    return 'settings';
  }
  // In a plain browser (`ng serve`) `?view=settings` previews the settings view.
  return new URLSearchParams(window.location.search).get('view') === 'settings'
    ? 'settings'
    : 'timer';
}

export { isTauri };
