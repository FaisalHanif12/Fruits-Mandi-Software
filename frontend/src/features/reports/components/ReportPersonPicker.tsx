// The box that picks one known one-time person (PRD FR-5.4, FR-13.5, FR-13.9): the register of
// one-time people (R9) is for them, and any report on a پختہ کھاتہ account can be narrowed to one
// of them. The list is the backend's (`reports_find_people`); it matches a name in either script, a
// close English spelling or the phone. Like every search it takes the text as typed and never
// converts it to Urdu (Code Standards F41). The first match is highlighted, so a name typed and
// Enter takes him. Nothing is searched or ordered here.

import { PukhtaPersonRow } from '../../pukhta_khata';
import { displayName, useLanguage } from '../../../shared/i18n';
import { SuggestInput, type FieldControlProps } from '../../../shared/ui';
import { personFromHit, type ReportPerson } from '../filters';
import { usePeopleSearch } from '../hooks';

interface ReportPersonPickerProps extends FieldControlProps {
  /** The person picked, or null. */
  person: ReportPerson | null;
  /** What is typed while no person is picked. */
  text: string;
  /**
   * The one-time account the report is on, when a party is already picked: only people with lines
   * on it are listed. Null (R9): any one-time account, and his first account is used.
   */
  accountId: number | null;
  onTextChange: (text: string) => void;
  onPick: (person: ReportPerson) => void;
}

export function ReportPersonPicker({
  person,
  text,
  accountId,
  onTextChange,
  onPick,
  ...control
}: ReportPersonPickerProps) {
  const language = useLanguage();
  const typed = person === null ? text : '';
  const hits = usePeopleSearch(typed).data ?? [];
  const rows = typed.trim() === '' ? [] : hits;
  // The name of a person picked is written in the app language, so a language switch changes it.
  const shown = person === null ? text : displayName(person, language);
  return (
    <SuggestInput
      {...control}
      value={shown}
      onChange={onTextChange}
      highlightFirst
      suggestions={rows.flatMap((hit) => {
        const picked = personFromHit(hit, accountId);
        return picked === null
          ? []
          : [
              {
                value: hit.person_key,
                content: <PukhtaPersonRow person={hit} />,
                onPick: () => {
                  onPick(picked);
                },
              },
            ];
      })}
    />
  );
}
