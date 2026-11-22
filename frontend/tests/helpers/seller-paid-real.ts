// The backend's REAL answers for plan 91 (a seller's money paid on the spot, PRD FR-12.23 v1.53): the four worked
// previews, the refused save, and the seller's day slip (R35) and seller bill
// (print menu) in Urdu and in English. The real engine wrote them in its own tests; they are copied here by
// `scripts/copy-backend-fixtures.mjs seller_paid_`, never edited. Money in them is paisa, as on the wire. A reader takes
// `unknown`, because the money in a binding is a branded type that a plain number is not.

import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import previewA from '../fixtures/seller_paid_preview_case_a.json';
import previewB from '../fixtures/seller_paid_preview_case_b.json';
import previewC from '../fixtures/seller_paid_preview_case_c.json';
import previewD from '../fixtures/seller_paid_preview_case_d.json';
import advancePreview from '../fixtures/lot_advance_preview_tick_off.json';
import refusedD from '../fixtures/seller_paid_save_refused_case_d.json';
import billAEn from '../fixtures/seller_paid_seller_bill_case_a_en.json';
import billAUr from '../fixtures/seller_paid_seller_bill_case_a_ur.json';
import billTwoEn from '../fixtures/seller_paid_seller_bill_two_lots_en.json';
import billTwoUr from '../fixtures/seller_paid_seller_bill_two_lots_ur.json';
import slipAEn from '../fixtures/seller_paid_slip_r35_case_a_en.json';
import slipAUr from '../fixtures/seller_paid_slip_r35_case_a_ur.json';
import slipTwoEn from '../fixtures/seller_paid_slip_r35_two_lots_en.json';
import slipTwoUr from '../fixtures/seller_paid_slip_r35_two_lots_ur.json';

function readPreview(value: unknown): BikriPreview {
  return value as BikriPreview;
}
function readPrint(value: unknown): ReportPrintData {
  const answer = value as Partial<ReportPrintData> &
    Omit<ReportPrintData, 'party' | 'named_title_id'>;
  return { ...answer, party: answer.party ?? null, named_title_id: answer.named_title_id ?? null };
}

/** Previous 30,000 بنام, credit 20,000, paid 5,000: carry 15,000, new 15,000 بنام. */
export const SELLER_PREVIEW_A = readPreview(previewA);
/** The second worked case of the plan (see the backend's test). */
export const SELLER_PREVIEW_B = readPreview(previewB);
/** The third worked case of the plan. */
export const SELLER_PREVIEW_C = readPreview(previewC);
/** Paid 25,000 when previous 30,000 بنام and credit 20,000: needs the Owner's yes, new 35,000 بنام. */
export const SELLER_PREVIEW_D = readPreview(previewD);
/** Previous 9,260 جمع, credit 52,900, نقد 30,000, پیشگی 20,000, tick off: rest 22,900, new 12,160 جمع (PRD FR-12.52). */
export const SELLER_PREVIEW_ADVANCE = readPreview(advancePreview);
/** What the save answers while the yes is missing: one ERR_SELLER_PAID_ABOVE_TODAY at `sellers[0].paid`. */
export const SELLER_SAVE_REFUSED_D = refusedD as unknown as { code: string; field: string }[];

export const SELLER_SLIP_A_EN = readPrint(slipAEn);
export const SELLER_SLIP_A_UR = readPrint(slipAUr);
export const SELLER_SLIP_TWO_EN = readPrint(slipTwoEn);
export const SELLER_SLIP_TWO_UR = readPrint(slipTwoUr);
export const SELLER_BILL_A_EN = readPrint(billAEn);
export const SELLER_BILL_A_UR = readPrint(billAUr);
export const SELLER_BILL_TWO_EN = readPrint(billTwoEn);
export const SELLER_BILL_TWO_UR = readPrint(billTwoUr);
