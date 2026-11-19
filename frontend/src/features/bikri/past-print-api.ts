// The calls of the print and the Excel of پچھلی بکری (plan 72 P8). The page of the print is made from the
// data of `bikri_list_print_data`; Print and PDF go through `bikri_list_print` (a hidden print window,
// like every document of the app); Excel through `bikri_list_export_excel`. Components call these only
// through the hooks of this module (Code Standards F4).

import type { ExcelExported } from '../../bindings/ExcelExported';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import { callCommand } from '../../shared/api';
import type { BikriListExportRequest } from '../../bindings/BikriListExportRequest';
import type { BikriListPrintData } from '../../bindings/BikriListPrintData';
import type { BikriListPrintInput } from '../../bindings/BikriListPrintInput';
import type { PrintBikriListInput } from '../../bindings/PrintBikriListInput';

/** Everything a print of the list needs, with every row, in the print language. No paging. */
export function bikriListPrintData(input: BikriListPrintInput): Promise<BikriListPrintData> {
  return callCommand<BikriListPrintData>('bikri_list_print_data', { input });
}

/** Sends the list to the printer or saves it as a PDF (a Mac opens the system print window). */
export function bikriListPrint(input: PrintBikriListInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('bikri_list_print', { input });
}

/** Saves EVERY row of the list as an Excel file at `path`; the progress comes by `export-progress`. */
export function bikriListExportExcel(input: BikriListExportRequest): Promise<ExcelExported> {
  return callCommand<ExcelExported>('bikri_list_export_excel', { input });
}
