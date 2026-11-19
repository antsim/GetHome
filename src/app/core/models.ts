/**
 * Types shared with the Rust backend. Keep in sync with the serde models in
 * `src-tauri/src` (field names are camelCase on the wire).
 */

export type TimerStatus = 'NotStarted' | 'Working' | 'OnBreak' | 'Done';

export type SyncWarning = 'none' | 'rateLimited' | 'offline' | 'authError' | 'noToken';

/** Payload of the `timer-state` event (`engine/compute.rs`). */
export interface TimerViewModel {
  status: TimerStatus;
  /** Negative means overtime. */
  remainingSecs: number;
  workedSecs: number;
  targetSecs: number;
  breakSecs: number;
  /** ISO timestamp of the estimated finish, only while working. */
  eta: string | null;
  running: boolean;
  lastSynced: string | null;
  syncWarning: SyncWarning;
}

/** `settings/mod.rs` → `Settings`. */
export interface Settings {
  /** Minutes per weekday, Monday first (7 items). */
  dailyTargets: number[];
  /** IANA time zone, or `null` to follow the system. */
  timezone: string | null;
  showSeconds: boolean;
  autostart: boolean;
  /** Percent, 60–100. */
  windowOpacity: number;
  timerWindowVisible: boolean;
}

export interface AccountInfo {
  name: string;
  workspaceName: string | null;
  defaultWorkspaceId: number;
}

export interface SettingsView {
  settings: Settings;
  account: AccountInfo | null;
  hasToken: boolean;
  systemTimezone: string;
}

export type RefreshOutcome = 'refreshed' | 'throttled' | 'rateLimited' | 'failed';

/** Error returned by commands (`error.rs`). `message` is user-facing. */
export interface AppError {
  code: string;
  message: string;
}

export const TauriEvent = {
  TimerState: 'timer-state',
  SettingsChanged: 'settings-changed',
} as const;

export const MIN_OPACITY = 60;
export const MAX_OPACITY = 100;
