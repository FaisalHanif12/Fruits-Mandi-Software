// The field that picks a known one-time person (PRD FR-13.7, FR-7.29): a receipt from a one-time
// buyer or a payment to a one-time seller names the person whose open lines it is for. He must
// already be recorded, so unlike the name fields of the sale book this one ends with a person taken
// from the list: the first match is highlighted, so a name or a phone typed and Enter takes him.
// The list is the backend's (`search_pukhta_people`); nothing is searched or ordered here.

import type { PukhtaPerson } from '../../../bindings/PukhtaPerson';
import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import { SuggestInput, type FieldControlProps } from '../../../shared/ui';
import { usePukhtaPeople } from '../hooks';
import { PukhtaPersonRow } from './PukhtaPersonRow';

interface PukhtaPersonPickerProps extends FieldControlProps {
  /** The buyer's or the seller's one-time account: the list shows the people of that side. */
  side: PukhtaSide;
  /** The text in the field. */
  text: string;
  onTextChange: (text: string) => void;
  /** A person was taken from the list. */
  onPick: (person: PukhtaPerson) => void;
  /**
   * A person in the dead amount is listed too, with the tag "ڈوبی" (PRD FR-10.10): only the receipt
   * screen says yes, because a recovery from him is allowed.
   */
  includeDead?: boolean | undefined;
}

export function PukhtaPersonPicker({
  side,
  text,
  onTextChange,
  onPick,
  includeDead = false,
  ...control
}: PukhtaPersonPickerProps) {
  const people = usePukhtaPeople(text, side, includeDead).data ?? [];
  return (
    <SuggestInput
      {...control}
      value={text}
      onChange={onTextChange}
      highlightFirst
      suggestions={(text.trim() === '' ? [] : people).map((person) => ({
        value: person.person_key,
        content: <PukhtaPersonRow person={person} />,
        onPick: () => {
          onPick(person);
        },
      }))}
    />
  );
}
