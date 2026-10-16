// formatPaisa mirrors the Rust format_paisa cases (plan 02 contract 4.1 and 4.7, M12).

import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  formatPaisa,
  formatPaisaValue,
  groupDigits,
  type Grouping,
} from '../../src/shared/format/money';
import { printDigits, toUrduDigits } from '../../src/shared/format/digits';
import { paisaForTest } from '../helpers/paisa';

const cases: readonly (readonly [paisa: number, international: string, lakh: string])[] = [
  // Every row of the Rust format_paisa test (backend/crates/core/src/money/format.rs) is here.
  [0, '0', '0'],
  [700, '7', '7'],
  [99_900, '999', '999'],
  [4_500, '45', '45'],
  [10_000, '100', '100'],
  [100_000, '1,000', '1,000'],
  [1_200_000, '12,000', '12,000'],
  [1_931_600, '19,316', '19,316'],
  [10_000_000, '100,000', '1,00,000'],
  [131_604_000, '1,316,040', '13,16,040'],
  // 100 crore rupees.
  [100_000_000_000, '1,000,000,000', '1,00,00,00,000'],
  // 1,000,000,000,000 paisa: the largest allowed amount, 10,00,00,00,000 rupees (دس ارب,
  // Code Standards 4.2).
  [1_000_000_000_000, '10,000,000,000', '10,00,00,00,000'],
  // Above the input limit, but a balance can grow past it: still grouped correctly.
  [100_000_000_000_000, '1,000,000,000,000', '10,00,00,00,00,000'],
  [-1_200_000, '-12,000', '-12,000'],
  [-131_604_000, '-1,316,040', '-13,16,040'],
  [-500, '-5', '-5'],
];

describe('formatPaisa', () => {
  it.each(cases)('%d paisa -> %s / %s', (paisa, international, lakh) => {
    expect(formatPaisa(paisaForTest(paisa), { grouping: 'international' })).toBe(international);
    expect(formatPaisa(paisaForTest(paisa), { grouping: 'lakh' })).toBe(lakh);
  });

  it.each([150, 1, -50, 0.5, 100.5, Number.MAX_SAFE_INTEGER + 1, Number.NaN, Infinity])(
    'throws in development on %d, which is not whole rupees in paisa',
    (bad) => {
      expect(() => formatPaisa(paisaForTest(bad), { grouping: 'international' })).toThrow(
        /not a whole number of rupees/,
      );
    },
  );

  describe('in production', () => {
    afterEach(() => {
      vi.unstubAllEnvs();
      vi.restoreAllMocks();
    });

    it('logs and shows a dash instead of stopping the screen', () => {
      vi.stubEnv('DEV', false);
      const log = vi.spyOn(console, 'error').mockImplementation(() => undefined);
      expect(formatPaisa(paisaForTest(150), { grouping: 'lakh' })).toBe('—');
      expect(log).toHaveBeenCalledOnce();
    });
  });
});

describe('groupDigits', () => {
  const groupings: readonly Grouping[] = ['international', 'lakh'];
  it.each(groupings)('leaves up to three digits alone (%s)', (grouping) => {
    expect(groupDigits('7', grouping)).toBe('7');
    expect(groupDigits('999', grouping)).toBe('999');
  });

  it('groups lakh as 2-2-3 from the end', () => {
    expect(groupDigits('1000', 'lakh')).toBe('1,000');
    expect(groupDigits('99999', 'lakh')).toBe('99,999');
    expect(groupDigits('999999999', 'lakh')).toBe('99,99,99,999');
  });
});

describe('print digits (NFR-URD-14)', () => {
  it('turns English digits into Urdu digits and keeps the separators', () => {
    expect(toUrduDigits('13,16,040')).toBe('۱۳,۱۶,۰۴۰');
    expect(toUrduDigits('0123456789')).toBe('۰۱۲۳۴۵۶۷۸۹');
  });

  it('is off by default', () => {
    expect(printDigits('12,000')).toBe('12,000');
    expect(printDigits('12,000', 'latin')).toBe('12,000');
    expect(printDigits('12,000', 'urdu')).toBe('۱۲,۰۰۰');
  });
});

describe('formatPaisaValue', () => {
  // The amounts inside the values of a backend error arrive as plain numbers (plan 12 contract 4).
  it.each(cases)('%d paisa gives the same text as formatPaisa', (paisa, international, lakh) => {
    expect(formatPaisaValue(paisa, { grouping: 'international' })).toBe(international);
    expect(formatPaisaValue(paisa, { grouping: 'lakh' })).toBe(lakh);
  });
});
