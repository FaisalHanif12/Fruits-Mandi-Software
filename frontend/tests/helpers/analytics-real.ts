// The real answers and requests of the backend for plan 21 (analytics views A, B, C, D, E and H). The
// backend wrote them in its own test (backend/crates/services/tests/fixtures/analytics_*.json); they
// are copied here, never edited, and read as the generated types. Money in them is paisa, as on the
// wire. A reader takes `unknown`, because the money in a binding is a branded type that a plain
// number is not. The ids in the requests are those of the worked day in a fresh test database.

import type { AnalyticsCatalogEntry } from '../../src/bindings/AnalyticsCatalogEntry';
import type { AnalyticsLinesInput } from '../../src/bindings/AnalyticsLinesInput';
import type { AnalyticsLinesPage } from '../../src/bindings/AnalyticsLinesPage';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import type { AnalyticsPrintInput } from '../../src/bindings/AnalyticsPrintInput';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import type { AnalyticsSavedView } from '../../src/bindings/AnalyticsSavedView';
import type { AnalyticsViewsSaveInput } from '../../src/bindings/AnalyticsViewsSaveInput';
import type { UiError } from '../../src/bindings/UiError';
import anRunA from '../fixtures/analytics_run_a.json';
import anRunARequest from '../fixtures/analytics_run_a_request.json';
import anRunACompare from '../fixtures/analytics_run_a_compare.json';
import anRunACompareRequest from '../fixtures/analytics_run_a_compare_request.json';
import anRunAPage2 from '../fixtures/analytics_run_a_page2.json';
import anRunAPage2Request from '../fixtures/analytics_run_a_page2_request.json';
import anRunASellerSide from '../fixtures/analytics_run_a_seller_side.json';
import anRunASellerSideRequest from '../fixtures/analytics_run_a_seller_side_request.json';
import anRunB from '../fixtures/analytics_run_b.json';
import anRunBRequest from '../fixtures/analytics_run_b_request.json';
import anRunC from '../fixtures/analytics_run_c.json';
import anRunCRequest from '../fixtures/analytics_run_c_request.json';
import anRunDDay from '../fixtures/analytics_run_d_day.json';
import anRunDDayRequest from '../fixtures/analytics_run_d_day_request.json';
import anRunDMonth from '../fixtures/analytics_run_d_month.json';
import anRunDMonthRequest from '../fixtures/analytics_run_d_month_request.json';
import anRunE from '../fixtures/analytics_run_e.json';
import anRunERequest from '../fixtures/analytics_run_e_request.json';
import anRunH from '../fixtures/analytics_run_h.json';
import anRunHRequest from '../fixtures/analytics_run_h_request.json';
import anRunACompareNeedsDates from '../fixtures/analytics_run_a_compare_needs_dates.json';
import anRunACompareNeedsDatesRequest from '../fixtures/analytics_run_a_compare_needs_dates_request.json';
import anRunAGroupByRefused from '../fixtures/analytics_run_a_group_by_refused.json';
import anRunAGroupByRefusedRequest from '../fixtures/analytics_run_a_group_by_refused_request.json';
import anRunAUnknownItem from '../fixtures/analytics_run_a_unknown_item.json';
import anRunAUnknownItemRequest from '../fixtures/analytics_run_a_unknown_item_request.json';
import anLinesABanana from '../fixtures/analytics_lines_a_banana.json';
import anLinesABananaRequest from '../fixtures/analytics_lines_a_banana_request.json';
import anLinesDBananaOctober from '../fixtures/analytics_lines_d_banana_october.json';
import anLinesDBananaOctoberRequest from '../fixtures/analytics_lines_d_banana_october_request.json';
import anLinesERahimBanana from '../fixtures/analytics_lines_e_rahim_banana.json';
import anLinesERahimBananaRequest from '../fixtures/analytics_lines_e_rahim_banana_request.json';
import anPrintDataAEn from '../fixtures/analytics_print_data_a_en.json';
import anPrintDataAEnRequest from '../fixtures/analytics_print_data_a_en_request.json';
import anPrintDataDUr from '../fixtures/analytics_print_data_d_ur.json';
import anPrintDataDUrRequest from '../fixtures/analytics_print_data_d_ur_request.json';
import anCatalog from '../fixtures/analytics_catalog.json';
import anViewsGetEmpty from '../fixtures/analytics_views_get_empty.json';
import anViewsGetTwo from '../fixtures/analytics_views_get_two.json';
import anViewsSaveTwo from '../fixtures/analytics_views_save_two.json';
import anViewsSaveTwoRequest from '../fixtures/analytics_views_save_two_request.json';
import anViewsSaveBad from '../fixtures/analytics_views_save_bad.json';
import anViewsSaveBadRequest from '../fixtures/analytics_views_save_bad_request.json';

