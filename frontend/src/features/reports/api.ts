// Typed wrappers around the commands of the `reports` module: the print commands of plan 14 and the
// report engine of plan 20 (`reports_catalog`, `reports_run`, `reports_print_data`). Components
// never call invoke() (Code Standards F4). Every command takes one `{ input }` object, except
// `reports_catalog`, which takes none. All of them need the signed-in Owner (NFR-SEC-13), except
// that the hidden print window may call only `reports_bikri_print_data`,
// `reports_voucher_print_data` and `reports_print_data` (its capability allows nothing else).

import type { BikriPrintDataInput } from '../../bindings/BikriPrintDataInput';
import type { BuyerBillsPrintData } from '../../bindings/BuyerBillsPrintData';
import type { FindPeopleInput } from '../../bindings/FindPeopleInput';
import type { PrintData } from '../../bindings/PrintData';
import type { PrintInput } from '../../bindings/PrintInput';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import type { PrintVoucherInput } from '../../bindings/PrintVoucherInput';
import type { PukhtaPersonHit } from '../../bindings/PukhtaPersonHit';
import type { ExcelExported } from '../../bindings/ExcelExported';
import type { ReportCatalogEntry } from '../../bindings/ReportCatalogEntry';
import type { ReportPage } from '../../bindings/ReportPage';
import type { ReportPrintData } from '../../bindings/ReportPrintData';
import type { ReportPrintInput } from '../../bindings/ReportPrintInput';
import type { ReportRunInput } from '../../bindings/ReportRunInput';
import type { ReportsExportRequest } from '../../bindings/ReportsExportRequest';
import type { VoucherPrintDataInput } from '../../bindings/VoucherPrintDataInput';
import { callCommand } from '../../shared/api';

/**
 * Everything a template needs to print one document of one saved بکری, in the print language.
 * Every value is already formatted by the backend; the template adds up nothing (CLAUDE.md rule 1).
 */
export function reportsBikriPrintData(input: BikriPrintDataInput): Promise<PrintData> {
  return callCommand<PrintData>('reports_bikri_print_data', { input });
}

/**
 * Everything the receipt slip of one saved voucher needs, in the print language (PRD FR-15.2): the
 * party, one total, the amount in words and the facts. The template adds up nothing.
 */
export function reportsVoucherPrintData(input: VoucherPrintDataInput): Promise<PrintData> {
  return callCommand<PrintData>('reports_voucher_print_data', { input });
}

/** Prints the receipt slip of a saved voucher on the printer, or saves it as a PDF. Windows only. */
export function reportsPrintVoucher(input: PrintVoucherInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('reports_print_voucher', { input });
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

// ---- Plan 20: the report engine (PRD 19, Code Standards B31). Run B, D5 early track. ----

/** Every report that is built, with its group and the filter boxes it takes (PRD 19.2). */
export function reportsCatalog(): Promise<ReportCatalogEntry[]> {
  return callCommand<ReportCatalogEntry[]>('reports_catalog');
}

/**
 * One page of a report, with the totals over the whole filter (PRD FR-19.11). Send every key of
 * the filters; the backend refuses a filter the report does not take, if it is filled.
 */
export function reportsRun(input: ReportRunInput): Promise<ReportPage> {
  return callCommand<ReportPage>('reports_run', { input });
}

/** Everything a print of a report needs, with every row, in the print language (PRD FR-19.3). */
export function reportsPrintData(input: ReportPrintInput): Promise<ReportPrintData> {
  return callCommand<ReportPrintData>('reports_print_data', { input });
}

/**
 * Everything a print of ALL the credit buyers' slips of R5 needs, one slip for each buyer, with every
 * row (PRD 19.2 R5, "Print All"). The input is the one of the R5 print; any other report is refused.
 */
export function reportsBuyerBillsPrintData(input: ReportPrintInput): Promise<BuyerBillsPrintData> {
  return callCommand<BuyerBillsPrintData>('reports_buyer_bills_print_data', { input });
}

/**
 * The one-time (پختہ کھاتہ) people whose name or phone matches the text, newest first (PRD FR-5.4,
 * FR-13.9). The report filters of R9, R20 and the global search use it. The text is sent as typed.
 * `limit` 0 means 10.
 */
export function reportsFindPeople(input: FindPeopleInput): Promise<PukhtaPersonHit[]> {
  return callCommand<PukhtaPersonHit[]>('reports_find_people', { input });
}

/**
 * Saves EVERY row of a report as an Excel file at `path` (D5-Q1), with the sheet in `language`. The
 * progress comes by the event `export-progress` for `job_id`. The file is never opened or read back.
 */
export function reportsExportExcel(input: ReportsExportRequest): Promise<ExcelExported> {
  return callCommand<ExcelExported>('reports_export_excel', { input });
}
