// The columns of the lists of the cash book area (PRD 16.1, FR-16.2, FR-16.5, FR-16.6, FR-19.11):
// the cash book, the day's shop expenses, a partner's money taken, a person's loans and a khata's
// entries. Amounts are the backend's and shown with `money`, never worked out. Less used columns
// come with the wider layout steps, so a list never scrolls sideways (NFR-USE-15, Code Standards
// F22). The room is the content area minus the main menu rail and the 200 px side menu: 692 px at
// the narrowest compact window. A cancelled entry says so in words (colour is never the only
// signal, NFR-USE-12).

import { isoToDisplayDate, type Paisa } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { voucherNumberText } from '../vouchers';
import type { CashbookRow, KhataEntryRow, LentRow, TakeRow, VoucherRow } from './types';

type Say = (id: string, vars?: Record<string, string>) => string;

/** The Fluent message ID of a stored kind such as `shop_expense`: hyphens, not underscores. */
export function kindId(kind: string): string {
  return kind.replaceAll('_', '-');
}

/** The row of the cash book as words: the kind of posting, and its number when it has one. */
export function cashbookKindText(row: Pick<CashbookRow, 'posting_type' | 'number'>, text: Say) {
  const kind = text(`cashbook-kind-${kindId(row.posting_type)}`);
  return row.number === null
    ? kind
    : text('cashbook-kind-number', { kind, number: String(row.number) });
}

/**
 * The cash in hand after a line. The backend sends it in date order only; a list sorted by anything
 * else has no running cash and the cell stays empty (the screen says so in words above the list).
 */
function runningText(running: Paisa | null, money: (amount: Paisa) => string): string {
  return running === null ? '' : money(running);
}

export const CASHBOOK_COLUMNS: readonly ListColumn<CashbookRow>[] = [
  {
    field: 'date',
    type: 'date',
    headerId: 'cashbook-col-date',
    sortKey: 'entry_date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'kind',
    type: 'text',
    headerId: 'cashbook-col-kind',
    sortKey: 'posting_type',
    grow: true,
    value: (row, { text }) => cashbookKindText(row, text),
  },
  {
    field: 'account',
    type: 'name',
    headerId: 'cashbook-col-account',
    from: 'normal',
    grow: true,
    value: (row, { language }) => (row.account === null ? '' : displayName(row.account, language)),
  },
  {
    field: 'narration',
    type: 'name',
    headerId: 'cashbook-col-narration',
    sortKey: 'narration',
    from: 'wide',
    grow: true,
    // A narration is free text, shown as typed (PRD FR-7.25).
    value: (row) => row.narration,
  },
  {
    field: 'in',
    type: 'figure',
    headerId: 'cashbook-col-in',
    sortKey: 'in_paisa',
    value: (row, { money }) => money(row.in_paisa),
  },
  {
    field: 'out',
    type: 'figure',
    headerId: 'cashbook-col-out',
    sortKey: 'out_paisa',
    value: (row, { money }) => money(row.out_paisa),
  },
  {
    field: 'running',
    type: 'figure',
    headerId: 'cashbook-col-running',
    value: (row, { money }) => runningText(row.running_paisa, money),
  },
];

/** The day's shop expenses under the form: number, head, amount, state, narration. */
export const EXPENSE_DAY_COLUMNS: readonly ListColumn<VoucherRow>[] = [
  {
    field: 'number',
    sortKey: 'number',
    type: 'code',
    headerId: 'cashbook-col-number',
    value: (row, { text }) => voucherNumberText(row, text),
  },
  {
    field: 'head',
    sortKey: 'account',
    type: 'name',
    headerId: 'cashbook-col-head',
    grow: true,
    value: (row, { language }) => (row.account === null ? '' : displayName(row.account, language)),
  },
  {
    field: 'amount',
    sortKey: 'amount_paisa',
    type: 'money',
    headerId: 'cashbook-col-amount',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'state',
    sortKey: 'status',
    type: 'status',
    headerId: 'cashbook-col-state',
    value: (row, { text }) => (row.status === 'cancelled' ? text('cashbook-state-cancelled') : ''),
  },
  {
    field: 'narration',
    type: 'name',
    headerId: 'cashbook-col-narration',
    from: 'normal',
    grow: true,
    value: (row) => row.narration,
  },
];