function readCatalog(value: unknown): AnalyticsCatalogEntry[] {
  return value as AnalyticsCatalogEntry[];
}
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
function readViews(value: unknown): AnalyticsSavedView[] {
  return value as AnalyticsSavedView[];
}
function readSaveInput(value: unknown): AnalyticsViewsSaveInput {
  return value as AnalyticsViewsSaveInput;
}
function readRefusal(value: unknown): UiError {
  return value as UiError;
}

export const ANALYTICS_RUN_A = readPage(anRunA);
export const ANALYTICS_RUN_A_REQUEST = readRunInput(anRunARequest);
export const ANALYTICS_RUN_A_COMPARE = readPage(anRunACompare);
export const ANALYTICS_RUN_A_COMPARE_REQUEST = readRunInput(anRunACompareRequest);
export const ANALYTICS_RUN_A_PAGE2 = readPage(anRunAPage2);
export const ANALYTICS_RUN_A_PAGE2_REQUEST = readRunInput(anRunAPage2Request);
export const ANALYTICS_RUN_A_SELLER_SIDE = readPage(anRunASellerSide);
export const ANALYTICS_RUN_A_SELLER_SIDE_REQUEST = readRunInput(anRunASellerSideRequest);
export const ANALYTICS_RUN_B = readPage(anRunB);
export const ANALYTICS_RUN_B_REQUEST = readRunInput(anRunBRequest);
export const ANALYTICS_RUN_C = readPage(anRunC);
export const ANALYTICS_RUN_C_REQUEST = readRunInput(anRunCRequest);
export const ANALYTICS_RUN_D_DAY = readPage(anRunDDay);
export const ANALYTICS_RUN_D_DAY_REQUEST = readRunInput(anRunDDayRequest);
export const ANALYTICS_RUN_D_MONTH = readPage(anRunDMonth);
export const ANALYTICS_RUN_D_MONTH_REQUEST = readRunInput(anRunDMonthRequest);
export const ANALYTICS_RUN_E = readPage(anRunE);
export const ANALYTICS_RUN_E_REQUEST = readRunInput(anRunERequest);
export const ANALYTICS_RUN_H = readPage(anRunH);
export const ANALYTICS_RUN_H_REQUEST = readRunInput(anRunHRequest);
export const ANALYTICS_RUN_A_COMPARE_NEEDS_DATES = readRefusal(anRunACompareNeedsDates);
export const ANALYTICS_RUN_A_COMPARE_NEEDS_DATES_REQUEST = readRunInput(
  anRunACompareNeedsDatesRequest,
);
export const ANALYTICS_RUN_A_GROUP_BY_REFUSED = readRefusal(anRunAGroupByRefused);
export const ANALYTICS_RUN_A_GROUP_BY_REFUSED_REQUEST = readRunInput(anRunAGroupByRefusedRequest);
export const ANALYTICS_RUN_A_UNKNOWN_ITEM = readRefusal(anRunAUnknownItem);
export const ANALYTICS_RUN_A_UNKNOWN_ITEM_REQUEST = readRunInput(anRunAUnknownItemRequest);
export const ANALYTICS_LINES_A_BANANA = readLines(anLinesABanana);
export const ANALYTICS_LINES_A_BANANA_REQUEST = readLinesInput(anLinesABananaRequest);
export const ANALYTICS_LINES_D_BANANA_OCTOBER = readLines(anLinesDBananaOctober);
export const ANALYTICS_LINES_D_BANANA_OCTOBER_REQUEST = readLinesInput(
  anLinesDBananaOctoberRequest,
);
export const ANALYTICS_LINES_E_RAHIM_BANANA = readLines(anLinesERahimBanana);
export const ANALYTICS_LINES_E_RAHIM_BANANA_REQUEST = readLinesInput(anLinesERahimBananaRequest);
export const ANALYTICS_PRINT_DATA_A_EN = readPrint(anPrintDataAEn);
export const ANALYTICS_PRINT_DATA_A_EN_REQUEST = readPrintInput(anPrintDataAEnRequest);
export const ANALYTICS_PRINT_DATA_D_UR = readPrint(anPrintDataDUr);
export const ANALYTICS_PRINT_DATA_D_UR_REQUEST = readPrintInput(anPrintDataDUrRequest);
export const ANALYTICS_CATALOG = readCatalog(anCatalog);
export const VIEWS_GET_EMPTY = readViews(anViewsGetEmpty);
export const VIEWS_GET_TWO = readViews(anViewsGetTwo);
export const VIEWS_SAVE_TWO = readViews(anViewsSaveTwo);
export const VIEWS_SAVE_TWO_REQUEST = readSaveInput(anViewsSaveTwoRequest);
export const VIEWS_SAVE_BAD = readRefusal(anViewsSaveBad);
export const VIEWS_SAVE_BAD_REQUEST = readSaveInput(anViewsSaveBadRequest);
