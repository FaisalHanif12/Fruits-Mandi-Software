// Who the page is for: the role (seller, buyer, one-time buyer), the name with its tag, the short code
// and the phone (PRD FR-19.13). The name is in the print language already. A seller or buyer shows
// the short code the backend sends (`1` for 03040001); an account that has none shows its full code.
// A one-time person has a phone and no code.

import { useLocalization } from '@fluent/react';
import { Code } from '../ui/Code';
import { PrintName } from './PrintName';
import { LAYOUT_IDS, type PrintParty } from './types';

export function PrintPartyBlock({ party }: { party: PrintParty }) {
  const { l10n } = useLocalization();
  const code = party.short_code ?? party.code;
  return (
    <div className="print-party">
      <span className="print-party-role">{l10n.getString(party.role_id)}</span>
      <span className="print-party-name">
        <PrintName name={party.name} tag={party.tag} />
      </span>
      {code === null ? null : (
        <span className="print-party-extra">
          {l10n.getString(LAYOUT_IDS.partyCode)} <Code>{code}</Code>
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
