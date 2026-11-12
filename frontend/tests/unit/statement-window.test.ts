// The window of 30 days of one party's statement (plan 74 W1 to W5, PRD FR-12.47): the date arithmetic and
// the address. Pure functions, no screen, no money. Today in these examples is 12-11-2026.

import { describe, expect, it } from 'vitest';
import { parseLedgerSearch } from '../../src/features/party_ledgers/search';
import {
  STATEMENT_DAYS,
  canComeNearer,
  statementEnd,
  statementWindowOf,
  stepStatementEnd,
} from '../../src/features/party_ledgers/statement-window';
import { addDays, dayOfText, isIsoDay } from '../../src/shared/format';

describe('the window of 30 days', () => {
  it('is 30 days, and the first window of 12-11-2026 is 14-10-2026 to 12-11-2026 (PRD FR-12.47)', () => {
    expect(STATEMENT_DAYS).toBe(30);
    expect(statementEnd(null, '2026-11-12')).toBe('2026-11-12');
    expect(statementWindowOf('2026-11-12')).toEqual({ from: '2026-10-14', to: '2026-11-12' });
  });

  it('a day after today is never the end; a day before is kept', () => {
    expect(statementEnd('2026-11-20', '2026-11-12')).toBe('2026-11-12');
    expect(statementEnd('2026-10-14', '2026-11-12')).toBe('2026-10-14');
  });

  it('one step back is 14-09-2026 to 13-10-2026, and a step back and forward comes home', () => {
    const back = stepStatementEnd('2026-11-12', -1, '2026-11-12');
    expect(statementWindowOf(back)).toEqual({ from: '2026-09-14', to: '2026-10-13' });
    expect(stepStatementEnd(back, 1, '2026-11-12')).toBe('2026-11-12');
  });

  it('the later button stops at today: 20 days from today is today, not past it', () => {
    expect(stepStatementEnd('2026-11-01', 1, '2026-11-12')).toBe('2026-11-12');
    expect(stepStatementEnd('2026-11-12', 1, '2026-11-12')).toBe('2026-11-12');
  });

  it('can come nearer only while the window does not end today', () => {
    expect(canComeNearer('2026-11-12', '2026-11-12')).toBe(false);
    expect(canComeNearer('2026-11-11', '2026-11-12')).toBe(true);
  });

  it('the windows work at a year end and in a leap year', () => {
    expect(statementWindowOf('2026-01-10')).toEqual({ from: '2025-12-12', to: '2026-01-10' });
    expect(statementWindowOf('2028-03-10')).toEqual({ from: '2028-02-10', to: '2028-03-10' });
  });
});

describe('the plain-day helpers of shared/format', () => {
  it('a real day is a real day, and a day that is not is refused', () => {
    expect(isIsoDay('2028-02-29')).toBe(true);
    expect(isIsoDay('2026-02-29')).toBe(false);
    expect(isIsoDay('2026-13-01')).toBe(false);
    expect(isIsoDay(20261112)).toBe(false);
  });

  it('addDays moves over a month end', () => {
    expect(addDays('2026-10-31', 1)).toBe('2026-11-01');
    expect(addDays('2026-03-01', -1)).toBe('2026-02-28');
  });

  it('a typed date reads with the shortcuts of FR-7.19, relative to the backend’s today', () => {
    expect(dayOfText('05-11-2026', '2026-11-12')).toBe('2026-11-05');
    expect(dayOfText('5', '2026-11-12')).toBe('2026-11-05');
    expect(dayOfText('5-9', '2026-11-12')).toBe('2026-09-05');
    expect(dayOfText('31-02-2026', '2026-11-12')).toBeNull();
  });
});

describe('the address of one party’s statement keeps the day the window ends on', () => {
  it('reads a real day only with a party and no slip date', () => {
    expect(parseLedgerSearch({ side: 'buyer', party: 20, end: '2026-09-10' })).toEqual({
      side: 'buyer',
      party: 20,
      end: '2026-09-10',
    });
    expect(parseLedgerSearch({ party: 20, end: '2026-02-30' })).toEqual({ party: 20 });
    expect(parseLedgerSearch({ end: '2026-09-10' })).toEqual({});
    expect(parseLedgerSearch({ party: 20, date: '2026-10-10', end: '2026-09-10' })).toEqual({
      party: 20,
      date: '2026-10-10',
    });
  });
});
