// The Printing settings before the Owner changes anything (plan 14 contract 3, PRD A21). Kept apart
// from print.ts, which uses Vitest, so the development page can use it in a plain browser tab.

import type { PrintingSettings } from '../../src/bindings/PrintingSettings';

export function printingSettings(overrides: Partial<PrintingSettings> = {}): PrintingSettings {
  return {
    paper_seller_bill: 'a4',
    paper_buyer_slips: 'a5',
    paper_pukhta_slip: 'a5',
    paper_receipt_slip: 'a5',
    printer: '',
    copies: 1,
    urdu_digits: false,
    print_after_save: ['seller_bill', 'buyer_slips', 'pukhta_slip'],
    ...overrides,
  };
}
