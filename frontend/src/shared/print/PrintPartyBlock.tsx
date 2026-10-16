// Who the page is for: the role (seller, buyer, one-time buyer), the name, the code and the phone.
// The name is in the print language already. A one-time person has a phone and no code.

import { useLocalization } from '@fluent/react';
import { Code } from '../ui/Code';
import { ScriptText } from '../ui/ScriptText';
import { LAYOUT_IDS, type PrintParty } from './types';

export function PrintPartyBlock({ party }: { party: PrintParty }) {
  const { l10n } = useLocalization();
  return (
    <div className="print-party">
      <span className="print-party-role">{l10n.getString(party.role_id)}</span>
      <span className="print-party-name">
        <ScriptText>{party.name}</ScriptText>
      </span>
      {party.code === null ? null : (
        <span className="print-party-extra">
          {l10n.getString(LAYOUT_IDS.partyCode)} <Code>{party.code}</Code>
        </span>
      )}
      {party.phone === null ? null : (
        <span className="print-party-extra">
          {l10n.getString(LAYOUT_IDS.partyPhone)} <Code>{party.phone}</Code>
        </span>
      )}
    </div>
  );
}
