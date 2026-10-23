// The two report buttons of the party card (PRD FR-7.65): کھاتہ (the ledger, R13) and فرد حساب (the
// statement, R14). The card is a shared component and knows no screen and no router; the app frame
// provides the function that opens a report, and a card rendered without one (a test, a gallery
// page) simply has no buttons. This is the one place where the card learns how to open a report.

import { createContext, useContext } from 'react';

/** The reports the card opens. */
export type PartyReportId = 'R13' | 'R14';

/** Opens a report for one account, with the date range already filled. */
export type OpenPartyReport = (report: PartyReportId, accountId: number) => void;

const PartyCardLinksContext = createContext<OpenPartyReport | null>(null);

/** Provided once by the app frame: `<PartyCardLinksProvider value={open}>`. */
export const PartyCardLinksProvider = PartyCardLinksContext.Provider;

/** The function that opens a report, or null when the card is shown where no report can open. */
export function usePartyCardLinks(): OpenPartyReport | null {
  return useContext(PartyCardLinksContext);
}
