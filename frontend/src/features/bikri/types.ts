// Types of the بکری screen. What the backend owns comes from the generated bindings (Code
// Standards F2). What the user types is text (CLAUDE.md rule 1); the amounts the backend sends
// are `Paisa`, shown and never worked out.

import type { BikriDraft } from '../../bindings/BikriDraft';
import type { BikriPreview } from '../../bindings/BikriPreview';
import type { FieldError } from '../../bindings/FieldError';
import type { FreightPart } from '../../bindings/FreightPart';
import type { LineCalc } from '../../bindings/LineCalc';
import type { LinePreview } from '../../bindings/LinePreview';
import type { LotPreview } from '../../bindings/LotPreview';
import type { PaymentType } from '../../bindings/PaymentType';
import type { RateHint } from '../../bindings/RateHint';
import type { SellerMemory } from '../../bindings/SellerMemory';
import type { SummaryTotals } from '../../bindings/SummaryTotals';
import type { TruckMemory } from '../../bindings/TruckMemory';

export type {
  BikriDraft,
  BikriPreview,
  FieldError,
  FreightPart,
  LineCalc,
  LinePreview,
  LotPreview,
  PaymentType,
  RateHint,
  SellerMemory,
  SummaryTotals,
  TruckMemory,
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
  // A one-time seller (the پختہ کھاتہ account).
  pukhtaUr: 'pukhtaUr',
  pukhtaEn: 'pukhtaEn',
  pukhtaPhone: 'pukhtaPhone',
  // Set once the app has filled the cell itself, so it never fills it again (PRD FR-7.51).
  sellerFilled: 'sellerFilled',
  memoryFilledFor: 'memoryFilledFor',
} as const;

/** The fields of a sale line (the keys of `typed` and `picked` in the sale lines grid). */
export const SALE = {
  // The lot column of the whole-sale view: the lot number and its seller (words, not typed).
  lot: 'lot',
  buyer: 'buyer',
  qty: 'qty',
  rate: 'rate',
  amount: 'amount',
  sellerComm: 'sellerComm',
  buyerComm: 'buyerComm',
  laga: 'laga',
  buyerTotal: 'buyerTotal',
  payment: 'payment',
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
  pukhtaUr: 'pukhtaUr',
  pukhtaEn: 'pukhtaEn',
  pukhtaPhone: 'pukhtaPhone',
  // Set once the app has filled the quantity itself (PRD FR-7.52).
  qtyFilled: 'qtyFilled',
} as const;

/** The codes of the two one-time accounts (PRD FR-13.1). */
export const PUKHTA_SELLER_CODE = '030119999';
export const PUKHTA_BUYER_CODE = '03049999';

/** The typed value that means "on" in a tick of a row. */
export const ON = '1';
