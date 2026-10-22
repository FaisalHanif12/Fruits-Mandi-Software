// The columns of the list of trip goods left, under the lots in باقی مال (PRD FR-12.25): the number
// of the purchase in its own series, its trip, the item and packing, what is left and its cost, and
// in the wider steps the supplier (or the market), the days waiting and the quantities bought and
// sold. The room beside the 200 px side menu is 692 px at the narrowest compact window, 874 at the
// narrowest normal one and 1,332 at the narrowest wide one (tests/unit/list-widths.test.ts), so the
// list shows the columns that fit; the card of the row that has focus shows all of them, with the
// quantity written off.
// Nothing scrolls sideways (NFR-USE-15). Every figure is the backend's; nothing is worked out here.

import type { OwnStockRow } from '../../bindings/OwnStockRow';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { purchaseNumberText, supplierText } from '../purchases';
import { tripNumberText } from '../trips';

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
    value: (row, { text }) => tripNumberText(row.trip.number, text),
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
  {
    field: 'balance',
    type: 'number',
    headerId: 'bikri-col-balance',
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
    from: 'normal',
    value: (row, { money }) => money(row.cost_left_paisa),
  },
  {
    field: 'days',
    type: 'code',
    headerId: 'bikri-stock-col-days',
    from: 'wide',
    value: (row) => String(row.days_waiting),
  },
  {
    field: 'bought',
    type: 'number',
    headerId: 'purchases-col-qty',
    from: 'wide',
    value: (row) => String(row.qty_bought),
  },
  {
    field: 'sold',
    type: 'number',
    headerId: 'bikri-saved-qty-sold',
    from: 'wide',
    value: (row) => String(row.qty_sold),
  },
];
