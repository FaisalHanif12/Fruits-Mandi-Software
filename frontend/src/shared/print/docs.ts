// The documents a saved بکری can print, in the order the print box and the Printing settings list
// them, and the Fluent message that names each one. The Printing settings keep a list of these
// ("print after save", PRD FR-7.57), so the settings tab and the print box share this one list.

import type { PrintDoc } from '../../bindings/PrintDoc';

export const PRINT_DOCS: readonly PrintDoc[] = ['seller_bill', 'buyer_slips', 'pukhta_slip'];

/** The message that names a document in a list or a box (plural for the buyer slips). */
export const DOC_NAME_ID: Record<PrintDoc, string> = {
  seller_bill: 'print-doc-seller-bill',
  buyer_slips: 'print-doc-buyer-slips',
  pukhta_slip: 'print-doc-pukhta-slip',
};
