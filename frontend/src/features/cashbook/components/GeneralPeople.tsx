// The one-time person of each line on a پختہ کھاتہ account (PRD FR-13.5, FR-7.29, 16.4): money kept
// under that account is kept per person, so a line on it must name the person. His name or phone is
// typed in one field with the one merged list of known people; a person is taken from the list, so
// all his lines stay under one key. The panel shows only the lines that need it. The person's name
// then shows in the grid. What is typed here lives above the grid (F39).

import { Localized } from '@fluent/react';
import { displayName, useLanguage } from '../../../shared/i18n';
import type { EntryRow } from '../../../shared/grid';
import { Card, Field, fieldError } from '../../../shared/ui';
import type { UiError } from '../../../bindings/UiError';
import { PUKHTA_BUYER_CODE, PukhtaPersonPicker, isPukhtaCode } from '../../pukhta_khata';
import { GENERAL_ACCOUNT, type PeopleMap, type PersonChoice } from '../general-draft';

interface GeneralPeopleProps {
  rows: readonly EntryRow[];
  people: PeopleMap;
  onChange: (rowId: string, choice: PersonChoice) => void;
  /** The person was taken from the list: the cursor goes back to the line. */
  onPicked: (rowId: string) => void;
  /** What the backend refused, by the row it points at: shown under that line's person field. */
  errorOf: (rowId: string) => UiError | null;
}

/** Attribute that lets the grid's Enter find the person field of a row. */
export const PERSON_ROW_ATTRIBUTE = 'data-person-row';

export function GeneralPeople({ rows, people, onChange, onPicked, errorOf }: GeneralPeopleProps) {
  const language = useLanguage();
  const needing = rows
    .map((row, index) => ({ row, index }))
    .filter(({ row }) => isPukhtaCode(row.picked?.[GENERAL_ACCOUNT]?.code));
  if (needing.length === 0) return null;
  return (
    <Card compact title={<Localized id="cashbook-general-people-title" />}>
      <Localized id="cashbook-general-people-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <div className="grid grid-cols-1 gap-3 normal:grid-cols-2">
        {needing.map(({ row, index }) => {
          const account = row.picked?.[GENERAL_ACCOUNT];
          const choice = people[row.rowId] ?? { text: '', person: null };
          return (
            <div key={row.rowId} {...{ [PERSON_ROW_ATTRIBUTE]: row.rowId }}>
              <Field
                label={
                  <Localized
                    id="cashbook-general-person-of"
                    vars={{
                      line: String(index + 1),
                      account: account === undefined ? '' : displayName(account, language),
                    }}
                  />
                }
                error={fieldError(errorOf(row.rowId), 'person_key')}
              >
                {(props) => (
                  <PukhtaPersonPicker
                    {...props}
                    side={account?.code === PUKHTA_BUYER_CODE ? 'buyer' : 'seller'}
                    text={choice.text}
                    onTextChange={(text) => {
                      // Typing again makes the earlier pick wrong: the person is taken from the list.
                      onChange(row.rowId, { text, person: null });
                    }}
                    onPick={(person) => {
                      onChange(row.rowId, {
                        text: displayName(person, language),
                        person,
                      });
                      onPicked(row.rowId);
                    }}
                  />
                )}
              </Field>
            </div>
          );
        })}
      </div>
    </Card>
  );
}
