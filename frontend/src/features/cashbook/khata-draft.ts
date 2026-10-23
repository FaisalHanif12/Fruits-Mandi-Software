// The text side of the الگ کھاتہ screen (Code Standards F39, PRD 16.6, FR-7.67): the form as text,
// the entry the backend takes for each type, and the field paths its problems point at. Nothing
// here works out money (CLAUDE.md rule 1).

import { isoToDisplayDate } from '../../shared/format';
import type { PickerValue } from '../../shared/ui';
import type { VoucherEntry } from './types';
import type { KhataType } from './screen-type';

/** One entry as the form holds it. Both types have the same fields, in the same places. */
export interface KhataDraft {
  type: KhataType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  khata: PickerValue | null;
  /** Whole rupees as typed. */
  amount: string;
  /** Why the money moved (required, PRD 16.6). */
  reason: string;
  narration: string;
}

export type KhataEntry = Extract<VoucherEntry, { type: 'khata_out' | 'khata_in' }>;

export function newKhataDraft(type: KhataType, today: string): KhataDraft {
  return {
    type,
    date: today === '' ? '' : isoToDisplayDate(today),
    khata: null,
    amount: '',
    reason: '',
    narration: '',
  };
}

/**
 * Changing the type (PRD FR-7.67): both types have the same fields, so nothing is lost; only
 * the direction of the money changes. Example: 4,000 typed as "paid out" and then "received" stays
 * 4,000 for the same khata and reason.
 */
export function changeKhataType(draft: KhataDraft, type: KhataType): KhataDraft {
  return { ...draft, type };
}

/** The next entry after a save: the date, the type and the khata stay. */
export function nextKhataDraft(draft: KhataDraft): KhataDraft {
  return { ...newKhataDraft(draft.type, ''), date: draft.date, khata: draft.khata };
}

/**
 * True once anything was typed, so leaving asks first (PRD FR-7.9). The khata alone is not work to
 * lose: after a save it stays chosen for the next entry.
 */
export function khataIsFilled(draft: KhataDraft): boolean {
  return [draft.amount, draft.reason, draft.narration].some((text) => text.trim() !== '');
}

function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

export function buildKhata(draft: KhataDraft, confirm: boolean): KhataEntry {
  const fields = {
    date: draft.date.trim(),
    khata_account_id: draft.khata?.id ?? null,
    amount: draft.amount.trim(),
    reason: draft.reason.trim(),
    narration: textOrNull(draft.narration),
    confirm_cash_below_zero: confirm,
  };
  return draft.type === 'khata_out'
    ? { type: 'khata_out', ...fields }
    : { type: 'khata_in', ...fields };
}

export const KHATA_FIELD_PATHS: readonly string[] = [
  'date',
  'khata_account_id',
  'amount',
  'reason',
  'narration',
];
