// "ڈوبی رقم میں ڈالیں" on the dues reports (R20, R24, R32; PRD FR-10.10): moves the person of the
// row that has focus. It sits among the buttons of the report, so it is reached with Tab like the
// others, and it is dimmed until a row with a person has focus. Where the app frame has no dialog
// for it (a small test) it is not drawn.

import { Localized } from '@fluent/react';
import { UserRoundX } from 'lucide-react';
import type { ReportParty } from '../../../bindings/ReportParty';
import { Button, useDeadAmountActions } from '../../../shared/ui';

export function DeadMoveButton({ party }: { party: ReportParty | null }) {
  const actions = useDeadAmountActions();
  if (actions === null) return null;
  return (
    <Button
      disabled={party === null}
      onClick={() => {
        if (party === null) return;
        actions.move({
          accountId: party.account_id,
          personKey: party.person_key,
          names: { name_ur: party.name_ur, name_en: party.name_en, tag: party.tag },
        });
      }}
    >
      <UserRoundX aria-hidden="true" className="size-icon-md" />
      <Localized id="dead-move-button" />
    </Button>
  );
}
