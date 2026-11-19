import { formatDuration, formatMinutes, parseMinutes } from './format';

describe('format', () => {
  it('formats durations with and without seconds', () => {
    expect(formatDuration(3 * 3600 + 42 * 60 + 15)).toBe('3:42:15');
    expect(formatDuration(3 * 3600 + 42 * 60 + 15, false)).toBe('3:42');
    expect(formatDuration(-12 * 60, false)).toBe('0:12');
  });

  it('round-trips minutes', () => {
    expect(formatMinutes(450)).toBe('7:30');
    expect(parseMinutes('7:30')).toBe(450);
    expect(parseMinutes('6')).toBe(360);
    expect(parseMinutes('7.15')).toBe(435);
  });

  it('rejects invalid targets', () => {
    expect(parseMinutes('')).toBeNull();
    expect(parseMinutes('7:75')).toBeNull();
    expect(parseMinutes('25')).toBeNull();
    expect(parseMinutes('abc')).toBeNull();
  });
});
