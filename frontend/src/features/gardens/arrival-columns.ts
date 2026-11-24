// The columns of the list of a garden's goods arrived entries (PRD FR-12.40): the number in the
// entry series (GA-1), the item and packing, and what is left (for our shop, for another shop and in
// all); at the wide step also the commission, the day to sell and the day it arrived. A cancelled
// entry says so in the item cell, in words. The panel of the entry that has focus holds every other
// figure (plan 96), and the list is narrow enough for the panel beside it, so it never scrolls
// sideways (NFR-USE-15): 5 columns take 660 px of the 730 a 1366 px window leaves. A click on a heading sorts the whole list (the backend sorts, plan
// 38 R4); the list's own order is the newest first (plan 96). Every figure is the backend's; nothing
// is worked out here.

import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { arrivalNumberText } from './list-columns';
import type { ArrivalRow } from './types';

export const ARRIVAL_COLUMNS: readonly ListColumn<ArrivalRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'gardens-arrival-col-number',
    sortKey: 'number',
    value: (row, { text }) => arrivalNumberText(row.number, text),
  },
  {
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
    field: 'qty_left',
    type: 'number',
    headerId: 'gardens-arrival-col-left',
    sortKey: 'qty_left',
    value: (row) => String(row.qty_left),
  },
  // The shop's commission on this entry's goods sold in our shop (PRD FR-12.37).
  {
    field: 'commission',
    type: 'money',
    headerId: 'glossary-commission',
    from: 'wide',
    value: (row, { money }) => money(row.commission_paisa),
  },
  {
    field: 'sell_date',
    type: 'date',
    headerId: 'purchases-col-sell-date',
    sortKey: 'sell_date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.sell_date),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'gardens-arrival-col-date',
    sortKey: 'entry_date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
];
