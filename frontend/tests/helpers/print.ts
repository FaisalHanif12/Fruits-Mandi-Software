// Test helper: the print commands answered like a healthy backend (plan 14 contract). The wrappers
// (`features/reports/api` and `features/settings/api`) must be mocked with vi.mock in the test
// file; this sets their answers and keeps what the screen sent. The print data is the canned PRD
// worked day (print-data.ts), in the print language the screen asked for.

import { vi } from 'vitest';
import type { BikriPrintDataInput } from '../../src/bindings/BikriPrintDataInput';
import type { PrintData } from '../../src/bindings/PrintData';
import type { PrintInput } from '../../src/bindings/PrintInput';
import type { PrintOutcome } from '../../src/bindings/PrintOutcome';
import type { PrintingSettings } from '../../src/bindings/PrintingSettings';
import type { SetPrintingInput } from '../../src/bindings/SetPrintingInput';
import * as reportsApi from '../../src/features/reports/api';
import * as settingsApi from '../../src/features/settings/api';
import { printDataFor } from './print-data';
import { printingSettings } from './print-settings';

export { printingSettings };

export interface FakePrinting {
  /** The settings as the backend holds them now. */
  settings: () => PrintingSettings;
  /** Every `settings_set_printing` the screen sent, in order. */
  saves: SetPrintingInput[];
  /** Every `reports_print` the screen sent, in order. */
  prints: PrintInput[];
  /** Every `reports_bikri_print_data` the screen asked for, in order. */
  dataRequests: BikriPrintDataInput[];
}

interface FakePrintingOptions {
  settings?: Partial<PrintingSettings>;
  printers?: string[];
  /** Decides the answer of `reports_print`. Default: everything asked for was printed. */
  print?: (input: PrintInput) => PrintOutcome | Promise<PrintOutcome>;
  /** Decides the print data. Default: the canned data of the PRD worked day. */
  data?: (input: BikriPrintDataInput) => PrintData;
}

export function fakePrinting(options: FakePrintingOptions = {}): FakePrinting {
  let current = printingSettings(options.settings);
  const fake: FakePrinting = {
    settings: () => current,
    saves: [],
    prints: [],
    dataRequests: [],
  };
  vi.mocked(settingsApi.settingsGetPrinting).mockImplementation(() => Promise.resolve(current));
  vi.mocked(settingsApi.settingsSetPrinting).mockImplementation((input) => {
    fake.saves.push(input);
    current = {
      paper_seller_bill: input.paper_seller_bill,
      paper_buyer_slips: input.paper_buyer_slips,
      paper_pukhta_slip: input.paper_pukhta_slip,
      paper_receipt_slip: input.paper_receipt_slip,
      printer: input.printer,
      copies: Number(input.copies),
      urdu_digits: input.urdu_digits,
      print_after_save: input.print_after_save,
    };
    return Promise.resolve(current);
  });
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue(options.printers ?? []);
  vi.mocked(reportsApi.reportsBikriPrintData).mockImplementation((input) => {
    fake.dataRequests.push(input);
    if (options.data !== undefined) return Promise.resolve(options.data(input));
    return Promise.resolve(
      printDataFor(input.doc, { language: input.language, paper: current[paperKey(input.doc)] }),
    );
  });
  vi.mocked(reportsApi.reportsPrint).mockImplementation((input) => {
    fake.prints.push(input);
    if (options.print !== undefined) return Promise.resolve(options.print(input));
    return Promise.resolve({
      printed: input.docs,
      pdf_paths:
        input.to === 'pdf'
          ? input.docs.map((doc) => `prints/${doc}-1-${input.language}.pdf`)
          : null,
    });
  });
  return fake;
}

function paperKey(doc: BikriPrintDataInput['doc']) {
  return doc === 'seller_bill'
    ? 'paper_seller_bill'
    : doc === 'buyer_slips'
      ? 'paper_buyer_slips'
      : doc === 'pukhta_slip'
        ? 'paper_pukhta_slip'
        : 'paper_receipt_slip';
}

/** A promise that fails with `value` as it is, the way Tauri's invoke fails (with the raw error). */
export function failWith(value: unknown): Promise<never> {
  return Promise.resolve().then(() => {
    throw value;
  });
}
