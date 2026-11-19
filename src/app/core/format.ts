/** Formats seconds as `H:MM` or `H:MM:SS`, using the absolute value. */
export function formatDuration(secs: number, withSeconds = true): string {
  const total = Math.floor(Math.abs(secs));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const mm = String(minutes).padStart(2, '0');
  return withSeconds ? `${hours}:${mm}:${String(seconds).padStart(2, '0')}` : `${hours}:${mm}`;
}

/** Formats an ISO timestamp as local `HH:MM`. */
export function formatClock(iso: string): string {
  const date = new Date(iso);
  const hh = String(date.getHours()).padStart(2, '0');
  const mm = String(date.getMinutes()).padStart(2, '0');
  return `${hh}:${mm}`;
}

/** Formats minutes as `H:MM` (e.g. 450 → `7:30`). */
export function formatMinutes(minutes: number): string {
  return `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, '0')}`;
}

/**
 * Parses `H`, `H:MM` or `H.MM` into minutes. Returns `null` for invalid input
 * or values above 24 hours.
 */
export function parseMinutes(text: string): number | null {
  const match = /^\s*(\d{1,2})(?:[:.]([0-5]\d))?\s*$/.exec(text);
  if (!match) {
    return null;
  }
  const minutes = Number(match[1]) * 60 + Number(match[2] ?? 0);
  return minutes <= 24 * 60 ? minutes : null;
}
