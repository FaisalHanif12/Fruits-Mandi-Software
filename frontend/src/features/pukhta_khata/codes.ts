// The codes of the two one-time accounts (PRD FR-13.1, FR-10.4): پختہ کھاتہ (بیوپاری) and
// پختہ کھاتہ (خریدار). They are fixed system accounts, so a screen that must know "is this a
// one-time account?" (the sale book, the vouchers) asks the code, in this one place.

// Their codes are type number 003 and the running numbers 0 and 1 (plan 32, R3).
export const PUKHTA_SELLER_CODE = '0031';
export const PUKHTA_BUYER_CODE = '0030';

/** True for either one-time account. Money kept under it is kept per person (PRD FR-13.5). */
export function isPukhtaCode(code: string | undefined | null): boolean {
  return code === PUKHTA_SELLER_CODE || code === PUKHTA_BUYER_CODE;
}
