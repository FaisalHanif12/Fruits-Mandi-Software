// The real answers and requests of the backend for the fruit slice of the report engine: R37, the
// fruit register (all lines, and the shop's guests only), the fruit column of the stock R10, the fruit
// row of the day summary R34, and the print of R37. The backend wrote them in its own test
// (backend/crates/services/tests/fixtures); they are copied here by `scripts/copy-backend-fixtures.mjs`,
// never edited, and read as the generated types. Money in them is paisa, as on the wire. The fruit
// world is E2E-42 on 10-10-2026: الف took 2 پیٹی کیلا at 580 from رحیم's lot (1,160), and 3 پیٹی سیب at
// 1,000 from another shop's agent (حاجی نذیر) were served to the shop's guests (3,000).

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportPrintInput } from '../../src/bindings/ReportPrintInput';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import printR37 from '../fixtures/reports_print_data_r37_en.json';
import printR37Request from '../fixtures/reports_print_data_r37_en_request.json';
import r10Fruit from '../fixtures/reports_run_r10_fruit.json';
import r10FruitRequest from '../fixtures/reports_run_r10_fruit_request.json';
import r34Fruit from '../fixtures/reports_run_r34_fruit.json';
import r34FruitRequest from '../fixtures/reports_run_r34_fruit_request.json';
import r37All from '../fixtures/reports_run_r37_e2e_42.json';
import r37AllRequest from '../fixtures/reports_run_r37_e2e_42_request.json';
import r37Guests from '../fixtures/reports_run_r37_guests.json';
import r37GuestsRequest from '../fixtures/reports_run_r37_guests_request.json';

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

/** R37 on 10-10-2026: الف's کیلا and the guests' سیب, 5 نگ, 4,160. */
export const R37_FRUIT = readPage(r37All);
export const R37_FRUIT_REQUEST = readRunInput(r37AllRequest);
/** R37 for the shop's guests only: the سیب line (`guests_only` true). */
export const R37_GUESTS = readPage(r37Guests);
export const R37_GUESTS_REQUEST = readRunInput(r37GuestsRequest);
export const PRINT_R37_EN = readPrint(printR37);
export const PRINT_R37_EN_REQUEST = readPrintInput(printR37Request);
/** The stock R10 with the fruit column: رحیم's lot, 50 received, 45 sold, 2 taken, 3 left. */
export const R10_FRUIT = readPage(r10Fruit);
export const R10_FRUIT_REQUEST = readRunInput(r10FruitRequest);
/** The day summary R34 with its fruit row (the count is نگ, the amount the value). */
export const R34_FRUIT = readPage(r34Fruit);
export const R34_FRUIT_REQUEST = readRunInput(r34FruitRequest);
