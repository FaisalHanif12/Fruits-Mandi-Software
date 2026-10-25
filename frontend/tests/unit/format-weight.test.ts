// A weight in grams shown as kilograms (plan 17d, PRD 16.7): text work on the digits only.

import { describe, expect, it } from 'vitest';
import { formatGrams } from '../../src/shared/format';

describe('formatGrams', () => {
  it('writes whole grams as kilograms with up to three decimals', () => {
    expect(formatGrams(2500)).toBe('2.5');
    expect(formatGrams(125)).toBe('0.125');
    expect(formatGrams(3000)).toBe('3');
    expect(formatGrams(12_345)).toBe('12.345');
    expect(formatGrams(7)).toBe('0.007');
    expect(formatGrams(0)).toBe('0');
  });

  it('shows anything that is not a whole number of grams as it is', () => {
    expect(formatGrams(-5)).toBe('-5');
    expect(formatGrams(1.5)).toBe('1.5');
  });
});
