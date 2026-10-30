// The heading of a ledger or a statement (R13, R14) of a person who is in the dead amount (PRD
// FR-7.38, FR-10.10): "ڈوبی رقم" with the date of the move and the reason, so nobody reads his
// ledger without knowing he is gone. "Make active again" is one press away where the app frame has
// the dialog. Nothing is drawn for a party who is not in the dead amount.

import { Localized } from '@fluent/react';
import type { ReportParty } from '../../../bindings/ReportParty';
import { DeadInfoLine, LinkButton, useDeadAmountActions } from '../../../shared/ui';

export function DeadHeading({ party }: { party: ReportParty | null | undefined }) {
  const actions = useDeadAmountActions();
  if (party?.dead === undefined) return null;
  const markId = party.dead.mark_id;
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
      <DeadInfoLine info={party.dead} />
      {actions === null ? null : (
        <LinkButton
          onClick={() => {
            actions.undo({
              markId,
              names: { name_ur: party.name_ur, name_en: party.name_en, tag: party.tag },
            });
          }}
        >
          <Localized id="dead-undo-button" />
        </LinkButton>
      )}
    </div>
  );
}
