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
  {
    field: 'qty_left',
    type: 'number',
    headerId: 'gardens-arrival-col-left',
    value: (row) => String(row.qty_left),
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
