// When a statement has nothing to print (plan 74 T3, PRD FR-12.47, Umar 13-11-2026): a period with no entry
// AND no balance carried through it. A period with no entry but a balance (he still owes 1,000) is printed as
// a one-line statement: the opening balance with its side, no day blocks, the closing equal to it, and its
// amount in words. The side of a balance is `none` at 0, so this reads only what the backend wrote.

import type { StatementPrintData } from '../../../bindings/StatementPrintData';

/** True when the period has no date with an entry and no balance (the opening is 0, so the closing is too). */
export function hasNothingToPrint(data: Pick<StatementPrintData, 'days' | 'opening'>): boolean {
  return data.days.length === 0 && data.opening.side === 'none';
}
