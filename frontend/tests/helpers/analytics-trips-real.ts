// The real answers and requests of the backend for plan 27 (analytics views F, G and J, and the new
// filters partner, trip, goods and trip layout). The backend wrote them in its own test
// (backend/crates/services/tests/fixtures/analytics_*.json); they are copied here, never edited, and
// read as the generated types. Money in them is paisa, as on the wire. A reader takes `unknown`,
// because the money in a binding is a branded type that a plain number is not.
//
// Some pairs have the same request and two answers: the same filters on another day of the test
// database (`_trip` and `_trip_written_off` are the E2E-34 trip before and after its write-off).

import type { AnalyticsLinesInput } from '../../src/bindings/AnalyticsLinesInput';
import type { AnalyticsLinesPage } from '../../src/bindings/AnalyticsLinesPage';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import type { AnalyticsPrintInput } from '../../src/bindings/AnalyticsPrintInput';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import type { UiError } from '../../src/bindings/UiError';
import anRunAGoodsConsigned from '../fixtures/analytics_run_a_goods_consigned.json';
import anRunAGoodsConsignedRequest from '../fixtures/analytics_run_a_goods_consigned_request.json';
import anRunAGoodsTrip from '../fixtures/analytics_run_a_goods_trip.json';
import anRunAGoodsTripRequest from '../fixtures/analytics_run_a_goods_trip_request.json';
import anRunF from '../fixtures/analytics_run_f.json';
import anRunFRequest from '../fixtures/analytics_run_f_request.json';
import anRunFTrip from '../fixtures/analytics_run_f_trip.json';
import anRunFTripRequest from '../fixtures/analytics_run_f_trip_request.json';
import anRunFCompareRefused from '../fixtures/analytics_run_f_compare_refused.json';
import anRunFCompareRefusedRequest from '../fixtures/analytics_run_f_compare_refused_request.json';
import anRunGDay from '../fixtures/analytics_run_g_day.json';
import anRunGDayRequest from '../fixtures/analytics_run_g_day_request.json';
import anRunGMonth from '../fixtures/analytics_run_g_month.json';
import anRunGMonthRequest from '../fixtures/analytics_run_g_month_request.json';
import anRunGTripDay from '../fixtures/analytics_run_g_trip_day.json';
import anRunGTripDayRequest from '../fixtures/analytics_run_g_trip_day_request.json';
import anRunGTripWrittenOff from '../fixtures/analytics_run_g_trip_written_off.json';
import anRunGTripWrittenOffRequest from '../fixtures/analytics_run_g_trip_written_off_request.json';
import anRunJEmpty from '../fixtures/analytics_run_j_empty.json';
import anRunJEmptyRequest from '../fixtures/analytics_run_j_empty_request.json';
import anRunJTrip from '../fixtures/analytics_run_j_trip.json';
import anRunJTripRequest from '../fixtures/analytics_run_j_trip_request.json';
import anRunJTripByPurchase from '../fixtures/analytics_run_j_trip_by_purchase.json';
import anRunJTripByPurchaseRequest from '../fixtures/analytics_run_j_trip_by_purchase_request.json';
import anRunJTripPartner from '../fixtures/analytics_run_j_trip_partner.json';
import anRunJTripPartnerRequest from '../fixtures/analytics_run_j_trip_partner_request.json';
import anRunJTripWrittenOff from '../fixtures/analytics_run_j_trip_written_off.json';
import anRunJTripWrittenOffRequest from '../fixtures/analytics_run_j_trip_written_off_request.json';
import anLinesFPartnerA from '../fixtures/analytics_lines_f_partner_a.json';
import anLinesFPartnerARequest from '../fixtures/analytics_lines_f_partner_a_request.json';
import anLinesJTrip from '../fixtures/analytics_lines_j_trip.json';
import anLinesJTripRequest from '../fixtures/analytics_lines_j_trip_request.json';
import anPrintDataJEn from '../fixtures/analytics_print_data_j_en.json';
import anPrintDataJEnRequest from '../fixtures/analytics_print_data_j_en_request.json';

