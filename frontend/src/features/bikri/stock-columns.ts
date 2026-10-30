// The columns of the list of goods left (PRD FR-12.14): date received, بکری number, seller, item
// and packing, received, sold, written off, balance and days waiting. The room beside the 200 px
// side menu is 692 px at the narrowest compact window, 874 at the narrowest normal one and 1,332 at
// the narrowest wide one (tests/unit/list-widths.test.ts), so the list shows the columns that
// fit; the card of the lot under the list shows all of them for the row that has focus. Nothing
// scrolls sideways (NFR-USE-15). Every figure is the backend's; nothing is worked out here. The
// four quantity columns (received, sold, written off, taken as fruit) are `count` columns of 100 px,
// so the ten columns of the wide step fit its 1,332 px with the column for the fruit taken (PRD
// FR-16.11) that was added after the first widths were worked out.

import type { StockRow } from '../../bindings/StockRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { PUKHTA_SELLER_CODE } from './types';

/** The seller of a lot. A one-time seller is shown by his own name (PRD FR-13.2). */
export function stockSellerName(
  row: StockRow,
  language: Parameters<typeof displayName>[1],
): string {
  return row.seller.code === PUKHTA_SELLER_CODE && row.pukhta !== null
    ? displayName(row.pukhta, language)
    : displayName(row.seller, language);
}

export const STOCK_COLUMNS: readonly ListColumn<StockRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'bikri-list-col-number',
    sortKey: 'number',
    value: (row) => String(row.bikri_number),
  },
  {
    field: 'received_date',
    type: 'date',
    headerId: 'bikri-stock-col-received-date',
    sortKey: 'received_date',
    from: 'normal',
    value: (row) => isoToDisplayDate(row.received_date),
  },
  {
    field: 'seller',
    type: 'name',
    headerId: 'glossary-beopari',
    sortKey: 'seller',
    grow: true,
    value: (row, { language }) => stockSellerName(row, language),
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'bikri-stock-col-item',
    sortKey: 'item',
    value: (row, { language, text }) =>
      text('bikri-stock-item-packing', {
        item: displayName(row.item, language),
        packing: displayName(row.packing, language),
      }),
  },
  {
    field: 'received',
    type: 'count',
    headerId: 'bikri-col-qty-received',
    sortKey: 'qty_received',
    from: 'wide',
    value: (row) => String(row.qty_received),
  },
  {
    field: 'sold',
    type: 'count',
    headerId: 'bikri-saved-qty-sold',
    sortKey: 'qty_sold',
    from: 'wide',
    value: (row) => String(row.qty_sold),
  },
  {
    field: 'written_off',
    type: 'count',
    headerId: 'bikri-stock-col-written-off',
    sortKey: 'qty_written_off',
    from: 'wide',
    value: (row) => String(row.qty_written_off),
  },
  {
    field: 'taken',
    type: 'count',
    headerId: 'bikri-stock-col-taken',
    from: 'wide',
    value: (row) => String(row.qty_taken),
  },
  {
    field: 'balance',
    type: 'number',
    headerId: 'bikri-col-balance',
    sortKey: 'balance',
    value: (row) => String(row.balance),
  },
  {
    field: 'days',
    type: 'code',
    headerId: 'bikri-stock-col-days',
    sortKey: 'days_waiting',
    from: 'wide',
    value: (row) => String(row.days_waiting),
  },
];
