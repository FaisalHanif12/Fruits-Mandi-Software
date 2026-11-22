// The window of days that every list of days shows (plan 76 point 8, plan 95 items 14 and 18, PRD FR-19.11 v1.57): pure
// dates. The window is 30 days that end on today at first, moves by 30 days, never passes today and never goes before
// the window that holds the first day of data. A shorter window is a size given to the same functions (پچھلی بکری keeps
// its own 7 days in `features/bikri/past-window.ts`, FR-12.46).

import { describe, expect, it } from 'vitest';
import {
  DAYS_WINDOW,
  canDaysWindowComeNearer,
  canDaysWindowGoBack,
  daysWindowEnd,
  daysWindowOf,
  stepDaysWindow,
} from '../../src/shared/format';

const TODAY = '2026-11-14';

describe('the window of 30 days', () => {
  it('is 30 days that end on the chosen day, across month ends', () => {
    expect(DAYS_WINDOW).toBe(30);
    expect(daysWindowOf('2026-11-14')).toEqual({ from: '2026-10-16', to: '2026-11-14' });
    expect(daysWindowOf('2026-03-05')).toEqual({ from: '2026-02-04', to: '2026-03-05' });
  });

  it('ends on today when nothing is asked, and never after today', () => {
    expect(daysWindowEnd(null, TODAY, null)).toBe(TODAY);
    expect(daysWindowEnd('2026-12-01', TODAY, null)).toBe(TODAY);
    expect(daysWindowEnd('2026-11-05', TODAY, null)).toBe('2026-11-05');
  });

  it('never ends before the window that holds the first day of data', () => {
    // First day 01-10-2026: the first window of 30 days that holds it ends on 30-10-2026.
    expect(daysWindowEnd('2020-01-01', TODAY, '2026-10-01')).toBe('2026-10-30');
    // Data that starts today: the one window is the one that ends today.
    expect(daysWindowEnd('2020-01-01', TODAY, '2026-11-14')).toBe(TODAY);
  });

  it('steps back and forward by 30 days inside the limits', () => {
    expect(stepDaysWindow('2026-11-14', -1, TODAY, null)).toBe('2026-10-15');
    expect(stepDaysWindow('2026-10-15', 1, TODAY, null)).toBe(TODAY);
    // Nearer than today stays on today.
    expect(stepDaysWindow('2026-11-14', 1, TODAY, null)).toBe(TODAY);
    expect(stepDaysWindow('2026-10-31', -1, TODAY, '2026-10-01')).toBe('2026-10-30');
  });

  it('says whether it can move', () => {
    expect(canDaysWindowComeNearer('2026-11-14', TODAY)).toBe(false);
    expect(canDaysWindowComeNearer('2026-10-15', TODAY)).toBe(true);
    expect(canDaysWindowGoBack('2026-10-30', '2026-10-01')).toBe(false);
    expect(canDaysWindowGoBack('2026-11-14', '2026-10-01')).toBe(true);
    expect(canDaysWindowGoBack('2026-11-14', null)).toBe(true);
  });
});

describe('a window of another size', () => {
  it('works with the size given, 7 days for example', () => {
    expect(daysWindowOf('2026-11-14', 7)).toEqual({ from: '2026-11-08', to: '2026-11-14' });
    expect(stepDaysWindow('2026-11-14', -1, TODAY, null, 7)).toBe('2026-11-07');
    expect(daysWindowEnd('2020-01-01', TODAY, '2026-10-01', 7)).toBe('2026-10-07');
    expect(canDaysWindowGoBack('2026-10-07', '2026-10-01', 7)).toBe(false);
  });
});
