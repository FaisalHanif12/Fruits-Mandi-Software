// The اپنا مال account (PRD FR-10.4, FR-12.26): the seller of a trip's goods in the sale book. The
// backend offers it only for the account scope `lot_seller` (the seller field of a lot) and refuses
// a posting to it anywhere else (ERR_OWN_GOODS_ACCOUNT_RESTRICTED). The code is fixed, so a screen
// that must know "is this lot a trip lot?" asks it here.

export const OWN_GOODS_CODE = '030119998';

/** True for the اپنا مال account. */
export function isOwnGoods(account: { code: string } | undefined | null): boolean {
  return account?.code === OWN_GOODS_CODE;
}
