// The real answers and requests of the backend for the garden slice of round 11 in the report engine:
// R36, the gardens report (by garden and by partner), the day summary R34 with its two garden rows,
// and the print of R36 by partner. The backend wrote them in its own test
// (backend/crates/services/tests/fixtures); they are copied here by `scripts/copy-backend-fixtures.mjs`,
// never edited, and read as the generated types. Money in them is paisa, as on the wire. The garden
// world is E2E-40: Shahpur, الف 60% from the shop and ب 40% from his pocket, closed on 13-10-26.

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import r36Garden from '../fixtures/reports_run_r36_garden.json';
import r36GardenRequest from '../fixtures/reports_run_r36_garden_request.json';
import r36Partner from '../fixtures/reports_run_r36_partner.json';
import r36PartnerRequest from '../fixtures/reports_run_r36_partner_request.json';
import r34Garden from '../fixtures/reports_run_r34_garden_day.json';
import r34GardenRequest from '../fixtures/reports_run_r34_garden_day_request.json';
import printR36 from '../fixtures/reports_print_data_r36_partner_en.json';
import printR36Request from '../fixtures/reports_print_data_r36_partner_en_request.json';

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

export const R36_GARDEN = readPage(r36Garden);
export const R36_GARDEN_REQUEST = readRunInput(r36GardenRequest);
export const R36_PARTNER = readPage(r36Partner);
export const R36_PARTNER_REQUEST = readRunInput(r36PartnerRequest);
export const R34_GARDEN_DAY = readPage(r34Garden);
export const R34_GARDEN_DAY_REQUEST = readRunInput(r34GardenRequest);
export const PRINT_R36_PARTNER_EN = readPrint(printR36);
export const PRINT_R36_PARTNER_EN_REQUEST = readPrintInput(printR36Request);
