// The public face of the partners module (Tech Stack 5.3). Other modules import only from here.
// The first-run wizard shows the same setup form as the setup page (NFR-INS-2).
export { PartnerSetupForm } from './components/PartnerSetupForm';
export { usePartnerSetupStore } from './setup-store';
// The running total of typed shares (the backend adds them up) and its box, for the new trip form
// (PRD FR-12.29): the shares of a trip total 100% like the firm's.
export { SharesTotal, type TotalState } from './components/SharesTotal';
export { useSharesCheck } from './hooks';
export type { ShareInput, SharesCheck } from './types';
// The list of partners, for the pickers of the entry screens (plan 17: the خرچہ screen).
export { PARTNERS_KEY, usePartnerList } from './hooks';
export type { PartnerRow } from './types';
