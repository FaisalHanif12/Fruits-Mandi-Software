// The marks of earlier edits on the cells of a sale book open for edit (plan 44, PRD FR-7.71): the
// history names a field by its stored path (`lots[0].sales[1].seller_rate_paisa`); the lots and the
// lines of the page are in the same order as the posting's, so a lot row is `lots[i]` and its line
// row j is `lots[i].sales[j]`. This file turns the marks of the entry into the cells of the two
// grids. Display only: the tip is the words the history gave, never worked out here.

import type { EntryMarks } from '../../shared/edit';
import { LOT, SALE } from './types';

type PerRow = Record<string, Record<string, string>>;

/** The cells of a grid that carry a mark, or undefined when none does (so the grid reads nothing). */
function orNone(cells: PerRow): PerRow | undefined {
  return Object.keys(cells).length === 0 ? undefined : cells;
}

/** The stored paths a grid field can have changed, the first that has a mark wins. */
const LOT_PATHS: Readonly<Record<string, readonly string[]>> = {
  [LOT.seller]: ['seller_account_id'],
  [LOT.item]: ['item_id'],
  [LOT.packing]: ['packing_id'],
  [LOT.qty]: ['qty_received'],
  [LOT.freight]: ['freight_paisa'],
  [LOT.labour]: ['labour.value_paisa', 'labour.amount_paisa'],
  [LOT.munshiana]: ['munshiana.value_paisa', 'munshiana.amount_paisa'],
  [LOT.cash]: ['cash_paid_paisa'],
};

const SALE_PATHS: Readonly<Record<string, readonly string[]>> = {
  [SALE.buyer]: ['buyer_account_id'],
  [SALE.qty]: ['qty'],
  [SALE.rate]: ['seller_rate_paisa'],
  [SALE.grade]: ['grade'],
  [SALE.paid]: ['paid_paisa'],
};

function cellsOf(
  marks: EntryMarks,
  prefix: string,
  paths: Readonly<Record<string, readonly string[]>>,
): Record<string, string> {
  const cells: Record<string, string> = {};
  for (const [field, candidates] of Object.entries(paths)) {
    for (const path of candidates) {
      const mark = marks.of(`${prefix}${path}`);
      if (mark !== undefined) {
        cells[field] = mark.tip;
        break;
      }
    }
  }
  return cells;
}

/** The marked cells of the lots grid, by lot row. `lotKeys` are the rows in the order of the lots. */
export function lotMarkCells(marks: EntryMarks, lotKeys: readonly string[]): PerRow | undefined {
  const out: PerRow = {};
  if (!marks.any) return undefined;
  lotKeys.forEach((key, lot) => {
    const cells = cellsOf(marks, `lots[${String(lot)}].`, LOT_PATHS);
    if (Object.keys(cells).length > 0) out[key] = cells;
  });
  return orNone(out);
}

/** The marked cells of the sale lines grid, by line row. `linesByLot[i]` are the lines of lot i. */
export function saleMarkCells(
  marks: EntryMarks,
  linesByLot: readonly (readonly string[])[],
): PerRow | undefined {
  const out: PerRow = {};
  if (!marks.any) return undefined;
  linesByLot.forEach((lines, lot) => {
    lines.forEach((key, line) => {
      const cells = cellsOf(marks, `lots[${String(lot)}].sales[${String(line)}].`, SALE_PATHS);
      if (Object.keys(cells).length > 0) out[key] = cells;
    });
  });
  return orNone(out);
}
