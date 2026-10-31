// The backend's REAL answers for plan 42 (PRD v1.12, FR-19.12, R34): a buyer's day slip (اکرم), a
// seller's day slip (رحیم) and the day total sheet of 10-10-2026 after Events 1 to 5, as the screen page
// (`reports_run`) and as the print data in Urdu and English (`reports_print_data`). The real engine wrote
// them in its own test (`backend/crates/services/tests/fixtures`); they are copied here by
// `scripts/copy-backend-fixtures.mjs report_r3`, never edited. Money in them is paisa, as on the wire.
// A reader takes `unknown`, because the money in a binding is a branded type that a plain number is not.

import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import dayPage from '../fixtures/report_r34_day_sheet.json';
import dayPrintEn from '../fixtures/report_r34_day_sheet_print_en.json';
import dayPrintUr from '../fixtures/report_r34_day_sheet_print_ur.json';
import buyerPage from '../fixtures/report_r35_slip_buyer_akram.json';
import buyerPrintEn from '../fixtures/report_r35_slip_buyer_akram_print_en.json';
import buyerPrintUr from '../fixtures/report_r35_slip_buyer_akram_print_ur.json';
import sellerPage from '../fixtures/report_r35_slip_seller_rahim.json';
import weighedPage from '../fixtures/report_r35_slip_buyer_weighed.json';
import weighedPrintEn from '../fixtures/report_r35_slip_buyer_weighed_print_en.json';
import weighedPrintUr from '../fixtures/report_r35_slip_buyer_weighed_print_ur.json';
import sellerPrintEn from '../fixtures/report_r35_slip_seller_rahim_print_en.json';
import sellerPrintUr from '../fixtures/report_r35_slip_seller_rahim_print_ur.json';

function readPage(value: unknown): ReportPage {
  return value as ReportPage;
}
function readPrint(value: unknown): ReportPrintData {
  return value as ReportPrintData;
}

/** The buyer's slip of اکرم: one line, 20 نگ, 12,880 with commission and لاگا; 10,000 paid; 2,880 left. */
export const SLIP_BUYER_AKRAM = readPage(buyerPage);
/** The seller's slip of رحیم: two lines of one item with its total row, the deductions, 24,316 net. */
export const SLIP_SELLER_RAHIM = readPage(sellerPage);
/**
 * A buyer's slip with a line of an item sold by weight (PRD A73): 2 نگ, 12.345 kg at 100 a kilo, the
 * stored amount 1,235, and nothing paid yet. The goods table has a sixth column, the weight.
 */
export const SLIP_BUYER_WEIGHED = readPage(weighedPage);
/** The day total sheet R34 of 10-10-2026, with the buyers' and the sellers' table. */
export const DAY_SHEET = readPage(dayPage);

export const PRINT_SLIP_BUYER_UR = readPrint(buyerPrintUr);
export const PRINT_SLIP_BUYER_EN = readPrint(buyerPrintEn);
export const PRINT_SLIP_SELLER_UR = readPrint(sellerPrintUr);
export const PRINT_SLIP_SELLER_EN = readPrint(sellerPrintEn);
export const PRINT_DAY_SHEET_UR = readPrint(dayPrintUr);
export const PRINT_SLIP_WEIGHED_UR = readPrint(weighedPrintUr);
export const PRINT_SLIP_WEIGHED_EN = readPrint(weighedPrintEn);
export const PRINT_DAY_SHEET_EN = readPrint(dayPrintEn);
