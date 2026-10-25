// The voucher as text in the screen's state (Code Standards F39): a new draft, changing the type,
// the input the backend takes, and a saved voucher loaded back for an edit. Nothing here works
// out money: the amount is the text the user typed (or the text of an amount the backend sent).

import type { Language } from '../../i18n';
import type { UiError } from '../../bindings/UiError';
import { formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import { displayName } from '../../shared/i18n';
import { pickedOf, type AccountHit, type AccountType } from '../../shared/accounts';
import type { PickerValue } from '../../shared/ui';
import { isPukhtaCode, PUKHTA_BUYER_CODE } from '../pukhta_khata';
import type { PukhtaSide } from '../../bindings/PukhtaSide';
import type { ScreenType } from './screen-type';
import type {
  AdvanceInput,
  FromSide,
  PaymentInput,
  ReceiptInput,
  VoucherDraft,
  VoucherEntry,
  VoucherRow,
  VoucherSaved,
} from './types';

/** A new voucher: today as the date, everything else empty. All money is روکڑ (PRD 15.1). */
export function newDraft(type: ScreenType, today: string): VoucherDraft {
  return {
    type,
    date: today === '' ? '' : isoToDisplayDate(today),
    account: null,
    accountType: null,
    from: 'buyer',
    person: null,
    personText: '',
    amount: '',
    collector: null,
    reason: '',
    expected_goods: '',
    narration: '',
  };
}

/**
 * The same screen with another type (PRD FR-7.67): what the types share stays (date, amount and
 * narration), the rest is cleared, because an account, a person or a reason of one type means
 * nothing to another. Example: 3,000 typed as a payment to Rahim, then the type list changed to
 * receipt: 3,000 stays, Rahim goes.
 */
export function changeType(draft: VoucherDraft, type: ScreenType): VoucherDraft {
  if (draft.type === type) return draft;
  return {
    ...newDraft(type, ''),
    date: draft.date,
    amount: draft.amount,
    narration: draft.narration,
  };
}

/**
 * The next voucher after one was saved: the type and the date stay, as the user is likely to enter
 * several of one kind in one sitting; the account, the amount and the notes start again.
 */
export function nextDraft(previous: VoucherDraft): VoucherDraft {
  return {
    ...newDraft(previous.type, ''),
    date: previous.date,
  };
}

/** A draft counts as a voucher once an account or an amount is typed. An empty form is not one. */
export function isFilled(draft: VoucherDraft): boolean {
  return draft.account !== null || draft.amount.trim() !== '';
}

/** What the account field takes, by type: a receipt takes the side the user chose (FR-15.12). */
export function accountScope(draft: VoucherDraft): 'buyer' | 'seller' | 'any' {
  if (draft.type === 'advance') return 'seller';
  if (draft.type === 'receipt') return draft.from;
  return 'any';
}

/** True when the account is one of the two one-time accounts: it needs a person (FR-13.5). */
export function needsPerson(draft: VoucherDraft): boolean {
  return isPukhtaCode(draft.account?.code);
}

/** Which side's one-time people the person field lists: the buyer account's or the seller's. */
export function personSide(draft: VoucherDraft): PukhtaSide {
  return draft.account?.code === PUKHTA_BUYER_CODE ? 'buyer' : 'seller';
}

/** A payment to a buyer is cash lent to him and needs a reason (PRD FR-15.11). */
export function isCashLent(draft: VoucherDraft): boolean {
  return draft.type === 'payment' && draft.accountType === 'buyer';
}

/** The account as the picker holds it, from a search result. */
export function pickerValueOf(hit: AccountHit): PickerValue {
  return pickedOf(hit);
}

/** The voucher that is checked or saved, with its type. */
export type BuiltVoucher =
  | { type: 'receipt'; input: ReceiptInput }
  | { type: 'payment'; input: PaymentInput }
  | { type: 'advance'; input: AdvanceInput };

/**
 * What goes to the backend: the text typed and the ids of what was chosen. Null while there is no
 * account, because the backend takes the account as an id. `confirmCash` is the user's answer to
 * "is this entry correct?" for an entry that takes cash below zero (PRD FR-6.7).
 */
export function build(draft: VoucherDraft, confirmCash: boolean): BuiltVoucher | null {
  const { account } = draft;
  if (account === null) return null;
  const common = {
    date: draft.date,
    amount: draft.amount,
    narration: draft.narration,
  };
  switch (draft.type) {
    case 'receipt':
      return {
        type: 'receipt',
        input: {
          ...common,
          account_id: account.id,
          person_key: needsPerson(draft) ? (draft.person?.person_key ?? null) : null,
          collector_staff_id: draft.collector?.id ?? null,
        },
      };
    case 'payment':
      return {
        type: 'payment',
        input: {
          ...common,
          account_id: account.id,
          person_key: needsPerson(draft) ? (draft.person?.person_key ?? null) : null,
          reason: isCashLent(draft) ? draft.reason : '',
          confirm_cash_below_zero: confirmCash,
        },
      };
    case 'advance':
      return {
        type: 'advance',
        input: {
          ...common,
          seller_account_id: account.id,
          expected_goods: draft.expected_goods,
          confirm_cash_below_zero: confirmCash,
        },
      };
  }
}

/** The tagged form the preview and the edit take. */
export function entryOf(built: BuiltVoucher): VoucherEntry {
  switch (built.type) {
    case 'receipt':
      return { type: 'receipt', ...built.input };
    case 'payment':
      return { type: 'payment', ...built.input };
    case 'advance':
      return { type: 'advance', ...built.input };
  }
}

/** The names of the records a saved voucher points at by id only (the collector). */
export interface RowExtras {
  accountType: AccountType | null;
  collector: PickerValue | null;
}

/**
 * A saved voucher loaded back into the form for an edit (PRD FR-15.3). The amount is the text of
 * the saved amount from the backend, so it is exactly what the user would have typed.
 */
export function draftOfRow(
  row: VoucherRow,
  type: ScreenType,
  extras: RowExtras,
  grouping: Grouping,
  language: Language,
): VoucherDraft {
  const person = row.person;
  return {
    type,
    date: isoToDisplayDate(row.entry_date),
    account: row.account === null ? null : { ...row.account },
    accountType: extras.accountType,
    from: type === 'receipt' && extras.accountType === 'seller' ? 'seller' : 'buyer',
    person,
    personText: person === null ? '' : displayName(person, language),
    amount: formatPaisa(row.amount_paisa, { grouping }),
    collector: extras.collector,
    reason: row.reason,
    expected_goods: row.expected_goods,
    narration: row.narration,
  };
}

/** Who a receipt comes from, from the kind of account (a seller returns advance money). */
export function fromSideOf(accountType: AccountType | null): FromSide {
  return accountType === 'seller' ? 'seller' : 'buyer';
}

/** Which helper panel the screen shows under the form, and for which account (PRD FR-7.58 to FR-7.60). */
export type HelperTarget =
  | { kind: 'bills'; accountId: number; personKey: string | null }
  | { kind: 'payables'; accountId: number; personKey: string | null }
  | { kind: 'advances'; accountId: number }
  | { kind: 'none' };

/**
 * The panel that fits the entry: a receipt from a buyer shows his open bills (the amount settles
 * the oldest first); a receipt from a seller and an advance show his advances; a payment to a
 * seller shows what is payable; a payment to a buyer is cash lent and shows his bills. A
 * one-time account shows nothing until the person is chosen, because its money is kept per person.
 */
export function helperTarget(draft: VoucherDraft): HelperTarget {
  const { account } = draft;
  if (account === null) return { kind: 'none' };
  const person = needsPerson(draft);
  if (person && draft.person === null) return { kind: 'none' };
  const personKey = person ? (draft.person?.person_key ?? null) : null;
  if (draft.type === 'advance') return { kind: 'advances', accountId: account.id };
  if (draft.type === 'receipt') {
    return draft.from === 'seller'
      ? person
        ? { kind: 'none' }
        : { kind: 'advances', accountId: account.id }
      : { kind: 'bills', accountId: account.id, personKey };
  }
  if (draft.accountType === 'seller') return { kind: 'payables', accountId: account.id, personKey };
  if (draft.accountType === 'buyer') return { kind: 'bills', accountId: account.id, personKey };
  return { kind: 'none' };
}

/** The field path of the account in a refusal: an advance names its seller differently (contract). */
export function accountPath(draft: VoucherDraft): string {
  return draft.type === 'advance' ? 'seller_account_id' : 'account_id';
}

/** Where a voucher of the waiting list stands (PRD FR-15.1). A saved one leaves the list. */
export type ListedStatus =
  | { kind: 'waiting' }
  /** The backend refused it: it stays with the reason and can be saved again or removed. */
  | { kind: 'refused'; error: UiError }
  /** The user answered "no" to the question about cash below zero: nothing was saved. */
  | { kind: 'declined' };

/** One voucher waiting in the list, with its own copy of the form as text. */
export interface ListedVoucher {
  id: string;
  draft: VoucherDraft;
  status: ListedStatus;
}

/** What one voucher of a list run came to. */
export type ListedOutcome =
  | { id: string; kind: 'saved'; draft: VoucherDraft; saved: VoucherSaved }
  | { id: string; kind: 'refused'; error: UiError }
  | { id: string; kind: 'declined' };

let listedCount = 0;

/** The form put in the list: its own copy of the text, with an id only the list uses. */
export function listed(draft: VoucherDraft): ListedVoucher {
  listedCount += 1;
  return { id: `waiting-${String(listedCount)}`, draft: { ...draft }, status: { kind: 'waiting' } };
}

/** The account is the one thing a voucher cannot be listed without: it is asked for at its field. */
export function accountRequired(draft: VoucherDraft): UiError {
  return {
    ok: false,
    errors: [{ code: 'ERR_REQUIRED', field: accountPath(draft), values: {} }],
    ref: '',
  };
}

/** The amount is the other thing a voucher cannot be listed without. */
export function amountRequired(): UiError {
  return {
    ok: false,
    errors: [{ code: 'ERR_REQUIRED', field: 'amount', values: {} }],
    ref: '',
  };
}
