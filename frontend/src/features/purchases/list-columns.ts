// The columns of the list of a trip's purchases (PRD FR-12.25, FR-12.27): the number in the own
// series (خ-5, PUR-5), the item and packing, what is left, and, in the wider steps, the cost left,
// the result and what was bought, sold and written off. Amounts and counts are the backend's and
// shown with `money` and `String`, never worked out. The list sits under the main menu only, so
// the room is the content area minus the 64 px rail; less used columns come with the wider layout
// steps and the list never scrolls sideways (NFR-USE-15, Code Standards F22). A cancelled
// purchase says so in the item cell, in words.

import type { PurchaseRow } from '../../bindings/PurchaseRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';

/** The number of a purchase in its series: "خ-5" or "PUR-5" (letters from Fluent). */
export function purchaseNumberText(
  number: number,
  text: (id: string, vars?: Record<string, string>) => string,
): string {
  return text('purchases-number', { number: String(number) });
}

/** Who the goods were bought from, as typed, or the word for "not named". */
export function supplierText(
  supplier: string | null,
  text: (id: string, vars?: Record<string, string>) => string,
): string {
  return supplier === null || supplier.trim() === '' ? text('purchases-market') : supplier;
}

export const TRIP_PURCHASE_COLUMNS: readonly ListColumn<PurchaseRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'purchases-col-number',
    value: (row, { text }) => purchaseNumberText(row.number, text),
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'purchases-col-item',
    grow: true,
    value: (row, { language, text }) => {
      const words = text('purchases-item-packing', {
        item: displayName(row.item, language),
        packing: displayName(row.packing, language),
      });
      return row.status === 'cancelled' ? text('purchases-item-cancelled', { item: words }) : words;
    },
  },
  {
    field: 'qty_left',
    type: 'number',
    headerId: 'purchases-col-left',
    value: (row) => String(row.qty_left),
  },
  {
    field: 'cost_left',
    type: 'money',
    headerId: 'purchases-col-cost-left',
    from: 'normal',
    value: (row, { money }) => money(row.cost_left_paisa),
  },
  {
    field: 'result',
    type: 'money',
    headerId: 'purchases-col-result',
    from: 'normal',
    value: (row, { money }) => money(row.result_paisa),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'purchases-col-date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'sell_date',
    type: 'date',
    headerId: 'purchases-col-sell-date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.sell_date),
  },
  {
    field: 'qty',
    type: 'number',
    headerId: 'purchases-col-qty',
    from: 'wide',
    value: (row) => String(row.qty),
  },
  {
    field: 'total_cost',
    type: 'money',
    headerId: 'purchases-col-total-cost',
    from: 'wide',
    value: (row, { money }) => money(row.total_cost_paisa),
  },
];
