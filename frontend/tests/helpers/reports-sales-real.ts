// The real answers and requests of the backend for round 12 of the report engine: R1, the sale book
// (the lots of one بکری, with its refusal when no بکری is picked), and R5, the credit buyers' details
// (one row for each credit buyer, with his sale lines), with the print of R5. The backend wrote them
// in its own test (backend/crates/services/tests/fixtures); they are copied here by
// `scripts/copy-backend-fixtures.mjs`, never edited, and read as the generated types. Money in them is
// paisa, as on the wire. The day is the worked day (E2E-36): بکری 1 of 10-10-2026.

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import type { UiError } from '../../src/bindings/UiError';
import printR5 from '../fixtures/reports_print_data_r5_en.json';
import printR5Request from '../fixtures/reports_print_data_r5_en_request.json';
import r1Bikri1 from '../fixtures/reports_run_r1_bikri_1.json';
import r1Bikri1Request from '../fixtures/reports_run_r1_bikri_1_request.json';
import r1NeedsABikri from '../fixtures/reports_run_r1_needs_a_bikri.json';
import r1NeedsABikriRequest from '../fixtures/reports_run_r1_needs_a_bikri_request.json';
import r5Credit from '../fixtures/reports_run_r5_credit_buyers.json';
import r5CreditRequest from '../fixtures/reports_run_r5_credit_buyers_request.json';

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
function readRefusal(value: unknown): UiError {
  return value as UiError;
}

/** The sale book of بکری 1: one lot of رحیم, 50 کیلا, خام 29,400, net 19,316. */
export const R1_BIKRI_1 = readPage(r1Bikri1);
export const R1_BIKRI_1_REQUEST = readRunInput(r1Bikri1Request);
/** The refusal when no بکری is picked: ERR_REQUIRED on `filters.bikri_id`. */
export const R1_NEEDS_A_BIKRI = readRefusal(r1NeedsABikri);
export const R1_NEEDS_A_BIKRI_REQUEST = readRunInput(r1NeedsABikriRequest);
/** The credit buyers of 10-10-2026: ارشد and اکرم, each with his sale lines. */
export const R5_CREDIT_BUYERS = readPage(r5Credit);
export const R5_CREDIT_BUYERS_REQUEST = readRunInput(r5CreditRequest);
export const PRINT_R5_EN = readPrint(printR5);
export const PRINT_R5_EN_REQUEST = readPrintInput(printR5Request);
