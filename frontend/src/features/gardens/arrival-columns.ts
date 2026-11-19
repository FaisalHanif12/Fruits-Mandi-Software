// The columns of the list of a garden's goods arrived entries (PRD FR-12.40): the number in the
// entry series (GA-1), the item and packing, what is left, and in the wider steps the day to sell,
// the day it arrived and the quantities that arrived and were sold. A cancelled entry says so in the
// item cell, in words. The card of the entry that has focus holds every figure, so the list never
// scrolls sideways (NFR-USE-15). Every figure is the backend's; nothing is worked out here.

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
    value: (row, { text }) => arrivalNumberText(row.number, text),
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'gardens-arrival-col-item',
    grow: true,
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
    from: 'normal',
    value: (row) => String(row.qty_left),
  },
  // The shop's commission on this entry's goods sold in our shop (PRD FR-12.37).
  {
    field: 'commission',
    type: 'money',
    headerId: 'glossary-commission',
    from: 'normal',
    value: (row, { money }) => money(row.commission_paisa),
  },
  {
    field: 'sell_date',
    type: 'date',
    headerId: 'purchases-col-sell-date',
    from: 'normal',
    value: (row) => isoToDisplayDate(row.sell_date),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'gardens-arrival-col-date',
    from: 'wide',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'qty',
    type: 'number',
    headerId: 'gardens-arrival-col-qty',
    from: 'wide',
    value: (row) => String(row.qty),
  },
  {
    field: 'qty_sold',
    type: 'number',
    headerId: 'bikri-saved-qty-sold',
    from: 'wide',
    value: (row) => String(row.qty_sold),
  },
];
