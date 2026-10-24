// The real answers and requests of the backend for R35, the party day sheet (PRD FR-19.12, E2E-43):
// the table of every party of 10-10-2026 after Events 1 to 5 (the buyers, then the sellers, with
// the buyers' total and the sellers' totals in the footer lines), the slips of one buyer (اکرم) and
// one seller (رحیم), and the print data of the buyer's slip. The backend wrote them in its own test
// (backend/crates/services/tests/fixtures); they are copied here by `scripts/copy-backend-fixtures.mjs`,
// never edited, and read as the generated types. Money in them is paisa, as on the wire. A reader
// takes `unknown`, because the money in a binding is a branded type that a plain number is not.

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import printR35SlipAkram from '../fixtures/reports_print_data_r35_slip_akram_en.json';
import printR35SlipAkramRequest from '../fixtures/reports_print_data_r35_slip_akram_en_request.json';
import r35SlipAkram from '../fixtures/reports_run_r35_slip_akram.json';
import r35SlipAkramRequest from '../fixtures/reports_run_r35_slip_akram_request.json';
import r35SlipRahim from '../fixtures/reports_run_r35_slip_rahim.json';
import r35SlipRahimRequest from '../fixtures/reports_run_r35_slip_rahim_request.json';
import r35Table from '../fixtures/reports_run_r35_table.json';
import r35TableRequest from '../fixtures/reports_run_r35_table_request.json';

function readPage(value: unknown): ReportPage {
  return value as ReportPage;
}
function readRunInput(value: unknown): ReportRunInput {
  return value as ReportRunInput;
}
function readPrint(value: unknown): ReportPrintData {
  return value as ReportPrintData;
}
function readPrintInput(value: unknown): ReportPrintInput {
  return value as ReportPrintInput;
}

/** Every party of the day: buyers (total 21,564 بنام) then sellers, a blank collector column. */
export const R35_TABLE = readPage(r35Table);
export const R35_TABLE_REQUEST = readRunInput(r35TableRequest);
/** The slip of اکرم: one line, 20 نگ, 12,880 with commission and لاگا. */
export const R35_SLIP_AKRAM = readPage(r35SlipAkram);
export const R35_SLIP_AKRAM_REQUEST = readRunInput(r35SlipAkramRequest);
/** The slip of رحیم, a seller: one lot, the cash given and the payment of the day. */
export const R35_SLIP_RAHIM = readPage(r35SlipRahim);
export const R35_SLIP_RAHIM_REQUEST = readRunInput(r35SlipRahimRequest);
export const PRINT_R35_SLIP_AKRAM_EN = readPrint(printR35SlipAkram);
export const PRINT_R35_SLIP_AKRAM_EN_REQUEST = readPrintInput(printR35SlipAkramRequest);
