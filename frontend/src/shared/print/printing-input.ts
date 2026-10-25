// The Printing settings as the backend takes them (PRD Section 21), made from the saved settings
// with one thing changed: the documents that print after a save. Every screen that remembers a tick
// (the print box of the sale book, the receipt slip of a voucher) saves through this one function,
// so a paper that was added later (the collection list) is never dropped by accident.

import type { PrintDoc } from '../../bindings/PrintDoc';
import type { PrintingSettings } from '../../bindings/PrintingSettings';
import type { SetPrintingInput } from '../../bindings/SetPrintingInput';

export function printingInputOf(
  saved: PrintingSettings,
  printAfterSave: readonly PrintDoc[],
): SetPrintingInput {
  return {
    paper_seller_bill: saved.paper_seller_bill,
    paper_buyer_slips: saved.paper_buyer_slips,
    paper_pukhta_slip: saved.paper_pukhta_slip,
    paper_receipt_slip: saved.paper_receipt_slip,
    paper_collection_list: saved.paper_collection_list,
    printer: saved.printer,
    copies: String(saved.copies),
    urdu_digits: saved.urdu_digits,
    print_after_save: [...printAfterSave],
  };
}
