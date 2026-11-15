// The month helpers of the one-month windows (plan 76, PRD FR-9.3, FR-17.12, FR-19.11). Pure dates, no
// screen: the arithmetic across a year end, the limits (first month of data, month of today), and
// the typed shortcuts of the go-to-month box.

import { describe, expect, it } from 'vitest';
import {
  addMonths,
  canMonthComeNearer,
  canMonthGoBack,
  displayMonth,
  isIsoMonth,
  monthBounds,
  monthOf,
  monthOfText,
  stepMonth,
  windowMonth,
} from '../../src/shared/format';

describe('month arithmetic', () => {
  it('moves across a year end in both directions', () => {
    expect(addMonths('2026-11', 2)).toBe('2027-01');
    expect(addMonths('2026-01', -1)).toBe('2025-12');
    expect(addMonths('2026-06', 0)).toBe('2026-06');
    expect(addMonths('2026-01', -13)).toBe('2024-12');
  });

  it('gives the first and last day, a leap February too', () => {
    expect(monthBounds('2026-10')).toEqual({ from: '2026-10-01', to: '2026-10-31' });
    expect(monthBounds('2026-02')).toEqual({ from: '2026-02-01', to: '2026-02-28' });
    expect(monthBounds('2028-02')).toEqual({ from: '2028-02-01', to: '2028-02-29' });
    expect(monthBounds('2026-12')).toEqual({ from: '2026-12-01', to: '2026-12-31' });
  });

  it('reads and writes the month', () => {
    expect(isIsoMonth('2026-10')).toBe(true);
    expect(isIsoMonth('2026-13')).toBe(false);
    expect(isIsoMonth('2026-1')).toBe(false);
    expect(isIsoMonth(5)).toBe(false);
    expect(monthOf('2026-11-12')).toBe('2026-11');
    expect(displayMonth('2026-10')).toBe('10-2026');
  });
});

describe('the window of one month', () => {
  it('opens on the month of today and never passes it', () => {
    expect(windowMonth(null, '2026-11-12', null)).toBe('2026-11');
    expect(windowMonth('2027-03', '2026-11-12', null)).toBe('2026-11');
    expect(windowMonth('2026-09', '2026-11-12', null)).toBe('2026-09');
  });

  it('never goes before the first month of data', () => {
    expect(windowMonth('2020-01', '2026-11-12', '2026-02')).toBe('2026-02');
    expect(stepMonth('2026-02', -1, '2026-11-12', '2026-02')).toBe('2026-02');
    expect(stepMonth('2026-11', 1, '2026-11-12', '2026-02')).toBe('2026-11');
    expect(stepMonth('2026-11', -1, '2026-11-12', '2026-02')).toBe('2026-10');
  });

  it('says whether it can move', () => {
    expect(canMonthComeNearer('2026-11', '2026-11-12')).toBe(false);
    expect(canMonthComeNearer('2026-10', '2026-11-12')).toBe(true);
    expect(canMonthGoBack('2026-02', '2026-02')).toBe(false);
    expect(canMonthGoBack('2026-03', '2026-02')).toBe(true);
    expect(canMonthGoBack('2026-03', null)).toBe(true);
  });
});

describe('the typed month', () => {
  it('reads a month, a month with a year, and a whole date', () => {
    expect(monthOfText('9', '2026-11-12')).toBe('2026-09');
    expect(monthOfText('9-25', '2026-11-12')).toBe('2025-09');
    expect(monthOfText('10-2026', '2026-11-12')).toBe('2026-10');
    expect(monthOfText('12-10-2026', '2026-11-12')).toBe('2026-10');
  });

  it('refuses what is not a month', () => {
    expect(monthOfText('', '2026-11-12')).toBeNull();
    expect(monthOfText('13', '2026-11-12')).toBeNull();
    expect(monthOfText('0', '2026-11-12')).toBeNull();
    expect(monthOfText('abc', '2026-11-12')).toBeNull();
    expect(monthOfText('31-02-2026', '2026-11-12')).toBeNull();
  });
});
