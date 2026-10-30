// The columns of the register of the one-time accounts (PRD FR-13.6, R9): date, بکری number, side,
// name, phone, item, نگ, amount, commission and paid or due. The amounts are the backend's (a buyer
// line: what he pays; a seller lot: خام بکری), and "due" says how much is still open, from the
// matching rows. A cancelled line is marked and is neither paid nor due. Less used columns come
// with the wider layout steps, so the list never scrolls sideways (NFR-USE-15, F22): with the 200 px
// side menu the room is 692, 874 and 1332 px at the narrowest window of each step
// (tests/unit/list-widths.test.ts). The نگ column does not fit beside the others at any step, so it
// comes with the "More columns" tick (`PUKHTA_MORE`), and a column steps aside where the room is
// short.

import { isoToDisplayDate } from '../../shared/format';
import { columnsWithMore, type ListColumn, type MoreColumnsPlan } from '../../shared/grid';
import type { LayoutStep } from '../../shared/hooks';
import { displayName } from '../../shared/i18n';
import type { PukhtaRegisterRow } from '../../bindings/PukhtaRegisterRow';

export const PUKHTA_COLUMNS: readonly ListColumn<PukhtaRegisterRow>[] = [
  {
    field: 'date',
    type: 'date',
    headerId: 'pukhta-col-date',
    sortKey: 'entry_date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'bikri',
    type: 'code',
    headerId: 'pukhta-col-bikri',
    sortKey: 'bikri_number',
    value: (row) => String(row.bikri_number),
  },
  {
    field: 'side',
    type: 'status',
    headerId: 'pukhta-col-side',
    sortKey: 'side',
    from: 'wide',
    value: (row, { text }) =>
      text(row.side === 'seller' ? 'pukhta-side-seller' : 'pukhta-side-buyer'),
  },
  {
    field: 'name',
    type: 'name',
    headerId: 'pukhta-col-name',
    sortKey: (language) => (language === 'ur' ? 'name_ur' : 'name_en'),
    grow: true,
    value: (row, { language }) => displayName(row, language),
  },
  {
    field: 'phone',
    // Eleven digits do not fit the 120 px of a code.
    type: 'number',
    headerId: 'pukhta-col-phone',
    sortKey: 'phone',
    from: 'normal',
    value: (row) => row.phone,
  },
  {
    field: 'item',
    type: 'text',
    headerId: 'pukhta-col-item',
    sortKey: 'item',
    from: 'wide',
    value: (row, { language }) => displayName(row.item, language),
  },
  {
    field: 'amount',
    type: 'money',
    headerId: 'pukhta-col-amount',
    sortKey: 'amount',
    value: (row, { money }) => money(row.amount),
  },
  {
    field: 'commission',
    type: 'money',
    headerId: 'glossary-commission',
    sortKey: 'commission',
    from: 'wide',
    value: (row, { money }) => money(row.commission),
  },
  {
    field: 'state',
    type: 'status',
    headerId: 'pukhta-col-state',
    sortKey: 'paid',
    value: (row, { text, money }) =>
      row.cancelled
        ? text('pukhta-state-cancelled')
        : row.paid
          ? text('pukhta-state-paid')
          : text('pukhta-state-due', { amount: money(row.open) }),
  },
];

/** The نگ column (PRD FR-13.6): the نگ of the line or lot, as the backend sends it. */
const QTY_COLUMN: ListColumn<PukhtaRegisterRow> = {
  field: 'qty',
  type: 'number',
  headerId: 'glossary-nag',
  sortKey: 'qty',
  value: (row) => String(row.qty),
};

/**
 * What "More columns" does: adds نگ after the item. Where the room is short, one column steps
 * aside: the date at the compact step (the register has its own date filters), the phone at the
 * normal step, the commission at the wide step.
 */
export const PUKHTA_MORE: MoreColumnsPlan<PukhtaRegisterRow> = {
  added: [{ column: QTY_COLUMN, after: 'item' }],
  makesRoom: { compact: ['date'], normal: ['phone'], wide: ['commission'] },
};

/** The columns for the layout step and the "More columns" tick. */
export function pukhtaColumns(
  step: LayoutStep,
  more: boolean,
): readonly ListColumn<PukhtaRegisterRow>[] {
  return columnsWithMore(PUKHTA_COLUMNS, PUKHTA_MORE, step, more);
}
