// The columns of the list of بکری (PRD FR-12.7): number, day and time, truck, sellers, items, نگ,
// خام, صافی, buyer total and status. All amounts are the stored sums the backend sends, shown with
// `money` and never worked out. Less used columns come with the wider layout steps, so no list
// scrolls sideways (NFR-USE-15, Code Standards F22). The room is the content area minus the 200 px
// side menu of the بکری area: 692 px at the narrowest compact window, 874 px at the narrowest
// normal one, 1332 px at the narrowest wide one (tests/unit/list-widths.test.ts). The نگ column
// does not fit beside the others at any step, so it comes with the "More columns" tick
// (`BIKRI_LIST_MORE`), and a few columns step aside where the room is short.

import { isoToDisplayDate, utcToLocalTime } from '../../shared/format';
import { columnsWithMore, type ListColumn, type MoreColumnsPlan } from '../../shared/grid';
import type { LayoutStep } from '../../shared/hooks';
import { displayName } from '../../shared/i18n';
import type { BikriListRow } from '../../bindings/BikriListRow';
import { PUKHTA_SELLER_CODE } from './types';

/** The sellers of a بکری in lot order. A one-time seller is shown by his own name (FR-13.2). */
export function sellerNames(
  row: BikriListRow,
  language: Parameters<typeof displayName>[1],
  separator: string,
): string {
  const named = row.sellers
    .filter((seller) => seller.code !== PUKHTA_SELLER_CODE)
    .map((seller) => displayName(seller, language));
  const oneTime = row.pukhta_sellers.map((person) => displayName(person, language));
  return [...named, ...oneTime].join(separator);
}

export const BIKRI_LIST_COLUMNS: readonly ListColumn<BikriListRow>[] = [
  {
    field: 'number',
    type: 'code',
    headerId: 'bikri-list-col-number',
    sortKey: 'number',
    value: (row) => String(row.number),
  },
  {
    field: 'when',
    type: 'datetime',
    headerId: 'bikri-list-col-when',
    sortKey: 'entry_date',
    from: 'normal',
    // The day of the posting is the day typed; the time is when it was saved, in Pakistan time.
    value: (row) => `${isoToDisplayDate(row.entry_date)} ${utcToLocalTime(row.created_at)}`,
  },
  {
    field: 'truck',
    type: 'code',
    headerId: 'bikri-list-col-truck',
    sortKey: 'truck_no',
    from: 'wide',
    value: (row) => row.truck_no ?? '',
  },
  {
    field: 'sellers',
    type: 'name',
    headerId: 'bikri-list-col-sellers',
    grow: true,
    value: (row, { language, text }) => sellerNames(row, language, text('bikri-list-separator')),
  },
  {
    field: 'items',
    type: 'text',
    headerId: 'bikri-list-col-items',
    from: 'wide',
    value: (row, { language, text }) =>
      row.items.map((item) => displayName(item, language)).join(text('bikri-list-separator')),
  },
  {
    field: 'gross',
    type: 'money',
    headerId: 'glossary-kham-bikri',
    sortKey: 'gross',
    value: (row, { money }) => money(row.gross),
  },
  {
    field: 'net',
    type: 'money',
    headerId: 'glossary-safi-bikri',
    sortKey: 'net',
    value: (row, { money }) => money(row.net),
  },
  {
    field: 'buyer_total',
    type: 'money',
    headerId: 'bikri-list-col-buyer-total',
    sortKey: 'buyer_total',
    from: 'wide',
    value: (row, { money }) => money(row.buyer_total),
  },
  {
    field: 'status',
    type: 'status',
    headerId: 'bikri-list-col-status',
    sortKey: 'status',
    value: (row, { text }) =>
      text(row.status === 'cancelled' ? 'bikri-status-cancelled' : 'bikri-status-saved'),
  },
];

/**
 * The نگ column (PRD FR-12.7): sold of received over all the lots, both from the backend, for
 * example "45 / 50". One cell, so a بکری with goods left shows it at once.
 */
const QTY_COLUMN: ListColumn<BikriListRow> = {
  field: 'qty',
  type: 'number',
  headerId: 'bikri-list-col-qty',
  sortKey: 'qty_received',
  value: (row, { text }) =>
    text('bikri-list-qty', { sold: String(row.qty_sold), received: String(row.qty_received) }),
};

/**
 * What "More columns" does: adds نگ after the items. Where the room is short, the least needed
 * column steps aside: خام at the compact step (صافی stays), the day and time at the others (the
 * day is a filter of the screen already).
 */
export const BIKRI_LIST_MORE: MoreColumnsPlan<BikriListRow> = {
  added: [{ column: QTY_COLUMN, after: 'items' }],
  makesRoom: { compact: ['gross'], normal: ['when'], wide: ['when'] },
};

/** The columns for the layout step and the "More columns" tick. */
export function bikriListColumns(
  step: LayoutStep,
  more: boolean,
): readonly ListColumn<BikriListRow>[] {
  return columnsWithMore(BIKRI_LIST_COLUMNS, BIKRI_LIST_MORE, step, more);
}
