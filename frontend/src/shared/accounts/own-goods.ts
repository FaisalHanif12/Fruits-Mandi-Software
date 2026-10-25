// The اپنا مال account (PRD FR-10.4, FR-12.26): the seller of a trip's goods in the sale book. The
// backend offers it only for the account scope `lot_seller` (the seller field of a lot) and refuses
// a posting to it anywhere else (ERR_OWN_GOODS_ACCOUNT_RESTRICTED). The code is fixed, so a screen
// that must know "is this lot a trip lot?" asks it here.

export const OWN_GOODS_CODE = '030119998';

/** True for the اپنا مال account. */
export function isOwnGoods(account: { code: string } | undefined | null): boolean {
  return account?.code === OWN_GOODS_CODE;
}

/** The control of the gardens' accounts, 0306 (PRD FR-10.4, FR-12.34). A code is the control plus a 4-digit serial. */
const GARDEN_CODE = /^0306\d{4}$/;

/**
 * Is this a garden's own account (PRD FR-12.34)? A lot with a garden as its seller gives no نقد
 * and no advance tick (PRD FR-12.36). The backend decides the rule; the screen only hides the
 * cells it does not apply to, so a typed نقد is never refused.
 */
export function isGardenAccount(account: { code: string } | undefined | null): boolean {
  return account !== undefined && account !== null && GARDEN_CODE.test(account.code);
}
