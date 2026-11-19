// The real answers and requests of the backend for the fruit slice of the analytics (view K, the fruit
// column of view F and the lines behind its numbers). The backend wrote them in its own test
// (backend/crates/services/tests/fixtures/analytics_*.json); they are copied here, never edited, and
// read as the generated types. Money in them is paisa, as on the wire. The fruit world is E2E-42: الف
// took 2 پیٹی کیلا at 580 (1,160) from رحیم's lot, and 3 پیٹی سیب at 1,000 (3,000) from another
// shop's agent were served to the shop's guests, shared equally between the three partners.

import type { AnalyticsLinesInput } from '../../src/bindings/AnalyticsLinesInput';
import type { AnalyticsLinesPage } from '../../src/bindings/AnalyticsLinesPage';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import type { AnalyticsPrintInput } from '../../src/bindings/AnalyticsPrintInput';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import anRunK from '../fixtures/analytics_run_k_e2e_42.json';
import anRunKRequest from '../fixtures/analytics_run_k_e2e_42_request.json';
import anRunFFruit from '../fixtures/analytics_run_f_fruit.json';
import anRunFFruitRequest from '../fixtures/analytics_run_f_fruit_request.json';
import anLinesK from '../fixtures/analytics_lines_k_guests.json';
import anLinesKRequest from '../fixtures/analytics_lines_k_guests_request.json';
import anLinesFFruit from '../fixtures/analytics_lines_f_fruit_alif.json';
import anLinesFFruitRequest from '../fixtures/analytics_lines_f_fruit_alif_request.json';
import anPrintK from '../fixtures/analytics_print_data_k_en.json';
import anPrintKRequest from '../fixtures/analytics_print_data_k_en_request.json';

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

/** View K on 10-10-2026: الف's کیلا (2 نگ, 1,160, our shop) and the guests' سیب (3 نگ, 3,000, another shop). */
export const ANALYTICS_RUN_K_FRUIT = readPage(anRunK);
export const ANALYTICS_RUN_K_FRUIT_REQUEST = readRunInput(anRunKRequest);
/** View F with the fruit column filled: his home fruit and his equal part of the guests' fruit. */
export const ANALYTICS_RUN_F_FRUIT = readPage(anRunFFruit);
export const ANALYTICS_RUN_F_FRUIT_REQUEST = readRunInput(anRunFFruitRequest);
/** The lines behind the guests' سیب row of view K: the same columns as the fruit register R37. */
export const ANALYTICS_LINES_K_GUESTS = readLines(anLinesK);
export const ANALYTICS_LINES_K_GUESTS_REQUEST = readLinesInput(anLinesKRequest);
/** The lines behind الف's fruit number in view F: date, document, partner and amount. */
export const ANALYTICS_LINES_F_FRUIT_ALIF = readLines(anLinesFFruit);
export const ANALYTICS_LINES_F_FRUIT_ALIF_REQUEST = readLinesInput(anLinesFFruitRequest);
export const ANALYTICS_PRINT_DATA_K_EN = readPrint(anPrintK);
export const ANALYTICS_PRINT_DATA_K_EN_REQUEST = readPrintInput(anPrintKRequest);
