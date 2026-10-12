// The advance voucher as text in the screen's state (Code Standards F39): a new draft, whether a
// draft has anything typed, and the list of vouchers waiting to be saved together (PRD FR-15.1).
// Nothing here works out money: the amount is the text the user typed.

import type { UiError } from '../../bindings/UiError';
import { isoToDisplayDate } from '../../shared/format';
import type { AdvanceDraft, ReadyAdvance, VoucherSaved } from './types';

/** A new voucher: today as the date, cash as the mode, everything else empty (PRD 15.1). */
export function newAdvanceDraft(today: string): AdvanceDraft {
  return {
    date: today === '' ? '' : isoToDisplayDate(today),
    seller: null,
    amount: '',
    mode: 'cash',
    bank: null,
    cheque_no: '',
    cheque_date: '',
    expected_goods: '',
    narration: '',
  };
}

/**
 * The next voucher after one was added to the list: the date and the mode stay, as the user is
 * likely to enter several of one day; the seller, the amount and the notes are empty again.
 */
export function nextAdvanceDraft(previous: AdvanceDraft): AdvanceDraft {
  return {
    ...newAdvanceDraft(''),
    date: previous.date,
    mode: previous.mode,
    bank: previous.bank,
  };
}

/** Narrows a draft to one with a seller, which is what the backend takes (as an id). */
export function hasSeller(draft: AdvanceDraft): draft is ReadyAdvance {
  return draft.seller !== null;
}

/** A draft counts as a voucher once a seller or an amount is typed. An empty form is not one. */
export function isFilled(draft: AdvanceDraft): boolean {
  return draft.seller !== null || draft.amount.trim() !== '';
}

/** A voucher in the list: waiting, saved (with its number and new balance), or refused. */
export type ListedStatus =
  | { kind: 'waiting' }
  | { kind: 'saved'; saved: VoucherSaved }
  | { kind: 'refused'; error: UiError };

export interface ListedVoucher {
  /** Stable for the life of the row, never its position. */
  id: string;
  draft: AdvanceDraft;
  status: ListedStatus;
}

export function listed(draft: AdvanceDraft): ListedVoucher {
  return { id: globalThis.crypto.randomUUID(), draft, status: { kind: 'waiting' } };
}

/** The vouchers that still have to be saved: waiting ones, and ones the backend refused. */
export function toSave(list: readonly ListedVoucher[]): ListedVoucher[] {
  return list.filter((voucher) => voucher.status.kind !== 'saved');
}
