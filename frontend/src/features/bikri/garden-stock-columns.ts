// The columns of the list of garden goods left, under the lots in باقی مال (PRD FR-12.40): the
// number of the goods arrived entry in its own series, its garden, the item and packing, what is
// left, and in the wider steps the day to sell, the days waiting and the quantities arrived and
// sold. The list shows the columns that fit; the card of the row that has focus shows all of them,
// with the quantity written off. Nothing scrolls sideways (NFR-USE-15). Every figure is the
// backend's; nothing is worked out here.

import type { GardenStockRow } from '../../bindings/GardenStockRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { arrivalNumberText } from '../gardens';

export const GARDEN_STOCK_COLUMNS: readonly ListColumn<GardenStockRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'bikri-stock-garden-col-number',
    value: (row, { text }) => arrivalNumberText(row.number, text),
  },
  {
    field: 'garden',
    type: 'code',
    headerId: 'bikri-stock-garden-col-garden',
    value: (row) => row.garden.code,
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'bikri-stock-col-item',
    grow: true,
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
    field: 'sell_date',
    type: 'date',
    headerId: 'purchases-col-sell-date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.sell_date),
  },
  {
    field: 'days',
    type: 'code',
    headerId: 'bikri-stock-col-days',
    from: 'wide',
    value: (row) => String(row.days_waiting),
  },
  {
    field: 'arrived',
    type: 'number',
    headerId: 'bikri-stock-garden-col-arrived',
    from: 'wide',
    value: (row) => String(row.qty_arrived),
  },
  {
    field: 'sold',
    type: 'number',
    headerId: 'bikri-saved-qty-sold',
    from: 'wide',
    value: (row) => String(row.qty_sold),
  },
];
