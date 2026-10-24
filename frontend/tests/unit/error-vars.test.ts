// messageVars: the values of a backend error, ready for Fluent (plan 02 contract 4.6, review
// finding 2). Error values are display units, never paisa.

import { describe, expect, it } from 'vitest';
import { messageVars } from '../../src/shared/api';

describe('messageVars', () => {
  it('groups the money limit, 10,00,00,00,000 rupees, in both groupings', () => {
    expect(messageVars({ max: 10_000_000_000 }, 'international')).toEqual({
      max: '10,000,000,000',
    });
    expect(messageVars({ max: 10_000_000_000 }, 'lakh')).toEqual({ max: '10,00,00,00,000' });
  });

  it('groups smaller limits and leaves short ones alone', () => {
    expect(messageVars({ max: 1_000_000 }, 'lakh')).toEqual({ max: '10,00,000' });
    expect(messageVars({ max: 500 }, 'international')).toEqual({ max: '500' });
    expect(messageVars({ max: 2 }, 'lakh')).toEqual({ max: '2' });
  });

  it('keeps the minus sign of a negative number in front of the grouped digits', () => {
    expect(messageVars({ available: -1_316_040 }, 'international')).toEqual({
      available: '-1,316,040',
    });
    expect(messageVars({ available: -1_316_040 }, 'lakh')).toEqual({ available: '-13,16,040' });
    expect(messageVars({ available: -5 }, 'lakh')).toEqual({ available: '-5' });
  });

  it('passes text values, such as a date, exactly as sent', () => {
    expect(messageVars({ closed_up_to: '09-10-2026' }, 'lakh')).toEqual({
      closed_up_to: '09-10-2026',
    });
    expect(messageVars({ today: '10-10-2026', max: 100_000 }, 'international')).toEqual({
      today: '10-10-2026',
      max: '100,000',
    });
  });

  it('shows a value that is exactly a real day, YYYY-MM-DD, as DD-MM-YYYY (the garden codes)', () => {
    // ERR_CLOSE_BEFORE_LAST_ENTRY sends last_date, ERR_DATE_BEFORE_GARDEN sends garden_date.
    expect(messageVars({ last_date: '2026-10-12', garden_date: '2026-10-10' }, 'lakh')).toEqual({
      last_date: '12-10-2026',
      garden_date: '10-10-2026',
    });
    // A leap day is a real day.
    expect(messageVars({ date: '2028-02-29' }, 'international')).toEqual({ date: '29-02-2028' });
  });

  it('leaves every other value alone: other text, a day that does not exist, a number', () => {
    expect(
      messageVars(
        {
          word: 'abc',
          month_only: '2026-10',
          reference: 'E-20261010-0001',
          padded: ' 2026-10-12',
          later: '2026-10-12 and more',
          no_month: '2026-13-01',
          no_day: '2026-02-30',
          no_leap_day: '2027-02-29',
          zero_day: '2026-10-00',
          shown_already: '12-10-2026',
        },
        'lakh',
      ),
    ).toEqual({
      word: 'abc',
      month_only: '2026-10',
      reference: 'E-20261010-0001',
      padded: ' 2026-10-12',
      later: '2026-10-12 and more',
      no_month: '2026-13-01',
      no_day: '2026-02-30',
      no_leap_day: '2027-02-29',
      zero_day: '2026-10-00',
      shown_already: '12-10-2026',
    });
    // A number that has the digits of a date is still a number.
    expect(messageVars({ count: 20261012 }, 'international')).toEqual({ count: '20,261,012' });
  });

  it('also gives a rate in basis points as the percent a person reads, from its digits', () => {
    // ERR_SHARES_NOT_100 sends total_bps (PRD FR-17.1): 9000 is 90%, 9050 is 90.5%, 5 is 0.05%.
    expect(messageVars({ total_bps: 9000 }, 'international')).toEqual({
      total_bps: '9,000',
      total: '90',
    });
    expect(messageVars({ total_bps: 9050 }, 'lakh')).toEqual({ total_bps: '9,050', total: '90.5' });
    expect(messageVars({ total_bps: 5 }, 'international')).toEqual({
      total_bps: '5',
      total: '0.05',
    });
    // A text value with that ending is not a number and is left alone.
    expect(messageVars({ note_bps: 'x' }, 'international')).toEqual({ note_bps: 'x' });
  });

  it('gives an empty set for an error with no values', () => {
    expect(messageVars({}, 'international')).toEqual({});
  });

  it('writes an amount named *_paisa as money, under both names (plan 12 contract 4)', () => {
    // ERR_CASH_BELOW_ZERO_NOT_CONFIRMED sends cash_after_paisa; the message uses $cash_after.
    expect(messageVars({ cash_after_paisa: -500_000 }, 'international')).toEqual({
      cash_after_paisa: '-5,000',
      cash_after: '-5,000',
    });
    // ERR_CREDIT_LIMIT_EXCEEDED sends would_owe_paisa and limit_paisa.
    expect(messageVars({ would_owe_paisa: 1_288_000, limit_paisa: 100_000_000 }, 'lakh')).toEqual({
      would_owe_paisa: '12,880',
      would_owe: '12,880',
      limit_paisa: '10,00,000',
      limit: '10,00,000',
    });
  });

  it('a name that merely ends like paisa but holds text is passed as sent', () => {
    expect(messageVars({ note_paisa: 'x' }, 'lakh')).toEqual({ note_paisa: 'x' });
  });
});
