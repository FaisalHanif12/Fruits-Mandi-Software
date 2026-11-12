// The statement print commands (plan 74 T3, PRD FR-12.47). The only file of the print that knows the wire
// shape (Code Standards F4). Nothing here works out a figure: the backend builds the data from the
// functions of R13, R14 and R35.

import type { PrintOutcome } from '../../../bindings/PrintOutcome';
import { callCommand } from '../../../shared/api';
import type { PrintStatementInput } from '../../../bindings/PrintStatementInput';
import type { StatementPrintData } from '../../../bindings/StatementPrintData';
import type { StatementPrintDataInput } from '../../../bindings/StatementPrintDataInput';

/** The statement of one party for a period, with every row, ready to print. */
export function partyLedgersStatementPrintData(
  input: StatementPrintDataInput,
): Promise<StatementPrintData> {
  return callCommand<StatementPrintData>('party_ledgers_statement_print_data', { input });
}

/** Sends the statement to the printer or saves it as a PDF (a Mac opens the system print window). */
export function partyLedgersStatementPrint(input: PrintStatementInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('party_ledgers_statement_print', { input });
}
