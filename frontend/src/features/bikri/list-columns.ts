// The columns of the list of بکری (PRD FR-12.7): number, day and time, truck, sellers, items, نگ,
// خام, صافی, buyer total and status. All amounts are the stored sums the backend sends, shown with
// `money` and never worked out. Less used columns come with the wider layout steps, so no list
// scrolls sideways (NFR-USE-15, Code Standards F22). The room is the content area minus the 200 px
// side menu of the بکری area: 692 px at the narrowest compact window, 874 px at the narrowest
// normal one, 1332 px at the narrowest wide one (tests/unit/list-widths.test.ts). The نگ column
// does not fit beside the others at any step, so it comes with the "More columns" tick
// (`BIKRI_LIST_MORE`), and a few columns step aside where the room is short.

import { editedTip, statusWithEdited } from '../../shared/edit';
import { isoToDisplayDate, utcToLocalTime } from '../../shared/format';
import { columnsWithMore, type ListColumn, type MoreColumnsPlan } from '../../shared/grid';
import type { LayoutStep } from '../../shared/hooks';
import { displayName, lotItemWith, lotNameWith, type Say } from '../../shared/i18n';
import type { BikriListRow } from '../../bindings/BikriListRow';

/** The sellers of a بکری in lot order. */
export function sellerNames(
  row: BikriListRow,
  language: Parameters<typeof displayName>[1],
  separator: string,
): string {
  return row.sellers.map((seller) => displayName(seller, language)).join(separator);
}

/**
 * The items of a بکری, one for each lot with its counter, in lot order: "Mango-2, Banana-1, Mango-3"
 * (PRD FR-12.45). The seller of each is in the column before it.
 */
export function lotItemNames(
  row: BikriListRow,
  language: Parameters<typeof displayName>[1],
  text: Say,
): string {
  return row.lots
    .map((lot) => lotItemWith(text, language, lot.item, lot.counter))
    .join(text('bikri-list-separator'));
}

/**
 * The names of the lots of a بکری, "Ehsan, Mango-2; Ali, Banana-1", in lot order: the seller, the item
 * and the counter of each (PRD FR-12.45). Used where the seller and the item share one line.
 */
export function lotNamesOf(
  row: BikriListRow,
  language: Parameters<typeof displayName>[1],
  text: Say,
): string {
  return row.lots
    .map((lot) => lotNameWith(text, language, lot.seller, lot.item, lot.counter))
    .join(text('bikri-list-separator'));
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
    // A new بکری has no truck (PRD 12.2, v1.34), so its cell is empty; an old one shows its truck
    // (plan 72 P3). The truck is also found through the search box.
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
    value: (row, { language, text }) => lotItemNames(row, language, text),
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
    value: (row, context) =>
      statusWithEdited(
        context.text(row.status === 'cancelled' ? 'bikri-status-cancelled' : 'bikri-status-saved'),
        row.edited,
        context,
      ),
    tip: (row, context) => editedTip(row.edited, context),
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
 * What "More columns" does: adds نگ after the items (and the truck at the normal step). Where the room is short, the least needed
 * column steps aside: خام at the compact step (صافی stays), the day and time at the others (the
 * day is a filter of the screen already).
 */
export const BIKRI_LIST_MORE: MoreColumnsPlan<BikriListRow> = {
  added: [{ column: QTY_COLUMN, after: 'items' }],
  // The truck is a base column from the wide step (PRD 12.7, plan 72 P3). The tick offers it at the
  // normal step too, with the نگ, by letting the day and time and the خام بکری step aside (صافی بکری
  // stays). At the compact step there is no room for both columns, so only the نگ comes, as before.
  promote: { normal: ['truck'] },
  makesRoom: { compact: ['gross'], normal: ['when', 'gross'], wide: ['when'] },
};

/** The columns for the layout step and the "More columns" tick. */
export function bikriListColumns(
  step: LayoutStep,
  more: boolean,
): readonly ListColumn<BikriListRow>[] {
  return columnsWithMore(BIKRI_LIST_COLUMNS, BIKRI_LIST_MORE, step, more);
}
