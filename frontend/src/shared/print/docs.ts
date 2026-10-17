// The documents the app prints, in the order the Printing settings list them, and the Fluent
// message that names each one. The Printing settings keep a list of these ("print after save",
// PRD FR-7.57), so the settings tab and the print boxes share this one list.
//
// The first three belong to a saved بکری (plan 14). The receipt slip belongs to a saved voucher
// (plan 15): the sale book never offers it, and a voucher never offers the other three.

import type { PrintDoc } from '../../bindings/PrintDoc';

/** The documents of a saved بکری: what the print box of the sale book lists. */
export const BIKRI_PRINT_DOCS: readonly PrintDoc[] = ['seller_bill', 'buyer_slips', 'pukhta_slip'];

/** Every document, for the Printing settings (paper of each, and what Ctrl+Enter prints). */
export const PRINT_DOCS: readonly PrintDoc[] = [...BIKRI_PRINT_DOCS, 'receipt_slip'];

/** The message that names a document in a list or a box (plural for the buyer slips). */
export const DOC_NAME_ID: Record<PrintDoc, string> = {
  seller_bill: 'print-doc-seller-bill',
  buyer_slips: 'print-doc-buyer-slips',
  pukhta_slip: 'print-doc-pukhta-slip',
  receipt_slip: 'print-doc-receipt-slip',
};
