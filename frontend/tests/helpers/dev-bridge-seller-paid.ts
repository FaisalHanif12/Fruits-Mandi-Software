// The stand-in answers of plan 91 (a seller's money paid on the spot) for the development page /dev/bikri?seller: the
// plan's first worked example (previous 30,000 بنام, today's credit 20,000) with the paid amount taken from the نقد typed
// on the seller's lots. A test double's arithmetic, never the app's: the app shows what the backend sends. Development
// only, like the rest of the stand-in.

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import { sellerLines } from './seller-paid';

/** The whole rupees typed as a lot's نقد, or 0. */
function typedCash(draft: BikriDraft): number {
  const sum = draft.lots.reduce(
    (total, lot) => total + Number((lot.cash_paid ?? '0').replaceAll(',', '') || '0'),
    0,
  );
  return Number.isFinite(sum) ? sum : 0;
}

/** The seller's lines for the draft: one seller (the first lot's), the plan's first example with this paid amount. */
export function devSellerLines(draft: BikriDraft) {
  const seller = draft.lots.find((lot) => lot.seller_account_id !== null)?.seller_account_id;
  if (seller === null || seller === undefined) return [];
  const paid = typedCash(draft);
  const credit = 20_000;
  const previous = 30_000;
  const above = paid > credit;
  const signedNew = previous + paid - credit;
  return [
    sellerLines({
      sellerId: seller,
      paid,
      credit,
      carry: Math.max(credit - paid, 0),
      cut: above ? previous : Math.max(credit - paid, 0),
      over: above ? paid - credit : 0,
      new: { rupees: Math.abs(signedNew), side: signedNew < 0 ? 'cr' : 'dr' },
      limit: credit,
      needsConfirm: above,
    }),
  ];
}

/** The spread: the whole amount on the first lot of the named seller (a test double; the backend spreads by lot order). */
export function devSpread(draft: BikriDraft, input: Record<string, unknown>) {
  const asked =
    (input['seller_paid'] as { seller_account_id: number; amount: string }[] | undefined) ?? [];
  const amount = Number((asked[0]?.amount ?? '0').replaceAll(',', '') || '0');
  const lots = draft.lots
    .filter((lot) => lot.seller_account_id === asked[0]?.seller_account_id)
    .map((lot, index) => ({ key: lot.key, cash_paid_paisa: (index === 0 ? amount : 0) * 100 }));
  return {
    lots,
    preview: {
      sellers: devSellerLines({
        ...draft,
        lots: draft.lots.map((lot) => ({ ...lot, cash_paid: String(amount) })),
      }),
    },
  };
}
