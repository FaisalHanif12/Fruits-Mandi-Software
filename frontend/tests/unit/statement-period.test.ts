// The periods of the statement print box (plan 74 T3, PRD FR-12.47): the last 10 days, the last 30 days (the
// default), the last 6 months, and own dates. Pure calendar days, written as typed (DD-MM-YYYY).

import { describe, expect, it } from 'vitest';
import {
  DEFAULT_STATEMENT_PERIOD,
  STATEMENT_PERIODS,
  statementRange,
} from '../../src/features/party_ledgers/print/statement-period';

describe('the periods of the statement print', () => {
  it('offers four, in the order of the box, and starts on the last 30 days', () => {
    expect(STATEMENT_PERIODS).toEqual(['last_10', 'last_30', 'last_6_months', 'custom']);
    expect(DEFAULT_STATEMENT_PERIOD).toBe('last_30');
  });

  it('the last 10 and the last 30 days end today and include it', () => {
    expect(statementRange('last_10', '2026-11-12')).toEqual({
      from: '03-11-2026',
      to: '12-11-2026',
    });
    expect(statementRange('last_30', '2026-11-12')).toEqual({
      from: '14-10-2026',
      to: '12-11-2026',
    });
  });

  it('crosses a month, a year and a leap day right', () => {
    expect(statementRange('last_30', '2026-01-05')).toEqual({
      from: '07-12-2025',
      to: '05-01-2026',
    });
    expect(statementRange('last_10', '2028-03-05')).toEqual({
      from: '25-02-2028',
      to: '05-03-2028',
    });
  });

  it('the last 6 months start on the same day, and a month end moves to the last day of a shorter month', () => {
    expect(statementRange('last_6_months', '2026-11-12')).toEqual({
      from: '12-05-2026',
      to: '12-11-2026',
    });
    expect(statementRange('last_6_months', '2026-08-31')).toEqual({
      from: '28-02-2026',
      to: '31-08-2026',
    });
    expect(statementRange('last_6_months', '2026-03-31')).toEqual({
      from: '30-09-2025',
      to: '31-03-2026',
    });
  });

  it('own dates have no days of their own, and a bad today gives none', () => {
    expect(statementRange('custom', '2026-11-12')).toBeNull();
    expect(statementRange('last_30', 'not a date')).toBeNull();
  });
});
