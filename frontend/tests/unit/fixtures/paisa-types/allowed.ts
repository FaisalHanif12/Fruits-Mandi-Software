// The ways Paisa may be used outside money.ts. This file must type-check with no error.
import { comparePaisa, type Paisa } from '../../../../src/shared/format/money';

declare const a: Paisa;
declare const b: Paisa;
declare const list: Paisa[];

export const order = comparePaisa(a, b);
export const sorted = [...list].sort(comparePaisa);
export const same = a === b;
export const row: { total: Paisa } = { total: a };
export function passOn(amount: Paisa): Paisa {
  return amount;
}
