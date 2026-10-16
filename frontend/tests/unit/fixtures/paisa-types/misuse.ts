// Each line marked "expect-error" must fail to type-check (reviewer recheck finding A).
// tests/unit/paisa-types.test.ts compiles this file and checks every mark.
// TypeScript accepts unary minus (`-a`) on any type; the lint rule mandi/no-paisa-arithmetic
// rejects it (tests/unit/eslint-rules.test.ts). `a < b` also compiles: comparing for a sort is
// allowed by Code Standards F3, and comparePaisa is the clearer way.
import type { Paisa } from '../../../../src/shared/format/money';

declare const a: Paisa;
declare const b: Paisa;
declare function double(n: number): number;
declare function show(text: string): void;

export const widened: number = a; // expect-error: widen to number
export const doubled = a * 2; // expect-error: multiply
export const plusOne = a + 1; // expect-error: add a number
export const sum = a + b; // expect-error: add two amounts
export const minus = a - b; // expect-error: subtract
export const passed = double(a); // expect-error: pass to a number parameter
export const rounded = Math.round(a); // expect-error: Math.round
export const fixed = a.toFixed(0); // expect-error: toFixed
export function giveBack(): number {
  return a; // expect-error: return as number
}
export const fromNumber: Paisa = 1200000; // expect-error: a plain number is not money
show(a); // expect-error: pass to a string parameter
