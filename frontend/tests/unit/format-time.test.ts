// The time of a بکری in the list is the time of the computer, which is Pakistan time on the
// client's machine (Code Standards G9). The test builds the moment in local time, so it holds in any
// time zone.

import { describe, expect, it } from 'vitest';
import { utcToLocalDisplay, utcToLocalTime } from '../../src/shared/format';

describe('utcToLocalTime', () => {
  it('gives the local hour and minute with two digits each', () => {
    const iso = new Date(2026, 9, 10, 7, 5).toISOString();
    expect(utcToLocalTime(iso)).toBe('07:05');
    expect(utcToLocalDisplay(iso)).toBe('10-10-2026 07:05');
  });

  it('shows text it cannot read as it is', () => {
    expect(utcToLocalTime('not a time')).toBe('not a time');
  });
});
