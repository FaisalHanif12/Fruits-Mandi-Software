// The real answers of the backend for sale lines that carry a quality (درجہ, plan 58, PRD FR-12.42).
// The backend wrote them in its own test (backend/crates/services/tests/fixtures/sale_grade_*.json):
// one lot of Apple with three lines, First at 1,200, Second at 800 and Third at 500, sold to Akram and
// Arshad. They are copied here, never edited, and read as the generated types. Money in them is paisa.

import type { PrintData } from '../../src/bindings/PrintData';
import type { ReportPage } from '../../src/bindings/ReportPage';
import bikriGet from '../fixtures/sale_grade_bikri_get.json';
import buyerSlipsUr from '../fixtures/sale_grade_print_buyer_slips_ur.json';
import sellerBillEn from '../fixtures/sale_grade_print_seller_bill_en.json';
import r2 from '../fixtures/sale_grade_report_r2.json';
import r35Akram from '../fixtures/sale_grade_report_r35_akram.json';
import r5 from '../fixtures/sale_grade_report_r5.json';

function readPrint(value: unknown): PrintData {
  return value as PrintData;
}
function readPage(value: unknown): ReportPage {
  return value as ReportPage;
}

/** The seller bill in English: three lines whose `item` is the item alone and whose `grade` is set. */
export const GRADE_SELLER_BILL_EN = readPrint(sellerBillEn);
/** The buyer slips (R5 "Print All") in Urdu: the backend writes the quality into each `detail`. */
export const GRADE_BUYER_SLIPS_UR = readPrint(buyerSlipsUr);
/** R2, the full sales register: a `grade` column right after the item. */
export const GRADE_R2 = readPage(r2);
/** R35, Akram's day slip: a `grade` column as the last one. */
export const GRADE_R35_AKRAM = readPage(r35Akram);
/** R5, the buyer bills: the quality is on each sale part. */
export const GRADE_R5 = readPage(r5);
/** The saved بکری with the three lines (`bikri_get`). */
export const GRADE_BIKRI_GET: unknown = bikriGet;
