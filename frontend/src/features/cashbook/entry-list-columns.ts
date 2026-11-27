// The columns of the lists of saved entries under the خرچہ, ادھار رقم, الگ کھاتہ and جنرل forms (PRD
// FR-19.11, plan 100 F): the number in the series of its type, the day, who or what it is for, the kind of
// entry where a list holds two (money lent and returned, paid out and received), the amount, why, and
// whether it stands. Every value is the backend's stored one; the list adds up nothing. Less used columns
// come with the wider layout steps, so the list never scrolls sideways (NFR-USE-15).

import type { VoucherType } from '../../bindings/VoucherType';
import { editedTip, statusWithEdited } from '../../shared/edit';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { STATUS_CANCELLED, STATUS_SAVED } from '../../shared/ui';
import { voucherNumberText } from '../vouchers';
import type { VoucherRow } from './types';

const NUMBER: ListColumn<VoucherRow> = {
  field: 'number',
  sortKey: 'number',
  type: 'code',
  headerId: 'cashbook-col-number',
  value: (row, { text }) => voucherNumberText(row, text),
};

const DATE: ListColumn<VoucherRow> = {
  field: 'date',
  sortKey: 'entry_date',
  type: 'date',
  headerId: 'cashbook-col-date',
  value: (row) => isoToDisplayDate(row.entry_date),
};

/** The account the entry is with: an expense head, a person or a khata. */
function accountColumn(headerId: string): ListColumn<VoucherRow> {
  return {
    field: 'account',
    sortKey: 'account',
    type: 'name',
    headerId,
    grow: true,
    value: (row, { language }) => (row.account === null ? '' : displayName(row.account, language)),
  };
}

/** The words of the kinds of the two lists that hold two, as the type list of each form says them. */
const KIND_IDS: Partial<Record<VoucherType, string>> = {
  money_lent: 'cashbook-type-money-lent',
  money_lent_return: 'cashbook-type-money-lent-return',
  khata_out: 'cashbook-type-khata-out',
  khata_in: 'cashbook-type-khata-in',
};

/** Which of the two kinds of the list the entry is. */
const KIND: ListColumn<VoucherRow> = {
  field: 'entry_kind',
  type: 'short',
  headerId: 'cashbook-col-kind',
  value: (row, { text }) => {
    const id = KIND_IDS[row.voucher_type];
    return id === undefined ? '' : text(id);
  },
};

const AMOUNT: ListColumn<VoucherRow> = {
  field: 'amount',
  sortKey: 'amount_paisa',
  type: 'money',
  headerId: 'cashbook-col-amount',
  value: (row, { money }) => money(row.amount_paisa),
};

/** Why: the reason, or the note when the entry has no reason (a shop expense). Free text, as typed. */
const REASON: ListColumn<VoucherRow> = {
  field: 'reason',
  type: 'name',
  headerId: 'cashbook-col-reason',
  from: 'normal',
  grow: true,
  value: (row) => (row.reason === '' ? row.narration : row.reason),
};

const STATE: ListColumn<VoucherRow> = {
  field: 'state',
  sortKey: 'status',
  type: 'status',
  headerId: 'cashbook-col-state',
  value: (row, context) =>
    statusWithEdited(
      context.text(
        row.status === 'cancelled' ? 'cashbook-state-cancelled' : 'vouchers-state-saved',
      ),
      row.edited,
      context,
    ),
  chip: (row) => (row.status === 'cancelled' ? STATUS_CANCELLED : STATUS_SAVED),
  tip: (row, context) => editedTip(row.edited, context),
};

export const SHOP_ENTRY_COLUMNS: readonly ListColumn<VoucherRow>[] = [
  NUMBER,
  DATE,
  accountColumn('cashbook-col-head'),
  AMOUNT,
  STATE,
  REASON,
];

export const LENT_ENTRY_COLUMNS: readonly ListColumn<VoucherRow>[] = [
  NUMBER,
  DATE,
  accountColumn('cashbook-col-person'),
  KIND,
  AMOUNT,
  STATE,
  REASON,
];

export const KHATA_ENTRY_COLUMNS: readonly ListColumn<VoucherRow>[] = [
  NUMBER,
  DATE,
  accountColumn('cashbook-col-khata'),
  KIND,
  AMOUNT,
  STATE,
  REASON,
];

/** A list of kinds that is never empty. */
type Kinds = readonly [VoucherType, ...VoucherType[]];

/** The kinds of entry each list under a form holds (plan 100 F). */
export const SHOP_ENTRY_TYPES: Kinds = ['shop_expense'];
export const LENT_ENTRY_TYPES: Kinds = ['money_lent', 'money_lent_return'];
export const KHATA_ENTRY_TYPES: Kinds = ['khata_out', 'khata_in'];
export const GENERAL_ENTRY_TYPES: Kinds = ['general'];
