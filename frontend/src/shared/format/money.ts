// Money on screen (PRD Section 22, Code Standards F3 and M12).
//
// The backend owns all money. It sends whole-rupee amounts as integer paisa (1 rupee = 100 paisa),
// for example 1,200,000 paisa for 12,000 rupees. The frontend only shows them.
//
// `Paisa` is OPAQUE: it is not a number type, so TypeScript itself rejects every way of doing
// maths with it outside this file. `const n: number = amount`, `amount * 2`, `amount + 1`,
// `Math.round(amount)` and passing it to a `number` parameter all fail to compile
// (tests/unit/paisa-types.test.ts). At run time the value is still the JSON number the backend
// sent; only this file looks inside it. The lint rule mandi/no-paisa-arithmetic is a second
// layer that also blocks conversions TypeScript allows, such as Number(amount) or `${amount}`.
//
// This file is the only place that unwraps Paisa: comparePaisa for sorting (allowed by F3), and
// formatPaisa for display (plan 02, contract 4.7). formatPaisa never divides: it works on the
// digits as text, so it can never make a decimal.

declare const paisaBrand: unique symbol;

/** An amount of money in paisa, as sent by the backend. Always a whole number of rupees. */
export interface Paisa {
  readonly [paisaBrand]: 'Paisa';
}

/** The number inside. Private to this file: everything else works through the functions here. */
function paisaNumber(amount: Paisa): number {
  return amount as unknown as number;
}

/**
 * Sort order of two amounts, for `Array.prototype.sort`: negative when `a` is smaller, 0 when
 * equal, positive when larger. Example: `[300, 100]` as Paisa sorts to `[100, 300]`.
 */
export function comparePaisa(a: Paisa, b: Paisa): number {
  const x = paisaNumber(a);
  const y = paisaNumber(b);
  if (x < y) return -1;
  if (x > y) return 1;
  return 0;
}

/**
 * True when the amount is more than zero. A question about an amount the backend sent (does this
 * receipt settle anything of this bill?), never arithmetic: the screen marks a row in words and
 * works nothing out.
 */
export function isPositivePaisa(amount: Paisa): boolean {
  return paisaNumber(amount) > 0;
}

/**
 * True when the amount is below zero. A question about an amount the backend sent (is this trip
 * making a loss?), never arithmetic: the screen colours it red and also says so in words.
 */
export function isNegativePaisa(amount: Paisa): boolean {
  return paisaNumber(amount) < 0;
}

/** True when the amount is exactly zero: a row or a note that has nothing to say is left out. */
export function isZeroPaisa(amount: Paisa): boolean {
  return paisaNumber(amount) === 0;
}

/** Number grouping on screen (PRD Section 21): 1,316,040 or 13,16,040. */
export type Grouping = 'international' | 'lakh';

export interface FormatPaisaOptions {
  grouping: Grouping;
}

/** Shown in place of an amount the backend should never send (production only). */
const INVALID_AMOUNT_TEXT = '—';
const PAISA_DIGITS = 2;
const GROUP_SIZE = 3;
const LAKH_GROUP_SIZE = 2;

/**
 * Puts the group separators into a string of digits, from the end. International groups by
 * three: 1316040 -> 1,316,040. Lakh groups the last three, then by two: 1316040 -> 13,16,040.
 */
export function groupDigits(digits: string, grouping: Grouping): string {
  if (digits.length <= GROUP_SIZE) return digits;
  const last = digits.slice(-GROUP_SIZE);
  const head = digits.slice(0, -GROUP_SIZE);
  const size = grouping === 'lakh' ? LAKH_GROUP_SIZE : GROUP_SIZE;
  const groups: string[] = [];
  let end = head.length;
  while (end > 0) {
    const start = Math.max(0, end - size);
    groups.unshift(head.slice(start, end));
    end = start;
  }
  return [...groups, last].join(',');
}

/**
 * An amount the backend broke the money rule on (M1). In development this is a bug to fix at
 * once, so it throws. In production the screen must stay up, so it logs and shows a dash.
 */
function invalidAmount(amount: number): string {
  const message = `formatPaisa: ${String(amount)} is not a whole number of rupees in paisa`;
  if (import.meta.env.DEV) throw new Error(message);
  console.error(message);
  return INVALID_AMOUNT_TEXT;
}

/**
 * The text of an amount, in whole rupees with group separators (Code Standards M12).
 * Examples: 1,200,000 paisa -> "12,000"; 131,604,000 paisa -> "1,316,040", or "13,16,040" with
 * lakh grouping; 4,500 paisa -> "45"; 0 -> "0". A negative amount gets a minus sign, for
 * technical views only (business screens show the بنام or جمع word instead).
 */
export function formatPaisa(amount: Paisa, options: FormatPaisaOptions): string {
  const value = paisaNumber(amount);
  if (!Number.isSafeInteger(value) || value % 100 !== 0) return invalidAmount(value);
  // A safe integer prints as plain digits, never in exponent form.
  const text = String(value);
  const negative = text.startsWith('-');
  const digits = negative ? text.slice(1) : text;
  const rupees = digits === '0' ? '0' : digits.slice(0, -PAISA_DIGITS);
  const grouped = groupDigits(rupees, options.grouping);
  return negative ? `-${grouped}` : grouped;
}

/**
 * The text of an amount that arrived as a plain JSON number inside the values of a backend error
 * or warning (a name that ends in `_paisa`, for example `cash_after_paisa`; plan 12 contract 4).
 * It is the same text as formatPaisa gives for the same amount. It lives here because this file
 * is the only one that looks inside a Paisa (Code Standards F3).
 */
export function formatPaisaValue(value: number, options: FormatPaisaOptions): string {
  return formatPaisa(value as unknown as Paisa, options);
}
