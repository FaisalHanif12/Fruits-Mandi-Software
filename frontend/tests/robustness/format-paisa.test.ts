// Robustness: formatPaisa on hostile values (plan 02 T6, money agent 3.2, Code Standards M12).
//
// Rules checked:
// - A value that is not a safe integer multiple of 100 is never shown as money: it throws in
//   development and shows a dash (and logs once) in production. No "NaN", "Infinity", "1e+21",
//   "-0", decimals or paisa ever reach the screen.
// - Every valid value is shown exactly: the text equals a BigInt reference worked out with no
//   floating point at all.
// The values come in through tests/helpers/paisa.ts, the way a broken backend reply would.

import { afterEach, describe, expect, it, vi } from 'vitest';
import { formatPaisa, type Grouping } from '../../src/shared/format/money';
import { paisaForTest } from '../helpers/paisa';

const GROUPINGS: readonly Grouping[] = ['international', 'lakh'];
const DASH = '—';

/** Reference with BigInt only: whole rupees, grouped by hand. */
function reference(paisa: bigint, grouping: Grouping): string {
  const negative = paisa < 0n;
  const digits = ((negative ? -paisa : paisa) / 100n).toString();
  const last = digits.slice(-3);
  let head = digits.slice(0, -3);
  const size = grouping === 'lakh' ? 2 : 3;
  const groups: string[] = [];
  while (head.length > 0) {
    groups.unshift(head.slice(-size));
    head = head.slice(0, -size);
  }
  const text = [...groups, last].join(',');
  return negative ? `-${text}` : text;
}

/** Values a broken reply could carry. `paisaForTest` takes a number, so others are cast. */
const hostile: readonly (readonly [label: string, value: unknown])[] = [
  ['1 paisa', 1],
  ['99 paisa', 99],
  ['101 paisa', 101],
  ['-1 paisa', -1],
  ['-150 paisa', -150],
  ['half a paisa', 0.5],
  ['100.000001', 100.000001],
  ['MAX_SAFE_INTEGER', Number.MAX_SAFE_INTEGER],
  ['MIN_SAFE_INTEGER', Number.MIN_SAFE_INTEGER],
  ['2^53 + 8 (unsafe)', 2 ** 53 + 8],
  ['1e17 (unsafe multiple of 100)', 1e17],
  ['-1e17', -1e17],
  ['1e21', 1e21],
  ['MAX_VALUE', Number.MAX_VALUE],
  ['NaN', Number.NaN],
  ['Infinity', Infinity],
  ['-Infinity', -Infinity],
  ['string "1200"', '1200'],
  ['string "abc"', 'abc'],
  ['empty string', ''],
  ['null', null],
  ['undefined', undefined],
  ['bigint', 120_000n],
  ['object', { value: 1200 }],
  ['array', [1200]],
  ['boolean', true],
];

const asPaisa = (value: unknown) => paisaForTest(value as number);

describe('formatPaisa refuses values that are not whole rupees in paisa', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.restoreAllMocks();
  });

  it.each(hostile)('throws in development on %s', (_label, value) => {
    for (const grouping of GROUPINGS) {
      expect(() => formatPaisa(asPaisa(value), { grouping })).toThrow(/not a whole number/);
    }
  });

  it.each(hostile)('shows a dash and logs once in production on %s', (_label, value) => {
    vi.stubEnv('DEV', false);
    const log = vi.spyOn(console, 'error').mockImplementation(() => undefined);
    expect(formatPaisa(asPaisa(value), { grouping: 'lakh' })).toBe(DASH);
    expect(log).toHaveBeenCalledOnce();
  });
});

describe('formatPaisa shows every valid value exactly', () => {
  it('negative zero is "0", never "-0"', () => {
    for (const grouping of GROUPINGS) {
      expect(formatPaisa(paisaForTest(-0), { grouping })).toBe('0');
    }
  });

  it('the largest safe multiple of 100 and its negative', () => {
    const top = Number.MAX_SAFE_INTEGER - (Number.MAX_SAFE_INTEGER % 100);
    expect(formatPaisa(paisaForTest(top), { grouping: 'international' })).toBe(
      '90,071,992,547,409',
    );
    expect(formatPaisa(paisaForTest(-top), { grouping: 'lakh' })).toBe('-9,00,71,99,25,47,409');
  });

  it('the input maximum, 10,00,00,00,000 rupees, and 1 rupee', () => {
    expect(formatPaisa(paisaForTest(1e12), { grouping: 'lakh' })).toBe('10,00,00,00,000');
    expect(formatPaisa(paisaForTest(100), { grouping: 'international' })).toBe('1');
  });

  it('matches a BigInt reference for 20,000 values across the whole safe range', () => {
    // A fixed linear congruential sequence, so every run checks the same values.
    let state = 0x5eed_0002n;
    const next = (): bigint => {
      state = (state * 6364136223846793005n + 1442695040888963407n) % 2n ** 64n;
      return state;
    };
    const maxRupees = BigInt(Number.MAX_SAFE_INTEGER) / 100n;
    for (let i = 0; i < 20_000; i += 1) {
      // Mix small and huge amounts: the digit count is random too.
      const digits = Number(next() % 17n) + 1;
      const rupees = (next() % 10n ** BigInt(digits)) % (maxRupees + 1n);
      const paisa = (next() % 2n === 0n ? 1n : -1n) * rupees * 100n;
      for (const grouping of GROUPINGS) {
        const text = formatPaisa(paisaForTest(Number(paisa)), { grouping });
        expect(text).toBe(reference(paisa, grouping));
        expect(text).toMatch(/^-?\d{1,3}(,\d{2,3})*$/);
      }
    }
  });
});
