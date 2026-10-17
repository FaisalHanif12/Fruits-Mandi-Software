// The types of a printed document, from the generated bindings (Code Standards F2), and the
// paper classes. `PrintData` is ONE shape for every document of the app (plan 14 contract 5), so
// the receipt slip of plan 15 and the reports of D5 reuse this layout.

import type { AmountRole } from '../../bindings/AmountRole';
import type { Paper } from '../../bindings/Paper';
import type { PrintAmount } from '../../bindings/PrintAmount';
import type { PrintData } from '../../bindings/PrintData';
import type { PrintFact } from '../../bindings/PrintFact';
import type { PrintLine } from '../../bindings/PrintLine';
import type { PrintPage } from '../../bindings/PrintPage';
import type { PrintParty } from '../../bindings/PrintParty';
import type { PrintWords } from '../../bindings/PrintWords';

export type { AmountRole, Paper, PrintAmount, PrintData, PrintFact, PrintLine, PrintPage };
export type { PrintParty, PrintWords };

/** The columns a goods table can have. The template picks them for each page. */
export type PrintColumn =
  'date' | 'buyer' | 'item' | 'packing' | 'qty' | 'weight' | 'rate' | 'amount';

/** The Fluent message of each column head. The heads are fixed by the layout (contract 6). */
export const COLUMN_HEAD_ID: Record<PrintColumn, string> = {
  date: 'print-col-date',
  buyer: 'print-col-buyer',
  item: 'print-col-item',
  packing: 'print-col-packing',
  qty: 'print-col-qty',
  weight: 'print-col-weight',
  rate: 'print-col-rate',
  amount: 'print-col-amount',
};

/** The paper classes, in the order the Settings tab lists them. */
export const PAPERS: readonly Paper[] = ['a4', 'a5', 'mm80'];

/** The messages of the page footer: the print time and the page words. */
export const FOOTER_LABEL_IDS = {
  printed: 'print-printed-at',
  page: 'print-page-label',
  of: 'print-page-of',
} as const;

/** Fluent ids of the layout itself (not sent by the backend). */
export const LAYOUT_IDS = {
  partyCode: 'print-party-code',
  partyPhone: 'print-party-phone',
  cancelled: 'print-cancelled-stamp',
  empty: 'print-preview-empty',
} as const;

/** The payment message that means the buyer has paid (contract 6). */
export const PAYMENT_PAID_ID = 'print-payment-cash';

/**
 * The messages that say how a voucher was paid (`print-mode-cash`, `print-mode-bank`,
 * `print-mode-cheque`, plan 15 contract 9). On a slip the money has moved, so the box is drawn as
 * paid (solid), not as a payment still due (dashed).
 */
export const MODE_ID_PREFIX = 'print-mode-';
