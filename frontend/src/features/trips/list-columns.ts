// The columns of the lists of the trips module (PRD FR-12.25, FR-19.10, FR-19.11): the trips, and
// the history of one trip. Amounts and counts are the backend's and shown with `money` and
// `String`, never worked out. The lists sit under the main menu only, so the room is the content
// area minus the 64 px rail; less used columns come with the wider layout steps and a list never
// scrolls sideways (NFR-USE-15, Code Standards F22). A cancelled entry says so in words.

import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { purchaseNumberText } from '../purchases';
import type { TripEntryKind, TripEntryRow, TripRow } from './types';

type Say = (id: string, vars?: Record<string, string>) => string;

/** The number of an entry in the series of its kind: a purchase, a trip cost or a sale elsewhere. */
export function entryNumberText(kind: TripEntryKind, number: number, text: Say): string {
  switch (kind) {
    case 'purchase':
      return purchaseNumberText(number, text);
    case 'cost':
      return text('trips-cost-number', { number: String(number) });
    case 'other_sale':
      return text('trips-sale-number', { number: String(number) });
  }
}

/** The word for the kind of an entry (the type list of the trip screen). */
export function entryKindText(kind: TripEntryKind, text: Say): string {
  return text(`trips-type-${kind.replaceAll('_', '-')}`);
}

export const TRIP_COLUMNS: readonly ListColumn<TripRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'trips-col-number',
    // The code of a trip, type 007 (plan 34): 0070, 0071 ... The backend makes it; it is shown as it is.
    value: (row) => row.code,
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'trips-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'market',
    type: 'name',
    headerId: 'trips-col-market',
    grow: true,
    // The market is free text, shown as typed (PRD FR-7.25).
    value: (row) => row.market ?? '',
  },
  {
    field: 'status',
    type: 'text',
    headerId: 'trips-col-status',
    value: (row, { text }) =>
      text(row.status === 'open' ? 'trips-status-open' : 'trips-status-closed'),
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
    field: 'cost',
    type: 'money',
    headerId: 'trips-fact-cost',
    from: 'wide',
    value: (row, { money }) => money(row.cost_paisa),
  },
  {
    field: 'sales',
    type: 'money',
    headerId: 'trips-col-sales',
    from: 'wide',
    value: (row, { money }) => money(row.sales_paisa),
  },
];

export const ENTRY_COLUMNS: readonly ListColumn<TripEntryRow>[] = [
  {
    field: 'entry',
    type: 'text',
    headerId: 'trips-col-entry',
    grow: true,
    value: (row, { text }) =>
      text('trips-entry-label', {
        kind: entryKindText(row.kind, text),
        number: entryNumberText(row.kind, row.number, text),
      }),
  },
  {
    field: 'date',
    type: 'date',
    headerId: 'trips-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'amount',
    type: 'money',
    headerId: 'trips-col-amount',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'state',
    type: 'text',
    headerId: 'trips-col-state',
    value: (row, { text }) => (row.cancelled ? text('trips-entry-cancelled') : ''),
  },
  {
    field: 'detail',
    type: 'name',
    headerId: 'trips-col-detail',
    from: 'normal',
    grow: true,
    // The reason and the shop are free text, shown as typed (PRD FR-7.25).
    value: (row, { text }) => {
      if (row.kind === 'cost') return row.reason ?? '';
      if (row.kind === 'other_sale') {
        return text('trips-entry-detail-sale', {
          shop: row.shop_name ?? '',
          qty: String(row.qty ?? 0),
          purchase:
            row.purchase_number === null ? '' : purchaseNumberText(row.purchase_number, text),
        });
      }
      return '';
    },
  },
];
