// The backend's REAL answers for the پیشگی typed on a lot (plan 95 item 11, PRD FR-12.52): رحیم, previous 0, the lot's
// credit 62,160 with a second lot of 52,900, نقد ادائیگی 30,000 and پیشگی 20,000 with the advance tick off: the seller's
// day slip (R35), the seller bills made from slips, the seller bill of the sale book (R1) and the whole sale print,
// in Urdu and in English. The real engine wrote them in its own tests; they are copied here by
// `scripts/copy-backend-fixtures.mjs lot_advance_`, never edited. A reader takes `unknown`, because the money in a
// binding is a branded type that a plain number is not.

import type { DaySlipsPrintData } from '../../src/bindings/DaySlipsPrintData';
import type { PrintData } from '../../src/bindings/PrintData';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { SalePrintData } from '../../src/bindings/SalePrintData';
import billR1En from '../fixtures/lot_advance_seller_bill_r1_en.json';
import billR1Ur from '../fixtures/lot_advance_seller_bill_r1_ur.json';
import billsEn from '../fixtures/lot_advance_seller_bills_en.json';
import billsUr from '../fixtures/lot_advance_seller_bills_ur.json';
import saleEn from '../fixtures/lot_advance_sale_print_en.json';
import saleUr from '../fixtures/lot_advance_sale_print_ur.json';
import slipEn from '../fixtures/lot_advance_slip_r35_en.json';
import slipUr from '../fixtures/lot_advance_slip_r35_ur.json';

function readReport(value: unknown): ReportPrintData {
  const answer = value as Partial<ReportPrintData> &
    Omit<ReportPrintData, 'party' | 'named_title_id'>;
  return { ...answer, party: answer.party ?? null, named_title_id: answer.named_title_id ?? null };
}

export const LOT_ADVANCE_SLIP_EN = readReport(slipEn);
export const LOT_ADVANCE_SLIP_UR = readReport(slipUr);
export const LOT_ADVANCE_BILLS_EN = billsEn as unknown as DaySlipsPrintData;
export const LOT_ADVANCE_BILLS_UR = billsUr as unknown as DaySlipsPrintData;
export const LOT_ADVANCE_R1_EN = billR1En as unknown as PrintData;
export const LOT_ADVANCE_R1_UR = billR1Ur as unknown as PrintData;
export const LOT_ADVANCE_SALE_EN = saleEn as unknown as SalePrintData;
export const LOT_ADVANCE_SALE_UR = saleUr as unknown as SalePrintData;
