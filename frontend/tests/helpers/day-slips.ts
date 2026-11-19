// Test helper: the buyer parchi, the seller bills, the whole sale print and ادھاری تفصیل (plan 76 point 3, plan 78 A1
// to A6, PRD FR-19.15) as the backend answers them (`reports_buyer_parchi_print_data`,
// `reports_seller_bills_print_data`, `reports_sale_print_data`, `reports_adhari_sheet_print_data`): the REAL answers of
// the worked day (Events 1 to 5, 10-10-26) in the print language, copied from the backend's own test and never
// edited. The sums are the backend's: the screen shows what comes back and adds nothing up (CLAUDE.md rule 1).

import { vi } from 'vitest';
import type { DaySlipsPrintData } from '../../src/bindings/DaySlipsPrintData';
import type { Language } from '../../src/bindings/Language';
import type { Paper } from '../../src/bindings/Paper';
import type { SalePrintData } from '../../src/bindings/SalePrintData';
import type { TablePrintData } from '../../src/bindings/TablePrintData';
import * as reportsApi from '../../src/features/reports/api';
import * as real from './plan78-real';
import { printOutcomeFor } from './print-job';

/** One of the two slip prints: the buyers' parchi, or the sellers' bills. */
export type SlipVariant = 'parchi' | 'seller-bills';

/**
 * The slips of the day in the print language, as the backend writes them for the worked day: a parchi has the buyers
 * only (one slip for each buyer: ارشد, اکرم, شاہد), the seller bills the sellers only (رحیم, کریم).
 */
export function daySlipsData(
  language: Language,
  variant: SlipVariant,
  paper: Paper = 'a4',
): DaySlipsPrintData {
  const parchi = language === 'ur' ? real.PARCHI_UR : real.PARCHI_EN;
  const bills = language === 'ur' ? real.SELLER_BILLS_UR : real.SELLER_BILLS_EN;
  const data = structuredClone(variant === 'parchi' ? parchi : bills);
  return {
    ...data,
    paper,
    sellers: data.sellers.map((slip) => ({ ...slip, paper })),
    buyers: data.buyers.map((slip) => ({ ...slip, paper })),
  };
}

/**
 * The whole sale print of 10-10-26 as the backend writes it (plan 78 A4): the sellers table with its 15 columns (date,
 * بکری, seller, lot, qty, gross, کرایہ, کمیشن, مزدوری, منشیانہ, نقد, کرایہ سٹور, فون, کل اخراجات, صافی بکری) and a totals
 * row, then the buyers table (buyer, bought, bill, نقد, ادھار) with its totals row 44,364 / 12,800 / 31,564.
 */
export function salePrintData(language: Language, paper: Paper = 'a4'): SalePrintData {
  const data = language === 'ur' ? real.SALE_PRINT_UR : real.SALE_PRINT_EN;
  return { ...structuredClone(data), paper };
}

/**
 * ادھاری تفصیل of 10-10-26 (plan 78 A5) as the backend writes it: one row for each of the day's buyers, then what
 * everyone owes; the totals read 44,364 / 12,800 / 31,564 / 0 / 31,564.
 */
export function adhariPrintData(language: Language, paper: Paper = 'a4'): TablePrintData {
  const data = structuredClone(language === 'ur' ? real.ADHARI_PRINT_UR : real.ADHARI_PRINT_EN);
  return { ...data, paper };
}

/** Makes the print-data commands and the print jobs of the slips and the whole sale answer. */
export function fakeDaySlips(): void {
  vi.mocked(reportsApi.reportsBuyerParchiPrintData).mockImplementation((input) =>
    Promise.resolve(daySlipsData(input.language, 'parchi', input.paper ?? 'a4')),
  );
  vi.mocked(reportsApi.reportsSellerBillsPrintData).mockImplementation((input) =>
    Promise.resolve(daySlipsData(input.language, 'seller-bills', input.paper ?? 'a4')),
  );
  vi.mocked(reportsApi.reportsSalePrintData).mockImplementation((input) =>
    Promise.resolve(salePrintData(input.language, input.paper ?? 'a4')),
  );
  vi.mocked(reportsApi.reportsAdhariSheetPrintData).mockImplementation((input) =>
    Promise.resolve(adhariPrintData(input.language, input.paper ?? 'a4')),
  );
  vi.mocked(reportsApi.reportsAdhariSheetPrint).mockImplementation((input) =>
    Promise.resolve(printOutcomeFor(input.to, `adhari-20261010-${input.sheet.language}.pdf`)),
  );
  vi.mocked(reportsApi.reportsBuyerParchiPrint).mockImplementation((input) =>
    Promise.resolve(printOutcomeFor(input.to, `parchi-20261010-${input.slips.language}.pdf`)),
  );
  vi.mocked(reportsApi.reportsSellerBillsPrint).mockImplementation((input) =>
    Promise.resolve(printOutcomeFor(input.to, `seller-bills-20261010-${input.slips.language}.pdf`)),
  );
  vi.mocked(reportsApi.reportsSalePrint).mockImplementation((input) =>
    Promise.resolve(printOutcomeFor(input.to, `sale-20261010-${input.slips.language}.pdf`)),
  );
}
