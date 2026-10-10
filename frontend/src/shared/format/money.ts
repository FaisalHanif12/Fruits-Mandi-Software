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
// formatPaisa for display, which arrives in plan 02.

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
