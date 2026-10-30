// The text side of the ادھار رقم screen (Code Standards F39, PRD 16.5, FR-16.4, FR-7.67): the form
// as text, the entry the backend takes for each type, and the field paths its problems point at.
// Nothing here works out money (CLAUDE.md rule 1).

import { isoToDisplayDate } from '../../shared/format';
import type { PickerValue } from '../../shared/ui';
import type { LentReturnMode, VoucherEntry } from './types';
import type { LentType } from './screen-type';

/** One entry as the form holds it. The person is the record the picker holds. */
export interface LentDraft {
  type: LentType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  person: PickerValue | null;
  /** Whole rupees as typed. */
  amount: string;
  /** Lent only: why the money was lent (required). */
  reason: string;
  /** Lent only. `DD-MM-YYYY` as typed. */
  returnBy: string;
  /** Returned only: in cash, or deducted from his salary (PRD FR-16.4). */
  mode: LentReturnMode;
  /** Returned by salary: the expense head the picker holds, or null for the default تنخواہ. */
  salaryHead: PickerValue | null;
  narration: string;
}

export type LentEntry = Extract<VoucherEntry, { type: 'money_lent' | 'money_lent_return' }>;

export function newLentDraft(type: LentType, today: string): LentDraft {
  return {
    type,
    date: today === '' ? '' : isoToDisplayDate(today),
    person: null,
    amount: '',
    reason: '',
    returnBy: '',
    mode: 'cash',
    salaryHead: null,
    narration: '',
  };
}

/**
 * Changing the type (PRD FR-7.67): the date, the amount and the narration stay, and so does the
 * person, because both types are about one person and the list below the form is his. The rest
 * (reason, return-by, how it is returned) belongs to one type and starts again.
 */
export function changeLentType(draft: LentDraft, type: LentType): LentDraft {
  return {
    ...newLentDraft(type, ''),
    date: draft.date,
    person: draft.person,
    amount: draft.amount,
    narration: draft.narration,
  };
}

/** The next entry after a save: the date and the type stay, and so does the person. */
export function nextLentDraft(draft: LentDraft): LentDraft {
  return { ...newLentDraft(draft.type, ''), date: draft.date, person: draft.person };
}

/**
 * True once anything was typed, so leaving asks first (PRD FR-7.9). The person alone is not work to
 * lose: after a save he stays chosen for the next entry.
 */
export function lentIsFilled(draft: LentDraft): boolean {
  return [draft.amount, draft.reason, draft.returnBy, draft.narration].some(
    (text) => text.trim() !== '',
  );
}

function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

export function buildLent(draft: LentDraft, confirm: boolean): LentEntry {
  const date = draft.date.trim();
  const amount = draft.amount.trim();
  const person = draft.person?.id ?? null;
  if (draft.type === 'money_lent') {
    return {
      type: 'money_lent',
      date,
      person_account_id: person,
      amount,
      reason: draft.reason.trim(),
      return_by: textOrNull(draft.returnBy),
      narration: textOrNull(draft.narration),
      confirm_cash_below_zero: confirm,
    };
  }
  return {
    type: 'money_lent_return',
    date,
    person_account_id: person,
    amount,
    mode: draft.mode,
    salary_head_id: draft.mode === 'salary' ? (draft.salaryHead?.id ?? null) : null,
    narration: textOrNull(draft.narration),
  };
}

export const LENT_FIELD_PATHS: Record<LentType, readonly string[]> = {
  money_lent: ['date', 'person_account_id', 'amount', 'reason', 'return_by', 'narration'],
  money_lent_return: ['date', 'person_account_id', 'amount', 'mode', 'salary_head_id', 'narration'],
};