/** A partner's money taken (PRD FR-16.2): what, why, who took it, when it comes back. */
export const TAKE_COLUMNS: readonly ListColumn<TakeRow>[] = [
  {
    field: 'date',
    sortKey: 'entry_date',
    type: 'date',
    headerId: 'cashbook-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'reason',
    sortKey: 'reason',
    type: 'name',
    headerId: 'cashbook-col-reason',
    grow: true,
    // An opening debit on his drawings account has no reason: the word says what it is.
    value: (row, { text }) =>
      row.source === 'opening' && row.reason === '' ? text('cashbook-take-opening') : row.reason,
  },
  {
    field: 'taken_by',
    sortKey: 'taken_by',
    type: 'name',
    headerId: 'cashbook-col-taken-by',
    from: 'wide',
    value: (row) => row.taken_by,
  },
  {
    field: 'return_by',
    sortKey: 'return_by',
    type: 'date',
    headerId: 'cashbook-col-return-by',
    from: 'normal',
    value: (row) => (row.return_by === null ? '' : isoToDisplayDate(row.return_by)),
  },
  {
    field: 'amount',
    sortKey: 'amount_paisa',
    type: 'money',
    headerId: 'cashbook-col-taken',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'returned',
    sortKey: 'returned_paisa',
    type: 'money',
    headerId: 'cashbook-col-returned',
    from: 'normal',
    value: (row, { money }) => money(row.returned_paisa),
  },
  {
    field: 'open',
    sortKey: 'open_paisa',
    type: 'money',
    headerId: 'cashbook-col-open',
    value: (row, { money }) => money(row.open_paisa),
  },
  {
    field: 'days_past',
    sortKey: 'days_past_return_by',
    type: 'number',
    headerId: 'cashbook-col-days-past',
    from: 'wide',
    value: (row) => (row.days_past_return_by === null ? '' : String(row.days_past_return_by)),
  },
];

/** A person's loans (PRD FR-16.5). */
export const LENT_COLUMNS: readonly ListColumn<LentRow>[] = [
  {
    field: 'number',
    sortKey: 'number',
    type: 'code',
    headerId: 'cashbook-col-number',
    // An opening debit on his account is a loan with no voucher: the backend sends voucher_id 0
    // and number 0 (plan 17 hand-off), so no number is shown for it.
    value: (row, { text }) =>
      row.voucher_id === 0
        ? ''
        : text('vouchers-number-money-lent', { number: String(row.number) }),
  },
  {
    field: 'date',
    sortKey: 'entry_date',
    type: 'date',
    headerId: 'cashbook-col-date',
    from: 'normal',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'reason',
    sortKey: 'reason',
    type: 'name',
    headerId: 'cashbook-col-reason',
    grow: true,
    value: (row, { text }) =>
      row.voucher_id === 0 && row.reason === '' ? text('cashbook-take-opening') : row.reason,
  },
  {
    field: 'return_by',
    sortKey: 'return_by',
    type: 'date',
    headerId: 'cashbook-col-return-by',
    from: 'normal',
    value: (row) => (row.return_by === null ? '' : isoToDisplayDate(row.return_by)),
  },
  {
    field: 'amount',
    sortKey: 'amount_paisa',
    type: 'money',
    headerId: 'cashbook-col-lent',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'returned',
    sortKey: 'returned_paisa',
    type: 'money',
    headerId: 'cashbook-col-returned',
    from: 'normal',
    value: (row, { money }) => money(row.returned_paisa),
  },
  {
    field: 'open',
    sortKey: 'open_paisa',
    type: 'money',
    headerId: 'cashbook-col-open',
    value: (row, { money }) => money(row.open_paisa),
  },
  {
    field: 'days_past',
    sortKey: 'days_past_return_by',
    type: 'number',
    headerId: 'cashbook-col-days-past',
    from: 'wide',
    value: (row) => (row.days_past_return_by === null ? '' : String(row.days_past_return_by)),
  },
];

/** A khata's entries in date order with the signed running balance (PRD FR-16.6). */
export const KHATA_COLUMNS: readonly ListColumn<KhataEntryRow>[] = [
  {
    field: 'date',
    sortKey: 'entry_date',
    type: 'date',
    headerId: 'cashbook-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'kind',
    sortKey: 'kind',
    type: 'text',
    headerId: 'cashbook-col-kind',
    from: 'normal',
    value: (row, { text }) =>
      row.kind === 'other' ? '' : text(`cashbook-khata-kind-${kindId(row.kind)}`),
  },
  {
    field: 'reason',
    sortKey: 'reason',
    type: 'name',
    headerId: 'cashbook-col-reason',
    grow: true,
    value: (row) => row.reason,
  },
  {
    field: 'amount',
    sortKey: 'amount_paisa',
    type: 'money',
    headerId: 'cashbook-col-amount',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'running',
    sortKey: 'running_paisa',
    type: 'money',
    headerId: 'cashbook-col-balance',
    value: (row, { money }) => runningText(row.running_paisa, money),
  },
];
