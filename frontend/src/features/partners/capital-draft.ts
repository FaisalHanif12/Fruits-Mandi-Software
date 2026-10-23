// The text side of the سرمایہ screen (Code Standards F39, PRD 17.4, FR-7.67): the form as text,
// the entry the backend takes for each type, and the field paths its problems point at. Nothing
// here works out money (CLAUDE.md rule 1).

import { isoToDisplayDate } from '../../shared/format';
import type { CapitalType } from './screen-type';
import type { CapitalEntry } from './types';

/** One entry as the form holds it. Both types have the same fields, in the same places. */
export interface CapitalDraft {
  type: CapitalType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** The partner's id as the drop-down holds it, or empty. */
  partnerId: string;
  /** Whole rupees as typed. */
  amount: string;
  narration: string;
}

export function newCapitalDraft(type: CapitalType, today: string, partnerId = ''): CapitalDraft {
  return {
    type,
    date: today === '' ? '' : isoToDisplayDate(today),
    partnerId,
    amount: '',
    narration: '',
  };
}

/**
 * Changing the type (PRD FR-7.67): both types have the same fields, so nothing is lost; only the
 * direction of the money changes. Example: 50,000 typed as "invested" and then "taken back" stays
 * 50,000 for the same partner.
 */
export function changeCapitalType(draft: CapitalDraft, type: CapitalType): CapitalDraft {
  return { ...draft, type };
}

/** The next entry after a save: the date, the type and the partner stay, so his list stays in view. */
export function nextCapitalDraft(draft: CapitalDraft): CapitalDraft {
  return { ...newCapitalDraft(draft.type, ''), date: draft.date, partnerId: draft.partnerId };
}

/**
 * True once anything was typed, so leaving asks first (PRD FR-7.9). The partner alone is not work
 * to lose: after a save he stays chosen.
 */
export function capitalIsFilled(draft: CapitalDraft): boolean {
  return [draft.amount, draft.narration].some((text) => text.trim() !== '');
}

function idOrNull(text: string): number | null {
  const id = Number(text);
  return text.trim() !== '' && Number.isSafeInteger(id) && id > 0 ? id : null;
}

export function buildCapital(draft: CapitalDraft, confirm: boolean): CapitalEntry {
  const narration = draft.narration.trim();
  return {
    type: draft.type,
    date: draft.date.trim(),
    partner_id: idOrNull(draft.partnerId),
    amount: draft.amount.trim(),
    narration: narration === '' ? null : narration,
    confirm_cash_below_zero: confirm,
  };
}

export const CAPITAL_FIELD_PATHS: readonly string[] = ['date', 'partner_id', 'amount', 'narration'];
