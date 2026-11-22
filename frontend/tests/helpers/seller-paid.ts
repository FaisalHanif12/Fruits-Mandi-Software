// Test helper for plan 91 (a seller's money paid on the spot): the lines of a seller's account as the backend sends them
// in `BikriPreview.sellers`, and the answer of `bikri_seller_paid_spread`. The worked numbers are the plan's (PRD FR-12.23
// v1.53): previous 30,000 بنام, today's credit 20,000, paid 5,000: cut 15,000, new 15,000 بنام. The shapes are the interim
// ones of the generated bindings; the spread answer is the generated `SellerPaidSpread`.

import type { BalanceSide } from '../../src/bindings/BalanceSide';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { SellerPaidLines } from '../../src/bindings/SellerPaidLines';
import type { SellerPaidSpread } from '../../src/bindings/SellerPaidSpread';
import { paisaForTest } from './paisa';
import { working } from './trips-data';

const rupees = (value: number) => paisaForTest(value * 100);

export interface SellerLinesOptions {
  sellerId: number;
  previous?: { rupees: number; side: BalanceSide };
  credit?: number;
  paid?: number;
  carry?: number;
  cut?: number;
  added?: number;
  over?: number;
  increase?: number;
  new?: { rupees: number; side: BalanceSide };
  limit?: number;
  needsConfirm?: boolean;
}

/** One seller's lines: the plan's first worked example unless the options say otherwise. */
export function sellerLines(options: SellerLinesOptions): SellerPaidLines {
  const previous = options.previous ?? { rupees: 30_000, side: 'dr' };
  const next = options.new ?? { rupees: 15_000, side: 'dr' };
  return {
    seller_account_id: options.sellerId,
    previous_paisa: rupees(previous.rupees),
    previous_side: previous.side,
    credit_paisa: rupees(options.credit ?? 20_000),
    paid_paisa: rupees(options.paid ?? 5_000),
    carry_paisa: rupees(
      options.carry ?? Math.max((options.credit ?? 20_000) - (options.paid ?? 5_000), 0),
    ),
    cut_paisa: rupees(options.cut ?? 15_000),
    added_to_credit_paisa: rupees(options.added ?? 0),
    over_paid_paisa: rupees(options.over ?? 0),
    increase_owed_paisa: rupees(options.increase ?? 0),
    new_paisa: rupees(next.rupees),
    new_side: next.side,
    limit_paisa: rupees(options.limit ?? 20_000),
    needs_confirm: options.needsConfirm ?? false,
    working: working('working-seller-new-balance', [
      { name: 'previous', value: { kind: 'money', paisa: rupees(previous.rupees) } },
      { name: 'paid', value: { kind: 'money', paisa: rupees(options.paid ?? 5_000) } },
      { name: 'credit', value: { kind: 'money', paisa: rupees(options.credit ?? 20_000) } },
      { name: 'result', value: { kind: 'money', paisa: rupees(next.rupees) } },
    ]),
  };
}

/** A preview with the sellers' lines put in. */
export function withSellers(
  preview: BikriPreview,
  lines: readonly SellerPaidLines[],
): BikriPreview {
  return { ...preview, sellers: [...lines] };
}

/** The answer of the spread: the lots named with their نقد, and a preview to go with it. */
export function spreadAnswer(
  preview: BikriPreview,
  lots: readonly { key: string; rupees: number }[],
): SellerPaidSpread {
  return {
    lots: lots.map((lot) => ({ key: lot.key, cash_paid_paisa: rupees(lot.rupees) })),
    preview,
  };
}
