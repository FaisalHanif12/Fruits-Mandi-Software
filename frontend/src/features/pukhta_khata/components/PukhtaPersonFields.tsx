// The one-time person of a lot, a line or a purchase (PRD FR-13.2, FR-7.24, FR-7.29): a name in
// both scripts and a phone, for the پختہ کھاتہ account. The user types one name; the other fills
// by itself and is checked (NamePair).
//
// One merged list (PRD FR-7.29): while a name or the phone is typed, the one-time people already
// recorded come first, each with both names, the phone and the day of his last line, then (in the
// name typed first) the Urdu spellings of the word. Taking a person puts his exact names and phone
// in the three fields, so all his lines stay under one person key (FR-13.5); the names then carry
// no spelling marks, since they are his, as recorded. The list is the backend's
// (`search_pukhta_people`); nothing is searched or ordered here.
//
// The value is the screen's own text (a row of the sale book, the purchase form), so a language
// switch keeps it (Code Standards F39). This piece holds no state and saves nothing.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { PukhtaPerson } from '../../../bindings/PukhtaPerson';
import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import type { UiError } from '../../../bindings/UiError';
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
import { usePukhtaPeople } from '../hooks';
import { PukhtaPersonRow } from './PukhtaPersonRow';

/** The one-time person as the screen keeps it: the names, the phone and the spelling marks as text. */
export interface PukhtaPersonValue {
  ur: string;
  en: string;
  phone: string;
  /** The spelling marks of the two names, as JSON text. */
  marks: string;
}

interface PukhtaPersonFieldsProps {
  value: PukhtaPersonValue;
  /** The fields that changed; the screen puts them in its own state. */
  onChange: (patch: Partial<PukhtaPersonValue>) => void;
  /** The seller's or the buyer's one-time account: the list shows the people of that side. */
  side: PukhtaSide;
  /** The field paths of the backend's problems for the three fields. */
  paths: { ur: string; en: string; phone: string };
  error: UiError | null;
  /** Enter on the last field: the screen takes focus on to the next field. */
  onDone: () => void;
  /** Names the block, so the screen can put focus on its first field (`data-pukhta-for`). */
  anchor: string;
  /** The labels of the three fields. Default: the sale book's. */
  labels?: { ur: ReactNode; en: ReactNode; phone: ReactNode } | undefined;
}

function isWords(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((word) => typeof word === 'string');
}

/** The marks kept as text. Anything that is not marks is no marks. */
export function readMarks(text: string | undefined): SpellingMarks {
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

/** No marks, as the text a screen keeps. */
export const NO_MARKS_TEXT = JSON.stringify(NO_MARKS);

export function PukhtaPersonFields({
  value,
  onChange,
  side,
  paths,
  error,
  onDone,
  anchor,
  labels,
}: PukhtaPersonFieldsProps) {
  const navigation = useFormNavigation(onDone);
  const language = useLanguage();
  const pair: NamePairValue = {
    urdu: value.ur,
    english: value.en,
    spelling: readMarks(value.marks),
  };
  // The people that fit the name typed first (Urdu in the Urdu app, English in the English app) and
  // the people that fit the phone. Each is one query of the backend.
  const nameText = language === 'ur' ? pair.urdu : pair.english;
  const byName = usePukhtaPeople(nameText, side).data ?? [];
  const byPhone = usePukhtaPeople(value.phone, side).data ?? [];
  const take = (person: PukhtaPerson) => {
    onChange({
      ur: person.name_ur,
      en: person.name_en,
      phone: person.phone,
      marks: NO_MARKS_TEXT,
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
  return (
    <div
      {...navigation}
      data-pukhta-for={anchor}
      className="grid grid-cols-1 gap-x-4 gap-y-3 normal:grid-cols-3"
    >
      <NamePair
        value={pair}
        onChange={(next) => {
          onChange({
            ur: next.urdu,
            en: next.english,
            marks: JSON.stringify(next.spelling ?? NO_MARKS),
          });
        }}
        urduLabel={labels?.ur ?? <Localized id="bikri-pukhta-name-ur" />}
        englishLabel={labels?.en ?? <Localized id="bikri-pukhta-name-en" />}
        urduError={fieldError(error, paths.ur)}
        englishError={fieldError(error, paths.en)}
        knownRecords={rowsOf(nameText.trim() === '' ? [] : byName)}
      />
      <Field
        label={labels?.phone ?? <Localized id="bikri-pukhta-phone" />}
        error={fieldError(error, paths.phone)}
      >
        {(control) => (
          <SuggestInput
            {...control}
            value={value.phone}
            onChange={(phone) => {
              onChange({ phone });
            }}
            suggestions={rowsOf(value.phone.trim() === '' ? [] : byPhone).map((extra) => ({
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
