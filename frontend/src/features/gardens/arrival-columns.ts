// The columns of the two lists of a garden's goods arrived entries (PRD FR-12.40, FR-12.53): "not
// sold yet" says what arrived, what has sold so far, what is left for our shop and for another shop,
// the days it has waited and its day to sell; "sold" says what arrived, what sold in our shop and in
// another shop, what was written off, the sale value, the commission and the day of the last sale.
// The number is in the entry series (GA-1). The item column grows; the less used columns come with the
// wider steps, so neither list scrolls sideways (NFR-USE-15). A cancelled entry says so in the item
// cell, in words. The details under the list hold every other figure. A click on a heading with an
// order sorts the whole list (the backend sorts, plan 38 R4). Every figure is the backend's; nothing
// is worked out here.

import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { arrivalNumberText } from './list-columns';
import type { ArrivalRow } from './types';

const NUMBER: ListColumn<ArrivalRow> = {
  field: 'number',
  type: 'code',
  headerId: 'gardens-arrival-col-number',
  sortKey: 'number',
  value: (row, { text }) => arrivalNumberText(row.number, text),
};

const ITEM: ListColumn<ArrivalRow> = {
  field: 'item',
  type: 'text',
  headerId: 'gardens-arrival-col-item',
  grow: true,
  sortKey: 'item',
  value: (row, { language, text }) => {
    const words = text('gardens-arrival-item-packing', {
      item: displayName(row.item, language),
      packing: displayName(row.packing, language),
    });
    return row.status === 'cancelled'
      ? text('gardens-arrival-item-cancelled', { item: words })
      : words;
  },
};

const ARRIVED: ListColumn<ArrivalRow> = {
  field: 'qty',
  type: 'count',
  headerId: 'gardens-arrival-col-qty',
  sortKey: 'qty',
  from: 'normal',
  value: (row) => String(row.qty),
};

/** The shop's commission on this entry's goods sold in our shop (PRD FR-12.37), in each line. */
const COMMISSION: ListColumn<ArrivalRow> = {
  field: 'commission',
  type: 'figure',
  headerId: 'glossary-commission',
  value: (row, { money }) => money(row.commission_paisa),
};

/** The day the goods arrived: the wide step has room for it. */
const ARRIVAL_DAY: ListColumn<ArrivalRow> = {
  field: 'date',
  type: 'date',
  headerId: 'gardens-arrival-col-date',
  sortKey: 'entry_date',
  from: 'wide',
  value: (row) => isoToDisplayDate(row.entry_date),
};

const count = (
  field: string,
  headerId: string,
  value: (row: ArrivalRow) => number,
  sortKey?: string,
): ListColumn<ArrivalRow> => ({
  field,
  type: 'count',
  headerId,
  ...(sortKey === undefined ? {} : { sortKey }),
  value: (row) => String(value(row)),
});

/** The goods not sold yet (PRD FR-12.53, "بکنا باقی"). */
export const ARRIVAL_IN_STOCK_COLUMNS: readonly ListColumn<ArrivalRow>[] = [
  NUMBER,
  ITEM,
  ARRIVED,
  count('qty_sold', 'gardens-arrival-col-sold', (row) => row.qty_sold, 'qty_sold'),
  // The goods left, by part (PRD FR-12.41): what is for our shop and what is for another shop.
  count('qty_our_left', 'purchases-col-our-left', (row) => row.qty_our_left),
  count('qty_other_left', 'purchases-col-other-left', (row) => row.qty_other_left),
  count('days_waiting', 'bikri-stock-col-days', (row) => row.days_waiting),
  {
    field: 'sell_date',
    type: 'date',
    headerId: 'purchases-col-sell-date',
    sortKey: 'sell_date',
    value: (row) => isoToDisplayDate(row.sell_date),
  },
  { ...COMMISSION, from: 'wide' },
  ARRIVAL_DAY,
];

/** The goods sold (PRD FR-12.53, "بک چکا"). */
export const ARRIVAL_SOLD_COLUMNS: readonly ListColumn<ArrivalRow>[] = [
  NUMBER,
  ITEM,
  ARRIVED,
  count('qty_our_sold', 'gardens-arrival-col-our-sold', (row) => row.qty_our_sold),
  count('qty_other_sold', 'gardens-arrival-col-other-sold', (row) => row.qty_other_sold),
  count(
    'qty_written_off',
    'bikri-stock-col-written-off',
    (row) => row.qty_written_off,
    'qty_written_off',
  ),
  {
    field: 'sale_value',
    type: 'figure',
    headerId: 'gardens-arrival-col-sale-value',
    value: (row, { money }) => money(row.sale_value_paisa),
  },
  COMMISSION,
  {
    field: 'last_sale_date',
    type: 'date',
    headerId: 'gardens-arrival-col-last-sale',
    from: 'normal',
    value: (row) => (row.last_sale_date === null ? '' : isoToDisplayDate(row.last_sale_date)),
  },
  ARRIVAL_DAY,
];
