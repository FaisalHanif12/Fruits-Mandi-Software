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
});
