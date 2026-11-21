// Types of the بکری screen. What the backend owns comes from the generated bindings (Code
// Standards F2). What the user types is text (CLAUDE.md rule 1); the amounts the backend sends
// are `Paisa`, shown and never worked out.

import type { BikriDraft as BackendDraft } from '../../bindings/BikriDraft';
import type { CarriedLotInput } from '../../bindings/CarriedLotInput';
import type { FieldError } from '../../bindings/FieldError';
import type { LineCalc } from '../../bindings/LineCalc';
import type { LinePreview } from '../../bindings/LinePreview';
import type { LotPreview } from '../../bindings/LotPreview';
import type { RateHint } from '../../bindings/RateHint';
import type { SellerMemory } from '../../bindings/SellerMemory';
import type { SummaryTotals } from '../../bindings/SummaryTotals';
import type { BikriPreviewWithSellers } from './seller-paid-types';

/**
 * The draft of the screen as the backend takes it. When lots carried over from earlier بکری are on the
 * screen it also holds them: they go with the mixed commands, never with `bikri_preview` or
 * `bikri_save` (plan 76 point 1). A screen with none has no `carried` at all, so what it sends is
 * exactly what it sent before.
 */
export type BikriDraft = BackendDraft & { carried?: CarriedLotInput[] };

/** The preview of the screen; from plan 91 it also says, for each seller, how his account stands after the sale. */
export type BikriPreview = BikriPreviewWithSellers;

export type {
  FieldError,
  LineCalc,
  LinePreview,
  LotPreview,
  RateHint,
  SellerMemory,
  SummaryTotals,
};

/**
 * The fields of a lot row (the keys of `typed` and `picked` in the lots grid). The first eight are
 * the cells of the grid; the rest are in the "more" panel of the lot.
 */
export const LOT = {
  seller: 'seller',
  item: 'item',
  packing: 'packing',
  qty: 'qty',
  freight: 'freight',
  labour: 'labour',
  munshiana: 'munshiana',
  cash: 'cash',
  // Calculated cells (normal and wide windows).
  net: 'net',
  balance: 'balance',
  // The "more" panel.
  marka: 'marka',
  store: 'store',
  weight: 'weight',
  phone: 'phone',
  storeRent: 'storeRent',
  freightPaidBy: 'freightPaidBy',
  adjustAdvance: 'adjustAdvance',
  labourMethod: 'labourMethod',
  munshianaMethod: 'munshianaMethod',
  phoneMethod: 'phoneMethod',
  // The purchase of an own goods lot (the seller is اپنا مال, PRD FR-12.26): its id, as text.
  purchase: 'purchase',
  // The goods arrived entry of a garden's lot (the seller is the garden, PRD FR-12.40): its id.
  arrival: 'arrival',
  // Set once the app has filled the cell itself, so it never fills it again (PRD FR-7.51).
  sellerFilled: 'sellerFilled',
  memoryFilledFor: 'memoryFilledFor',
  // The saved lot this row is, when a posting is open for edit (plan 16). An id, never shown.
  lotId: 'lotId',
  // Set on a lot carried over from an earlier بکری (باقی مال, PRD FR-12.17, FR-12.48): the lot is saved, its
  // cells are shown and not typed, and only its new sale lines are the day's work. Its `lotId` is the
  // lot, and `carriedBikri` the number of the بکری it came from (for the words on the screen only).
  carried: 'carried',
  carriedBikri: 'carriedBikri',
} as const;

/** The fields of a sale line (the keys of `typed` and `picked` in the sale lines grid). */
export const SALE = {
  // The lot column of the whole-sale view: the lot number and its seller (words, not typed).
  lot: 'lot',
  buyer: 'buyer',
  qty: 'qty',
  rate: 'rate',
  // The quality of the goods of the line (درجہ): first, second, third, or nothing (PRD FR-12.42).
  grade: 'grade',
  amount: 'amount',
  sellerComm: 'sellerComm',
  buyerComm: 'buyerComm',
  laga: 'laga',
  buyerTotal: 'buyerTotal',
  // The last money cell of the line: what the buyer paid on the spot (موقع پر جمع, PRD FR-12.44).
  // Typed as text and sent as typed; for a buyer who must pay in full the backend sets it.
  paid: 'paid',
  // Set only in the preview's cells (never typed): the amount the backend decided for a buyer who
  // must pay in full. While it is there the paid cell shows it and cannot be typed in.
  paidFixed: 'paidFixed',
  // What is left on the buyer's account (کھاتے میں باقی): a figure of the preview, never typed.
  leftOnAccount: 'leftOnAccount',
  // The details of the line.
  weight: 'weight',
  differs: 'differs',
  buyerRate: 'buyerRate',
  sellerCommValue: 'sellerCommValue',
  sellerCommMethod: 'sellerCommMethod',
  buyerCommValue: 'buyerCommValue',
  buyerCommMethod: 'buyerCommMethod',
  lagaValue: 'lagaValue',
  lagaMethod: 'lagaMethod',
  note: 'note',
  // Set once the app has filled the quantity itself (PRD FR-7.52).
  qtyFilled: 'qtyFilled',
  // The saved line this row is, when a posting is open for edit (plan 16). An id, never shown.
  saleId: 'saleId',
} as const;

/** The اپنا مال account: the seller of a lot of the firm's own goods (PRD FR-10.4, FR-12.26). */
export { OWN_GOODS_CODE } from '../../shared/accounts';

/** The typed value that means "on" in a tick of a row. */
export const ON = '1';

/**
 * The reason the backend stores when a save is undone at once (PRD FR-7.47). It is the Fluent ID of
 * the words, so a cancelled بکری shows them in the app language (plan 16 contract 9).
 */
export const UNDO_REASON_ID = 'bikri-undo-reason';
