// Test helper: the buyer parchi and the overall sale print (plan 76 point 3, PRD FR-19.15) as the backend
// answers them (`reports_buyer_parchi_print_data`, `reports_sale_print_data`): the REAL R35 slips of رحیم
// (the seller) and اکرم (the buyer) of 10-10-26 after Events 1 to 5 (day-slips-real.ts), in the print
// language, with the header of the print. The sums are the backend's: the screen shows what comes back and
// adds nothing up (CLAUDE.md rule 1).

import { vi } from 'vitest';
import type { DaySlipsPrintData } from '../../src/bindings/DaySlipsPrintData';
import type { Language } from '../../src/bindings/Language';
import type { Paper } from '../../src/bindings/Paper';
import * as reportsApi from '../../src/features/reports/api';
import {
  PRINT_SLIP_BUYER_EN,
  PRINT_SLIP_BUYER_UR,
  PRINT_SLIP_SELLER_EN,
  PRINT_SLIP_SELLER_UR,
} from './day-slips-real';
import { printFrameFor } from './print-frame-real';
import { printOutcomeFor } from './print-job';

/** The slips of the day in the print language: one seller, one buyer. A parchi has the buyers only. */
export function daySlipsData(
  language: Language,
  variant: 'parchi' | 'sale',
  paper: Paper = 'a4',
): DaySlipsPrintData {
  const seller = language === 'ur' ? PRINT_SLIP_SELLER_UR : PRINT_SLIP_SELLER_EN;
  const buyer = language === 'ur' ? PRINT_SLIP_BUYER_UR : PRINT_SLIP_BUYER_EN;
  return {
    frame: printFrameFor({
      language,
      title_id: variant === 'parchi' ? 'print-parchi-title' : 'print-sale-title',
      facts: [],
    }),
    paper,
    date: '2026-10-10',
    sellers: variant === 'parchi' ? [] : [{ ...seller, paper }],
    buyers: [{ ...buyer, paper }],
  };
}

/** Makes the two print-data commands and the two print jobs answer. */
export function fakeDaySlips(): void {
  vi.mocked(reportsApi.reportsBuyerParchiPrintData).mockImplementation((input) =>
    Promise.resolve(daySlipsData(input.language, 'parchi', input.paper ?? 'a4')),
  );
  vi.mocked(reportsApi.reportsSalePrintData).mockImplementation((input) =>
    Promise.resolve(daySlipsData(input.language, 'sale', input.paper ?? 'a4')),
  );
  vi.mocked(reportsApi.reportsBuyerParchiPrint).mockImplementation((input) =>
    Promise.resolve(printOutcomeFor(input.to, `parchi-20261010-${input.slips.language}.pdf`)),
  );
  vi.mocked(reportsApi.reportsSalePrint).mockImplementation((input) =>
    Promise.resolve(printOutcomeFor(input.to, `sale-20261010-${input.slips.language}.pdf`)),
  );
}
