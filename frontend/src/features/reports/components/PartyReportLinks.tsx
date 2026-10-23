// Gives the party card its two report buttons (PRD FR-7.65): کھاتہ opens the ledger (R13) and فرد حساب
// the statement (R14) of the account on the card. The app frame wraps the screens in it once; the
// card itself is shared and knows no router (shared/ui/party-card-links.ts).

import { useMemo, type ReactNode } from 'react';
import { PartyCardLinksProvider, type OpenPartyReport } from '../../../shared/ui/party-card-links';
import { partyReportSearch, useOpenReport } from '../open-report';

export function PartyReportLinks({ children }: { children: ReactNode }) {
  const open = useOpenReport();
  const openParty = useMemo<OpenPartyReport | null>(
    () =>
      open === null
        ? null
        : (report, accountId) => {
            open(report, partyReportSearch(accountId));
          },
    [open],
  );
  return <PartyCardLinksProvider value={openParty}>{children}</PartyCardLinksProvider>;
}
