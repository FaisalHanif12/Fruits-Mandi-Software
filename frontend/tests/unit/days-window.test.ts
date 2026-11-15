// The 7-day window helpers (plan 76 point 8, PRD FR-19.11): pure dates. The window ends on today at first,
// moves by 7 days, never passes today and never goes before the window that holds the first day of data.

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

describe('the window of 7 days', () => {
  it('is 7 days that end on the chosen day', () => {
    expect(DAYS_WINDOW).toBe(7);
    expect(daysWindowOf('2026-11-14')).toEqual({ from: '2026-11-08', to: '2026-11-14' });
    // Across a month end.
    expect(daysWindowOf('2026-11-03')).toEqual({ from: '2026-10-28', to: '2026-11-03' });
  });

  it('ends on today when nothing is asked, and never after today', () => {
    expect(daysWindowEnd(null, TODAY, null)).toBe(TODAY);
    expect(daysWindowEnd('2026-12-01', TODAY, null)).toBe(TODAY);
    expect(daysWindowEnd('2026-11-05', TODAY, null)).toBe('2026-11-05');
  });

  it('never ends before the window that holds the first day of data', () => {
    expect(daysWindowEnd('2020-01-01', TODAY, '2026-10-01')).toBe('2026-10-07');
    // Data that starts today: the one window is the one that ends today.
    expect(daysWindowEnd('2020-01-01', TODAY, '2026-11-14')).toBe(TODAY);
  });

  it('steps back and forward by 7 days inside the limits', () => {
    expect(stepDaysWindow('2026-11-14', -1, TODAY, null)).toBe('2026-11-07');
    expect(stepDaysWindow('2026-11-07', 1, TODAY, null)).toBe('2026-11-14');
    // Nearer than today stays on today.
    expect(stepDaysWindow('2026-11-14', 1, TODAY, null)).toBe(TODAY);
    expect(stepDaysWindow('2026-10-08', -1, TODAY, '2026-10-01')).toBe('2026-10-07');
  });

  it('says whether it can move', () => {
    expect(canDaysWindowComeNearer('2026-11-14', TODAY)).toBe(false);
    expect(canDaysWindowComeNearer('2026-11-07', TODAY)).toBe(true);
    expect(canDaysWindowGoBack('2026-10-07', '2026-10-01')).toBe(false);
    expect(canDaysWindowGoBack('2026-10-14', '2026-10-01')).toBe(true);
    expect(canDaysWindowGoBack('2026-10-14', null)).toBe(true);
  });
});
