// The documents the app prints, in the order the Printing settings list them, and the Fluent
// message that names each one. The Printing settings keep a list of these ("print after save",
// PRD FR-7.57), so the settings tab and the print boxes share this one list.
//
// The first three belong to a saved بکری (plan 14). The receipt slip belongs to a saved voucher
// (plan 15): the sale book never offers it, and a voucher never offers the other three. The
// collection list (اگراہی فہرست, plan 17e, PRD FR-15.6) belongs to the collection screen: it has a
// paper of its own in the settings, and it is never printed after a save.

import type { PrintDoc } from '../../bindings/PrintDoc';

/** The documents of a saved بکری: what the print box of the sale book lists. */
export const BIKRI_PRINT_DOCS: readonly PrintDoc[] = ['seller_bill', 'buyer_slips', 'pukhta_slip'];

/** The documents a save can print (the Printing settings tick them, PRD FR-7.57). */
export const AFTER_SAVE_DOCS: readonly PrintDoc[] = [...BIKRI_PRINT_DOCS, 'receipt_slip'];

/** Every document, for the Printing settings: the paper of each. */
export const PRINT_DOCS: readonly PrintDoc[] = [...AFTER_SAVE_DOCS, 'collection_list'];

/** The message that names a document in a list or a box (plural for the buyer slips). */
export const DOC_NAME_ID: Record<PrintDoc, string> = {
  seller_bill: 'print-doc-seller-bill',
  buyer_slips: 'print-doc-buyer-slips',
  pukhta_slip: 'print-doc-pukhta-slip',
  receipt_slip: 'print-doc-receipt-slip',
  collection_list: 'print-doc-collection-list',
};
