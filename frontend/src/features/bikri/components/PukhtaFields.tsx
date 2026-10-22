// The one-time person of a lot or a line (PRD FR-13.2, FR-7.24, FR-7.29): a name in both scripts
// and a phone, for the پختہ کھاتہ account. The user types one name; the other fills by itself and
// is checked (NamePair).
//
// One merged list (PRD FR-7.29): while a name or the phone is typed, the one-time people already
// recorded come first, each with both names, the phone and the day of his last line, then (in the
// name typed first) the Urdu spellings of the word. Taking a person puts his exact names and phone
// in the three fields, so all his lines stay under one person key (FR-13.5); the names then carry
// no spelling marks, since they are his, as recorded. The list is the backend's
// (`search_pukhta_people`); nothing is searched or ordered here.
//
// The names, the phone and the spelling marks are text on the row, so a language switch keeps
// them (Code Standards F39). Nothing is saved here.

import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import type { UiError } from '../../../bindings/UiError';
import type { EntryRow, EntryStore } from '../../../shared/grid';
import { PukhtaPersonFields } from '../../pukhta_khata';

/** The keys of the row that hold the one-time person. */
export interface PukhtaKeys {
  ur: string;
  en: string;
  phone: string;
  marks: string;
}

interface PukhtaFieldsProps {
  store: EntryStore;
  row: EntryRow;
  /** The seller's or the buyer's one-time account: the list shows the people of that side. */
  side: PukhtaSide;
  keys: PukhtaKeys;
  /** The field paths of the backend's problems for the three fields, with the line or lot index. */
  paths: { ur: string; en: string; phone: string };
  error: UiError | null;
  /** Enter on the last field: the screen takes focus back to the grid. */
  onDone: () => void;
}

/**
 * Puts focus on the first field of the one-time person of a row, if its fields are on screen.
 * The grid calls it after the one-time account is chosen, so Enter goes to the name next.
 */
export function focusPukhtaFields(rowId: string): boolean {
  const first = document.querySelector<HTMLElement>(
    `[data-pukhta-for="${CSS.escape(rowId)}"] input`,
  );
  first?.focus();
  return first !== null && document.activeElement === first;
}

/** How many tries (one for each frame, about 16 ms) the fields of a new one-time person get to appear. */
const FOCUS_TRIES = 40;

/**
 * Like `focusPukhtaFields`, for the moment right after the one-time account was chosen: the fields
 * are drawn a moment later, so it tries again for each frame until they are there. On a slow
 * computer the first try can be too early.
 */
export function focusPukhtaFieldsSoon(rowId: string): void {
  let tries = 0;
  const attempt = () => {
    if (focusPukhtaFields(rowId)) return;
    tries += 1;
    if (tries < FOCUS_TRIES) window.requestAnimationFrame(attempt);
  };
  window.setTimeout(attempt, 0);
}

export function PukhtaFields({ store, row, side, keys, paths, error, onDone }: PukhtaFieldsProps) {
  return (
    <PukhtaPersonFields
      anchor={row.rowId}
      value={{
        ur: row.typed[keys.ur] ?? '',
        en: row.typed[keys.en] ?? '',
        phone: row.typed[keys.phone] ?? '',
        marks: row.typed[keys.marks] ?? '',
      }}
      onChange={(patch) => {
        const typed: Record<string, string> = {};
        if (patch.ur !== undefined) typed[keys.ur] = patch.ur;
        if (patch.en !== undefined) typed[keys.en] = patch.en;
        if (patch.phone !== undefined) typed[keys.phone] = patch.phone;
        if (patch.marks !== undefined) typed[keys.marks] = patch.marks;
        store.getState().patchRow(row.rowId, { typed });
      }}
      side={side}
      paths={paths}
      error={error}
      onDone={onDone}
    />
  );
}
