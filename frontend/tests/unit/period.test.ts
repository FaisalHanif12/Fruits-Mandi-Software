// The periods of a list (plan 38 R4, PRD FR-19.2): the days each one covers, from the day the app is on.

import { describe, expect, it } from 'vitest';
import { periodMessageId, periodRange } from '../../src/shared/format';

describe('periodRange for Saturday 10-10-2026', () => {
  const today = '2026-10-10';

  it('today is the one day', () => {
    expect(periodRange('today', today)).toEqual({ from: '10-10-2026', to: '10-10-2026' });
  });

  it('this week starts on the Monday and ends today', () => {
    expect(periodRange('week', today)).toEqual({ from: '05-10-2026', to: '10-10-2026' });
    // On a Monday the week is that day alone.
    expect(periodRange('week', '2026-10-05')).toEqual({ from: '05-10-2026', to: '05-10-2026' });
    // On a Sunday it reaches back six days.
    expect(periodRange('week', '2026-10-11')).toEqual({ from: '05-10-2026', to: '11-10-2026' });
  });

  it('this month and this year run from their first day to today', () => {
    expect(periodRange('month', today)).toEqual({ from: '01-10-2026', to: '10-10-2026' });
    expect(periodRange('year', today)).toEqual({ from: '01-01-2026', to: '10-10-2026' });
  });

  it('last month and last year are the whole of them', () => {
    expect(periodRange('last_month', today)).toEqual({ from: '01-09-2026', to: '30-09-2026' });
    expect(periodRange('last_year', today)).toEqual({ from: '01-01-2025', to: '31-12-2025' });
  });

  it('last month crosses a year and a short February', () => {
    expect(periodRange('last_month', '2026-01-15')).toEqual({
      from: '01-12-2025',
      to: '31-12-2025',
    });
    expect(periodRange('last_month', '2028-03-02')).toEqual({
      from: '01-02-2028',
      to: '29-02-2028',
    });
  });

  it('the first date has no end, and an unreadable day gives no days', () => {
    expect(periodRange('all', today)).toEqual({ from: '', to: '' });
    expect(periodRange('custom', today)).toEqual({ from: '', to: '' });
    expect(periodRange('month', 'later')).toEqual({ from: '', to: '' });
  });

  it('names the Fluent message of each kind', () => {
    expect(periodMessageId('last_month')).toBe('common-period-last-month');
    expect(periodMessageId('today')).toBe('common-period-today');
  });
});
