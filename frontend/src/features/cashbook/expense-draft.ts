// The text side of the خرچہ screen (Code Standards F39, PRD 16.2, FR-7.67): the form as text in the
// screen's state, the entry the backend takes for each type, and the field paths its problems point
// at. Nothing here works out money (CLAUDE.md rule 1): the amount is the text the user typed (a
// simple sum such as `2000+500` is read by the backend, FR-7.48), and the backend reads it.

import { isoToDisplayDate } from '../../shared/format';
import type { PickerValue } from '../../shared/ui';
import type { ExpenseEntry } from './types';
import type { ExpenseType } from './screen-type';

/** One expense entry as the form holds it, all as text. A picked record is the one its picker holds. */
export interface ExpenseDraft {
  type: ExpenseType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** Whole rupees as typed. */
  amount: string;
  narration: string;
  /** دکان خرچہ only: the expense head the picker holds, or null. */
  head: PickerValue | null;
  /** دکان خرچہ only: who was paid, as typed (one script, PRD FR-7.25). */
  paidTo: string;
  /** ذاتی خرچہ and its واپسی: the partner's id, or empty. */
  partnerId: string;
  /** ذاتی خرچہ only. */
  reason: string;
  /** `DD-MM-YYYY` as typed. ذاتی خرچہ only. */
  returnBy: string;
  /** ذاتی خرچہ only: the person who took the cash, when not the partner himself. */
  takenBy: string;
}

/** A new form of one type on the day given (`YYYY-MM-DD`, or empty while it is not known yet). */
export function newExpenseDraft(type: ExpenseType, today: string): ExpenseDraft {
  return {
    type,
    date: today === '' ? '' : isoToDisplayDate(today),
    amount: '',
    narration: '',
    head: null,
    paidTo: '',
    partnerId: '',
    reason: '',
    returnBy: '',
    takenBy: '',
  };
}

/**
 * Changing the type (PRD FR-7.67): the date, the amount and the narration stay; the rest starts
 * again, so a head never leaks into a partner's spending. Example: a 3,000 دکان خرچہ changed to
 * ذاتی خرچہ keeps 3,000 and asks for the partner and the reason.
 */
export function changeExpenseType(draft: ExpenseDraft, type: ExpenseType): ExpenseDraft {
  return {
    ...newExpenseDraft(type, ''),
    date: draft.date,
    amount: draft.amount,
    narration: draft.narration,
  };
}

/**
 * The next entry of the same type after a save: the date stays, the rest starts again. A partner's
 * own spending and its return stay with the same partner, so his list of money taken stays in view
 * (PRD FR-16.2).
 */
export function nextExpenseDraft(draft: ExpenseDraft): ExpenseDraft {
  return {
    ...newExpenseDraft(draft.type, ''),
    date: draft.date,
    partnerId: draft.type === 'shop_expense' ? '' : draft.partnerId,
  };
}

/**
 * True once anything was typed, so leaving asks first (PRD FR-7.9). A head or a partner that is only
 * chosen is not work to lose: after a save the partner stays chosen for the next entry.
 */
export function expenseIsFilled(draft: ExpenseDraft): boolean {
  return [
    draft.amount,
    draft.narration,
    draft.paidTo,
    draft.reason,
    draft.returnBy,
    draft.takenBy,
  ].some((text) => text.trim() !== '');
}

/** The id of a picked record, or null while nothing is picked (the backend answers ..._REQUIRED). */
function idOrNull(text: string): number | null {
  const id = Number(text);
  return text.trim() !== '' && Number.isSafeInteger(id) && id > 0 ? id : null;
}

/** Text or nothing: an empty optional field is sent as null. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** What goes to the backend, for the type of the form. `confirm` answers the cash question. */
export function buildExpense(draft: ExpenseDraft, confirm: boolean): ExpenseEntry {
  const date = draft.date.trim();
  const amount = draft.amount.trim();
  switch (draft.type) {
    case 'shop_expense':
      return {
        type: 'shop_expense',
        date,
        expense_head_id: draft.head?.id ?? null,
        amount,
        paid_to: textOrNull(draft.paidTo),
        narration: textOrNull(draft.narration),
        confirm_cash_below_zero: confirm,
      };
    case 'personal_expense':
      return {
        type: 'personal_expense',
        date,
        partner_id: idOrNull(draft.partnerId),
        amount,
        reason: draft.reason.trim(),
        return_by: textOrNull(draft.returnBy),
        taken_by: textOrNull(draft.takenBy),
        narration: textOrNull(draft.narration),
        confirm_cash_below_zero: confirm,
      };
    case 'personal_return':
      return {
        type: 'personal_return',
        date,
        partner_id: idOrNull(draft.partnerId),
        amount,
        narration: textOrNull(draft.narration),
      };
  }
}

/** The fields each type draws, by the field paths the backend's problems use. */
export const EXPENSE_FIELD_PATHS: Record<ExpenseType, readonly string[]> = {
  shop_expense: ['date', 'expense_head_id', 'amount', 'paid_to', 'narration'],
  personal_expense: [
    'date',
    'partner_id',
    'amount',
    'reason',
    'return_by',
    'taken_by',
    'narration',
  ],
  personal_return: ['date', 'partner_id', 'amount', 'narration'],
};

/** The partner chosen as a number, for the list of his money taken. Null while none is chosen. */
export function chosenPartnerId(draft: ExpenseDraft): number | null {
  return idOrNull(draft.partnerId);
}
