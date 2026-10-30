// The picker for the accounts of one control that has people or khatas of its own: the persons who
// can borrow (control 0103, PRD 16.5) and the الگ کھاتے (control 0305, PRD 16.6). It is the shared
// account picker's base with the control found by its code, the party card under it, and the add row
// (or `F4`) asking the screen to add one, because adding such an account is
// its own action with its own command (`lent_person_create`, `khata_create`), not the plain
// account form. Pickers never convert English letters to Urdu (F41): the text is searched as typed.

import { useMemo, useState } from 'react';
import { accountSource, useCoaControls, type AccountHit } from '../accounts';
import { Balance } from './Balance';
import type { FieldControlProps } from './Field';
import { PartyCard } from './PartyCard';
import { PickerBase, type PickerValue } from './PickerBase';

interface ControlAccountPickerProps extends FieldControlProps {
  /** The code of the control whose accounts the field takes, for example `0103`. */
  controlCode: string;
  value: PickerValue | null;
  /** The account chosen, or null when the field was emptied. */
  onChange: (account: AccountHit | null) => void;
  /**
   * Called with the typed text from the add row and from `F4` (plan 41). The screen opens its own
   * form and puts the new account in the field itself. Leave out and the field has no add row.
   */
  onAddNew?: ((text: string) => void) | undefined;
  placeholder?: string | undefined;
  /** Show the party card under the field. Default: yes (PRD FR-7.38). */
  showCard?: boolean | undefined;
  /** A person in the dead amount may be offered (the ادھار رقم واپسی field, PRD FR-10.10). */
  includeDead?: boolean | undefined;
}

/** The word of each control's accounts in the add row's and the empty list's messages (plan 41). */
const ADD_NOUN: Readonly<Record<string, string>> = {
  // The persons who can borrow, and the separate khatas.
  '0103': 'lent-person',
  '0305': 'khata',
};

function nounOf(controlCode: string): string {
  return ADD_NOUN[controlCode] ?? 'account';
}

export function ControlAccountPicker({
  controlCode,
  value,
  onChange,
  onAddNew,
  placeholder,
  showCard = true,
  includeDead = false,
  readOnly,
  ...field
}: ControlAccountPickerProps) {
  const controls = useCoaControls();
  const controlId = controls.data?.find((control) => control.code === controlCode)?.id ?? null;
  const source = useMemo(
    () => accountSource('control', controlId, includeDead),
    [controlId, includeDead],
  );
  const [highlight, setHighlight] = useState<AccountHit | null>(null);
  const cardId = highlight?.id ?? value?.id ?? null;
  return (
    <div className="flex flex-col gap-2">
      <PickerBase<AccountHit>
        {...field}
        // Until the control is known the field cannot search, so it is not offered for typing.
        readOnly={readOnly === true || controlId === null}
        source={source}
        value={value}
        onChange={onChange}
        onHighlight={setHighlight}
        onAddNew={onAddNew}
        addNoun={onAddNew === undefined ? undefined : nounOf(controlCode)}
        placeholder={placeholder}
        renderEnd={(hit) => (hit.balance === null ? null : <Balance balance={hit.balance} />)}
      />
      {showCard ? <PartyCard accountId={cardId} /> : null}
    </div>
  );
}
