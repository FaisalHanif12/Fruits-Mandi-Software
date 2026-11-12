// Typed wrappers around the two commands of the `party_ledgers` module (plan 66, PRD FR-12.43).
// Components never call invoke() (Code Standards F4). Both are read only and need the signed-in
// Owner (NFR-SEC-4). Level 3, the day slip, is not here: it is the R35 report of the `reports`
// module for one party and one date.

import type { PartyLedgerGoTo } from '../../bindings/PartyLedgerGoTo';
import type { PartyLedgerGoToInput } from '../../bindings/PartyLedgerGoToInput';
import type { PartyLedgersList } from '../../bindings/PartyLedgersList';
import type { PartyLedgersListInput } from '../../bindings/PartyLedgersListInput';
import type { PartyLedgerStatement } from '../../bindings/PartyLedgerStatement';
import type { PartyLedgerStatementInput } from '../../bindings/PartyLedgerStatementInput';
import { callCommand } from '../../shared/api';

/**
 * Level 1: the buyers or the sellers, one row each, by code, with the sums of the debit and the
 * credit balances over the whole filter (PRD FR-19.11). The shared چکتا account is never listed
 * (PRD FR-13.6).
 */
export function partyLedgersList(input: PartyLedgersListInput): Promise<PartyLedgersList> {
  return callCommand<PartyLedgersList>('party_ledgers_list', { input });
}

/**
 * Level 2: one party's statement, a row for each date on which he has an entry, newest first, with
 * the totals over every date the filter matches.
 */
export function partyLedgersStatement(
  input: PartyLedgerStatementInput,
): Promise<PartyLedgerStatement> {
  return callCommand<PartyLedgerStatement>('party_ledgers_statement', { input });
}

/**
 * Go to a date of the statement (PRD FR-12.47, plan 74 W5): `date` is `DD-MM-YYYY` or a shortcut. The
 * answer is the 30-day window that holds it (it ends on the earlier of the day plus 15 days and
 * today), that window's own statement page and totals, whether the party has an entry on the date,
 * and the place of that date among the rows. Read only. A day after today is refused.
 */
export function partyLedgersGoToDate(input: PartyLedgerGoToInput): Promise<PartyLedgerGoTo> {
  return callCommand<PartyLedgerGoTo>('party_ledgers_go_to_date', { input });
}
