// The real answers and requests of the backend for the garden slice of round 11 (analytics view I,
// the garden columns of F and G, the garden box of the other views). The backend wrote them in its own
// test (backend/crates/services/tests/fixtures/analytics_*.json); they are copied here, never edited,
// and read as the generated types. Money in them is paisa, as on the wire. The garden world is E2E-40:
// the garden Shahpur, الف 60% from the shop and ب 40% from his pocket, closed on 13-10-26 with a
// result of 11,400.

import type { AnalyticsLinesInput } from '../../src/bindings/AnalyticsLinesInput';
import type { AnalyticsLinesPage } from '../../src/bindings/AnalyticsLinesPage';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import type { AnalyticsPrintInput } from '../../src/bindings/AnalyticsPrintInput';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import anRunI from '../fixtures/analytics_run_i_garden.json';
import anRunIRequest from '../fixtures/analytics_run_i_garden_request.json';
import anRunISeller from '../fixtures/analytics_run_i_garden_seller_side.json';
import anRunISellerRequest from '../fixtures/analytics_run_i_garden_seller_side_request.json';
import anRunFGarden from '../fixtures/analytics_run_f_garden.json';
import anRunFGardenRequest from '../fixtures/analytics_run_f_garden_request.json';
import anRunGGarden from '../fixtures/analytics_run_g_garden_day.json';
import anRunGGardenRequest from '../fixtures/analytics_run_g_garden_day_request.json';
import anRunAGarden from '../fixtures/analytics_run_a_garden_goods.json';
import anRunAGardenRequest from '../fixtures/analytics_run_a_garden_goods_request.json';
import anLinesI from '../fixtures/analytics_lines_i_garden.json';
import anLinesIRequest from '../fixtures/analytics_lines_i_garden_request.json';
import anPrintI from '../fixtures/analytics_print_data_i_en.json';
import anPrintIRequest from '../fixtures/analytics_print_data_i_en_request.json';

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

export const ANALYTICS_RUN_I_GARDEN = readPage(anRunI);
export const ANALYTICS_RUN_I_GARDEN_REQUEST = readRunInput(anRunIRequest);
export const ANALYTICS_RUN_I_GARDEN_SELLER_SIDE = readPage(anRunISeller);
export const ANALYTICS_RUN_I_GARDEN_SELLER_SIDE_REQUEST = readRunInput(anRunISellerRequest);
export const ANALYTICS_RUN_F_GARDEN = readPage(anRunFGarden);
export const ANALYTICS_RUN_F_GARDEN_REQUEST = readRunInput(anRunFGardenRequest);
export const ANALYTICS_RUN_G_GARDEN_DAY = readPage(anRunGGarden);
export const ANALYTICS_RUN_G_GARDEN_DAY_REQUEST = readRunInput(anRunGGardenRequest);
export const ANALYTICS_RUN_A_GARDEN_GOODS = readPage(anRunAGarden);
export const ANALYTICS_RUN_A_GARDEN_GOODS_REQUEST = readRunInput(anRunAGardenRequest);
export const ANALYTICS_LINES_I_GARDEN = readLines(anLinesI);
export const ANALYTICS_LINES_I_GARDEN_REQUEST = readLinesInput(anLinesIRequest);
export const ANALYTICS_PRINT_DATA_I_EN = readPrint(anPrintI);
export const ANALYTICS_PRINT_DATA_I_EN_REQUEST = readPrintInput(anPrintIRequest);
