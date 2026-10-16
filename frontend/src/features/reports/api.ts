// Typed wrappers around the print commands of the `reports` module (plan 14 contract). Components
// never call invoke() (Code Standards F4). Every command takes one `{ input }` object. All of
// them need the signed-in Owner (NFR-SEC-13), except that the hidden print window may call only
// `reports_bikri_print_data` (its capability allows nothing else).

import type { BikriPrintDataInput } from '../../bindings/BikriPrintDataInput';
import type { PrintData } from '../../bindings/PrintData';
import type { PrintInput } from '../../bindings/PrintInput';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import { callCommand } from '../../shared/api';

/**
 * Everything a template needs to print one document of one saved بکری, in the print language.
 * Every value is already formatted by the backend; the template adds up nothing (CLAUDE.md rule 1).
 */
export function reportsBikriPrintData(input: BikriPrintDataInput): Promise<PrintData> {
  return callCommand<PrintData>('reports_bikri_print_data', { input });
}

/**
 * Prints the documents on the printer or saves them as PDF files. Windows only: elsewhere the
 * backend answers `ERR_PRINT_WINDOWS_ONLY` and the screen offers the preview instead.
 */
export function reportsPrint(input: PrintInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('reports_print', { input });
}

/** The names of the printers of this computer. Empty when the list is not available. */
export function reportsListPrinters(): Promise<string[]> {
  return callCommand<string[]>('reports_list_printers');
}
