// The one-time person of a lot or a line (PRD FR-13.2, FR-7.24, FR-7.29): a name in both scripts
// and a phone, for the پختہ کھاتہ account. The user types one name; the other fills by itself and
// is checked (NamePair). Plan 13 adds the list of known one-time people over these fields.
//
// The names, the phone and the spelling marks are text on the row, so a language switch keeps
// them (Code Standards F39). Nothing is saved here.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import type { EntryRow, EntryStore } from '../../../shared/grid';
import { useFormNavigation } from '../../../shared/keyboard';
import { NO_MARKS, type SpellingMarks } from '../../../shared/urdu/spelling';
import { Field, NamePair, TextInput, fieldError, type NamePairValue } from '../../../shared/ui';

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

export function PukhtaFields({ store, row, keys, paths, error, onDone }: PukhtaFieldsProps) {
  const navigation = useFormNavigation(onDone);
  const value: NamePairValue = {
    urdu: row.typed[keys.ur] ?? '',
    english: row.typed[keys.en] ?? '',
    spelling: readMarks(row.typed[keys.marks]),
  };
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
      />
      <Field label={<Localized id="bikri-pukhta-phone" />} error={fieldError(error, paths.phone)}>
        {(control) => (
          <TextInput
            {...control}
            value={row.typed[keys.phone] ?? ''}
            onChange={(phone) => {
              store.getState().updateTyped(row.rowId, keys.phone, phone);
            }}
          />
        )}
      </Field>
    </div>
  );
}
