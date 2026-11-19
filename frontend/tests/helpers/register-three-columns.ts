// A sales register (R2) with all three optional columns the backend can add: the quality (`grade`,
// PRD FR-12.42), the packing (when the filter has more than one, PRD FR-7.53) and the seller's rate
// (`seller_rate`, when a line has a rate difference, PRD FR-12.3). The backend's recorded answers
// never have the three together, so this page is built from the recorded R2 with the quality, in the
// order the backend writes the columns (backend/crates/reports/src/reports/sales/register.rs, plan 59):
// date, entry, buyer, city, item, packing, quality, quantity, rate, seller's rate, value, buyer's total.
// When the rate has a seller's rate beside it, the backend gives it the heading "Rate, Buyer". Replace
// it with a recorded answer when the backend writes one. Money is paisa.

import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportColumn } from '../../src/bindings/ReportColumn';
import type { ReportPage } from '../../src/bindings/ReportPage';
import { paisaForTest } from './paisa';
import { GRADE_R2 } from './sale-grade-real';

const PACKING: ReportCell = { type: 'names', names: { ur: 'پیٹی', en: 'Box' } };

/** The seller's rate of each line, in paisa: lower than the buyer's rate, so the line has a rate difference. */
const SELLER_RATES_PAISA = [115000, 78000, 48000] as const;

const PACKING_COLUMN: ReportColumn = {
  key: 'packing',
  title_id: 'report-col-packing',
  kind: 'names',
};
const SELLER_RATE_COLUMN: ReportColumn = {
  key: 'seller_rate',
  title_id: 'report-col-seller-rate',
  kind: 'money',
};

/** Puts a column after the one with `after`, in the column list and in every row and the totals. */
function insertAfter(
  page: ReportPage,
  after: string,
  column: ReportColumn,
  cellFor: (rowIndex: number) => ReportCell,
): ReportPage {
  const at = page.columns.findIndex((candidate) => candidate.key === after) + 1;
  const put = <T>(list: readonly T[], item: T): T[] => [
    ...list.slice(0, at),
    item,
    ...list.slice(at),
  ];
  return {
    ...page,
    columns: put(page.columns, column),
    rows: page.rows.map((row, index) => ({ ...row, cells: put(row.cells, cellFor(index)) })),
    totals: put(page.totals, { type: 'empty' }),
  };
}

/** R2 with the packing after the item, the quality, and the seller's rate after the rate. */
export function registerWithThreeColumns(): ReportPage {
  const withPacking = insertAfter(GRADE_R2, 'item', PACKING_COLUMN, () => PACKING);
  const withSellerRate = insertAfter(withPacking, 'rate', SELLER_RATE_COLUMN, (index) => ({
    type: 'money',
    paisa: paisaForTest(SELLER_RATES_PAISA[index] ?? 0),
  }));
  return {
    ...withSellerRate,
    columns: withSellerRate.columns.map((column) =>
      column.key === 'rate' ? { ...column, title_id: 'report-col-buyer-rate' } : column,
    ),
  };
}

export const R2_THREE_COLUMNS: ReportPage = registerWithThreeColumns();
