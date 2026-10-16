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

import { Localized } from '@fluent/react';
import type { PukhtaPerson } from '../../../bindings/PukhtaPerson';
import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import type { UiError } from '../../../bindings/UiError';
import type { EntryRow, EntryStore } from '../../../shared/grid';
import { useLanguage } from '../../../shared/i18n';
import { useFormNavigation } from '../../../shared/keyboard';
import { NO_MARKS, type SpellingMarks } from '../../../shared/urdu/spelling';
import {
  Field,
  NamePair,
  SuggestInput,
  fieldError,
  type ListExtra,
  type NamePairValue,
} from '../../../shared/ui';
import { PukhtaPersonRow, usePukhtaPeople } from '../../pukhta_khata';

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

function isWords(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((word) => typeof word === 'string');
}

/** The marks kept on the row as text. Anything that is not marks is no marks. */
function readMarks(text: string | undefined): SpellingMarks {
  if (text === undefined || text === '') return NO_MARKS;
  try {
    const value: unknown = JSON.parse(text);
    if (typeof value !== 'object' || value === null) return NO_MARKS;
    const { urdu, english, confirmed } = value as Record<string, unknown>;
    return isWords(urdu) && isWords(english) && isWords(confirmed)
      ? { urdu, english, confirmed }
      : NO_MARKS;
  } catch {
    return NO_MARKS;
  }
}

export function PukhtaFields({ store, row, side, keys, paths, error, onDone }: PukhtaFieldsProps) {
  const navigation = useFormNavigation(onDone);
  const language = useLanguage();
  const value: NamePairValue = {
    urdu: row.typed[keys.ur] ?? '',
    english: row.typed[keys.en] ?? '',
    spelling: readMarks(row.typed[keys.marks]),
  };
  // The people that fit the name typed first (Urdu in the Urdu app, English in the English app) and
  // the people that fit the phone. Each is one query of the backend.
  const nameText = language === 'ur' ? value.urdu : value.english;
  const phoneText = row.typed[keys.phone] ?? '';
  const byName = usePukhtaPeople(nameText, side).data ?? [];
  const byPhone = usePukhtaPeople(phoneText, side).data ?? [];
  const take = (person: PukhtaPerson) => {
    store.getState().patchRow(row.rowId, {
      typed: {
        [keys.ur]: person.name_ur,
        [keys.en]: person.name_en,
        [keys.phone]: person.phone,
        [keys.marks]: JSON.stringify(NO_MARKS),
      },
    });
  };
  const rowsOf = (people: readonly PukhtaPerson[]): ListExtra[] =>
    people.map((person) => ({
      key: person.person_key,
      content: <PukhtaPersonRow person={person} />,
      onPick: () => {
        take(person);
      },
    }));
  const change = (next: NamePairValue) => {
    store.getState().patchRow(row.rowId, {
      typed: {
        [keys.ur]: next.urdu,
        [keys.en]: next.english,
        [keys.marks]: JSON.stringify(next.spelling ?? NO_MARKS),
      },
    });
  };
  return (
    <div
      {...navigation}
      data-pukhta-for={row.rowId}
      className="grid grid-cols-1 gap-x-4 gap-y-3 normal:grid-cols-3"
    >
      <NamePair
        value={value}
        onChange={change}
        urduLabel={<Localized id="bikri-pukhta-name-ur" />}
        englishLabel={<Localized id="bikri-pukhta-name-en" />}
        urduError={fieldError(error, paths.ur)}
        englishError={fieldError(error, paths.en)}
        knownRecords={rowsOf(nameText.trim() === '' ? [] : byName)}
      />
      <Field label={<Localized id="bikri-pukhta-phone" />} error={fieldError(error, paths.phone)}>
        {(control) => (
          <SuggestInput
            {...control}
            value={phoneText}
            onChange={(phone) => {
              store.getState().updateTyped(row.rowId, keys.phone, phone);
            }}
            suggestions={rowsOf(phoneText.trim() === '' ? [] : byPhone).map((extra) => ({
              value: extra.key,
              content: extra.content,
              onPick: extra.onPick,
            }))}
          />
        )}
      </Field>
    </div>
  );
}
