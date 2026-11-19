// The real answers and requests of the backend for round 8 of the report engine: stock (R10), the
// vouchers list (R15), the receipts and payments registers (R23, R25), the trip profit or loss
// (R33, in its trip, partner, item and month layouts) and the day summary (R34). The backend wrote
// them in its own test (backend/crates/services/tests/fixtures); they are copied here by
// `scripts/copy-backend-fixtures.mjs`, never edited, and read as the generated types. Money in them
// is paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a branded
// type that a plain number is not. The worked day is E2E-36 (R34) and the trip is E2E-34b (R33).

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import printR33Partner from '../fixtures/reports_print_data_r33_partner_en.json';
import printR33PartnerRequest from '../fixtures/reports_print_data_r33_partner_en_request.json';
import printR34 from '../fixtures/reports_print_data_r34_en.json';
import printR34Request from '../fixtures/reports_print_data_r34_en_request.json';
import r10Stock from '../fixtures/reports_run_r10_stock.json';
import r10StockRequest from '../fixtures/reports_run_r10_stock_request.json';
import r15All from '../fixtures/reports_run_r15_all.json';
import r15AllRequest from '../fixtures/reports_run_r15_all_request.json';
import r15Payments from '../fixtures/reports_run_r15_payments.json';
import r15PaymentsRequest from '../fixtures/reports_run_r15_payments_request.json';
import r23Day from '../fixtures/reports_run_r23_day.json';
import r23DayRequest from '../fixtures/reports_run_r23_day_request.json';
import r25Day from '../fixtures/reports_run_r25_day.json';
import r25DayRequest from '../fixtures/reports_run_r25_day_request.json';
import r33Item from '../fixtures/reports_run_r33_item_e2e_34b.json';
import r33ItemRequest from '../fixtures/reports_run_r33_item_e2e_34b_request.json';
import r33Month from '../fixtures/reports_run_r33_month_e2e_34b.json';
import r33MonthRequest from '../fixtures/reports_run_r33_month_e2e_34b_request.json';
import r33Partner from '../fixtures/reports_run_r33_partner_e2e_34b.json';
import r33PartnerRequest from '../fixtures/reports_run_r33_partner_e2e_34b_request.json';
import r33Trip from '../fixtures/reports_run_r33_trip_e2e_34b.json';
import r33TripRequest from '../fixtures/reports_run_r33_trip_e2e_34b_request.json';
import r34Day from '../fixtures/reports_run_r34_e2e_36.json';
import r34DayRequest from '../fixtures/reports_run_r34_e2e_36_request.json';

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

/** Stock: the 50 کیلا of بکری 1, 20 sold, 30 left (E2E-16 style, Event 1 with two lines). */
export const R10_STOCK = readPage(r10Stock);
export const R10_STOCK_REQUEST = readRunInput(r10StockRequest);

/** Every voucher, no kind picked: the totals are empty and the summary has one line per kind. */
export const R15_ALL = readPage(r15All);
export const R15_ALL_REQUEST = readRunInput(r15AllRequest);
/** Payments only: one total, no summary. */
export const R15_PAYMENTS = readPage(r15Payments);
export const R15_PAYMENTS_REQUEST = readRunInput(r15PaymentsRequest);

export const R23_DAY = readPage(r23Day);
export const R23_DAY_REQUEST = readRunInput(r23DayRequest);
export const R25_DAY = readPage(r25Day);
export const R25_DAY_REQUEST = readRunInput(r25DayRequest);

/** The trip of E2E-34b: bought 20, sold 15, 5 written off, a loss of 1,940. */
export const R33_TRIP = readPage(r33Trip);
export const R33_TRIP_REQUEST = readRunInput(r33TripRequest);
export const R33_PARTNER = readPage(r33Partner);
export const R33_PARTNER_REQUEST = readRunInput(r33PartnerRequest);
export const R33_ITEM = readPage(r33Item);
export const R33_ITEM_REQUEST = readRunInput(r33ItemRequest);
export const R33_MONTH = readPage(r33Month);
export const R33_MONTH_REQUEST = readRunInput(r33MonthRequest);

/** The day summary of 10-10-2026 after Events 1 to 5 (E2E-36). */
export const R34_DAY = readPage(r34Day);
export const R34_DAY_REQUEST = readRunInput(r34DayRequest);

export const PRINT_R33_PARTNER_EN = readPrint(printR33Partner);
export const PRINT_R33_PARTNER_EN_REQUEST = readPrintInput(printR33PartnerRequest);
export const PRINT_R34_EN = readPrint(printR34);
export const PRINT_R34_EN_REQUEST = readPrintInput(printR34Request);
