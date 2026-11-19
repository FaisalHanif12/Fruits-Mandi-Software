// The columns of the list of trip goods left, under the lots in باقی مال (PRD FR-12.25): the number
// of the purchase in its own series, its trip, the item and packing, what is left for our shop and
// for another shop (PRD FR-12.41) and its cost, and in the wider steps the supplier (or the market)
// and the day to sell. The days waiting and the quantities bought and sold are in the card. The
// room beside the 200 px side menu is 692 px at the narrowest compact window, 874 at the narrowest
// normal one and 1,332 at the narrowest wide one (tests/unit/list-widths.test.ts), so the list
// shows the columns that fit; the card of the row that has focus shows all of them, with the
// quantity written off.
// Nothing scrolls sideways (NFR-USE-15). Every figure is the backend's; nothing is worked out here.

import type { OwnStockRow } from '../../bindings/OwnStockRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { purchaseNumberText, supplierText } from '../purchases';

export const OWN_STOCK_COLUMNS: readonly ListColumn<OwnStockRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'purchases-col-number',
    value: (row, { text }) => purchaseNumberText(row.number, text),
  },
  {
    field: 'trip',
    type: 'code',
    headerId: 'trips-col-trip',
    from: 'normal',
    value: (row) => row.trip.code,
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'bikri-stock-col-item',
    value: (row, { language, text }) =>
      text('bikri-stock-item-packing', {
        item: displayName(row.item, language),
        packing: displayName(row.packing, language),
      }),
  },
  // The goods left, by part (PRD FR-12.41): what is for our shop and what is for another shop.
  {
    field: 'qty_our_left',
    type: 'figure',
    headerId: 'purchases-col-our-left',
    value: (row) => String(row.qty_our_left),
  },
  {
    field: 'qty_other_left',
    type: 'figure',
    headerId: 'purchases-col-other-left',
    value: (row) => String(row.qty_other_left),
  },
  {
    field: 'balance',
    type: 'number',
    headerId: 'bikri-col-balance',
    from: 'normal',
    value: (row) => String(row.balance),
  },
  {
    field: 'supplier',
    type: 'name',
    headerId: 'purchases-col-supplier',
    from: 'wide',
    grow: true,
    // The supplier is free text, shown as typed (PRD FR-7.25).
    value: (row, { text }) => supplierText(row.supplier, text),
  },
  {
    field: 'cost_left',
    type: 'money',
    headerId: 'purchases-col-cost-left',
    from: 'wide',
    value: (row, { money }) => money(row.cost_left_paisa),
  },
  {
    field: 'sell_date',
    type: 'date',
    headerId: 'purchases-col-sell-date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.sell_date),
  },
];