function readPage(value: unknown): AnalyticsPage {
  return value as AnalyticsPage;
}
function readRunInput(value: unknown): AnalyticsRunInput {
  return value as AnalyticsRunInput;
}
function readLines(value: unknown): AnalyticsLinesPage {
  return value as AnalyticsLinesPage;
}
function readLinesInput(value: unknown): AnalyticsLinesInput {
  return value as AnalyticsLinesInput;
}
function readPrint(value: unknown): AnalyticsPrintData {
  return value as AnalyticsPrintData;
}
function readPrintInput(value: unknown): AnalyticsPrintInput {
  return value as AnalyticsPrintInput;
}
function readRefusal(value: unknown): UiError {
  return value as UiError;
}

export const ANALYTICS_RUN_A_GOODS_CONSIGNED = readPage(anRunAGoodsConsigned);
export const ANALYTICS_RUN_A_GOODS_CONSIGNED_REQUEST = readRunInput(anRunAGoodsConsignedRequest);
export const ANALYTICS_RUN_A_GOODS_TRIP = readPage(anRunAGoodsTrip);
export const ANALYTICS_RUN_A_GOODS_TRIP_REQUEST = readRunInput(anRunAGoodsTripRequest);
export const ANALYTICS_RUN_F = readPage(anRunF);
export const ANALYTICS_RUN_F_REQUEST = readRunInput(anRunFRequest);
export const ANALYTICS_RUN_F_TRIP = readPage(anRunFTrip);
export const ANALYTICS_RUN_F_TRIP_REQUEST = readRunInput(anRunFTripRequest);
export const ANALYTICS_RUN_F_COMPARE_REFUSED = readRefusal(anRunFCompareRefused);
export const ANALYTICS_RUN_F_COMPARE_REFUSED_REQUEST = readRunInput(anRunFCompareRefusedRequest);
export const ANALYTICS_RUN_G_DAY = readPage(anRunGDay);
export const ANALYTICS_RUN_G_DAY_REQUEST = readRunInput(anRunGDayRequest);
export const ANALYTICS_RUN_G_MONTH = readPage(anRunGMonth);
export const ANALYTICS_RUN_G_MONTH_REQUEST = readRunInput(anRunGMonthRequest);
export const ANALYTICS_RUN_G_TRIP_DAY = readPage(anRunGTripDay);
export const ANALYTICS_RUN_G_TRIP_DAY_REQUEST = readRunInput(anRunGTripDayRequest);
export const ANALYTICS_RUN_G_TRIP_WRITTEN_OFF = readPage(anRunGTripWrittenOff);
export const ANALYTICS_RUN_G_TRIP_WRITTEN_OFF_REQUEST = readRunInput(anRunGTripWrittenOffRequest);
export const ANALYTICS_RUN_J_EMPTY = readPage(anRunJEmpty);
export const ANALYTICS_RUN_J_EMPTY_REQUEST = readRunInput(anRunJEmptyRequest);
export const ANALYTICS_RUN_J_TRIP = readPage(anRunJTrip);
export const ANALYTICS_RUN_J_TRIP_REQUEST = readRunInput(anRunJTripRequest);
export const ANALYTICS_RUN_J_TRIP_BY_PURCHASE = readPage(anRunJTripByPurchase);
export const ANALYTICS_RUN_J_TRIP_BY_PURCHASE_REQUEST = readRunInput(anRunJTripByPurchaseRequest);
export const ANALYTICS_RUN_J_TRIP_PARTNER = readPage(anRunJTripPartner);
export const ANALYTICS_RUN_J_TRIP_PARTNER_REQUEST = readRunInput(anRunJTripPartnerRequest);
export const ANALYTICS_RUN_J_TRIP_WRITTEN_OFF = readPage(anRunJTripWrittenOff);
export const ANALYTICS_RUN_J_TRIP_WRITTEN_OFF_REQUEST = readRunInput(anRunJTripWrittenOffRequest);
export const ANALYTICS_LINES_F_PARTNER_A = readLines(anLinesFPartnerA);
export const ANALYTICS_LINES_F_PARTNER_A_REQUEST = readLinesInput(anLinesFPartnerARequest);
export const ANALYTICS_LINES_J_TRIP = readLines(anLinesJTrip);
export const ANALYTICS_LINES_J_TRIP_REQUEST = readLinesInput(anLinesJTripRequest);
export const ANALYTICS_PRINT_DATA_J_EN = readPrint(anPrintDataJEn);
export const ANALYTICS_PRINT_DATA_J_EN_REQUEST = readPrintInput(anPrintDataJEnRequest);
