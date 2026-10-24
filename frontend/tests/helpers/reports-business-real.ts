// The real answers and requests of the backend for round 7 of the report engine: the cash book (R12),
// the expense report (R29), the partner statement (R31) and the money lent report (R32). The backend
// wrote them in its own test (backend/crates/services/tests/fixtures); they are copied here by
// `scripts/copy-backend-fixtures.mjs`, never edited, and read as the generated types. Money in
// them is paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a
// branded type that a plain number is not. The ids in the requests are those of the worked day in a
// fresh test database (partner الف is 1, partner ب is 2).

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import type { UiError } from '../../src/bindings/UiError';
import printR31BayEn from '../fixtures/reports_print_data_r31_bay_en.json';
import printR31BayEnRequest from '../fixtures/reports_print_data_r31_bay_en_request.json';
import r12Day from '../fixtures/reports_run_r12_day.json';
import r12DayRequest from '../fixtures/reports_run_r12_day_request.json';
import r29Alif from '../fixtures/reports_run_r29_alif.json';
import r29AlifRequest from '../fixtures/reports_run_r29_alif_request.json';
import r29Day from '../fixtures/reports_run_r29_day.json';
import r29DayRequest from '../fixtures/reports_run_r29_day_request.json';
import r31Bay from '../fixtures/reports_run_r31_bay.json';
import r31BayRequest from '../fixtures/reports_run_r31_bay_request.json';
import r31PartnerRequired from '../fixtures/reports_run_r31_partner_required.json';
import r31PartnerRequiredRequest from '../fixtures/reports_run_r31_partner_required_request.json';
import r32E2e30b from '../fixtures/reports_run_r32_e2e_30b.json';
import r32E2e30bRequest from '../fixtures/reports_run_r32_e2e_30b_request.json';

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

export const R12_DAY = readPage(r12Day);
export const R12_DAY_REQUEST = readRunInput(r12DayRequest);

/** One partner (الف, id 1), all dates. */
export const R29_ALIF = readPage(r29Alif);
export const R29_ALIF_REQUEST = readRunInput(r29AlifRequest);
/** All partners, one day. */
export const R29_DAY = readPage(r29Day);
export const R29_DAY_REQUEST = readRunInput(r29DayRequest);

/** The statement of partner ب (id 2): income shares, a shop expense and his capital in. */
export const R31_BAY = readPage(r31Bay);
export const R31_BAY_REQUEST = readRunInput(r31BayRequest);
/** R31 asked with no partner: refused under the partner box (`filters.partner_id`). */
export const R31_PARTNER_REQUIRED = readRefusal(r31PartnerRequired);
export const R31_PARTNER_REQUIRED_REQUEST = readRunInput(r31PartnerRequiredRequest);

/** Money lent after E2E-30b (a loan past its return-by day). */
export const R32_E2E_30B = readPage(r32E2e30b);
export const R32_E2E_30B_REQUEST = readRunInput(r32E2e30bRequest);

export const PRINT_R31_BAY_EN = readPrint(printR31BayEn);
export const PRINT_R31_BAY_EN_REQUEST = readPrintInput(printR31BayEnRequest);
