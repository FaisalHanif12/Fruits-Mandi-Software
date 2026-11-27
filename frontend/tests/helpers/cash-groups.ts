// Test helper: the types of the روکڑ بک filter and the kinds in each, as the backend groups them
// (`CashKind::group`, PRD FR-16.14 v1.60). Used by the fake books of the cash book.

import type { CashGroup } from '../../src/bindings/CashGroup';
import type { CashKind } from '../../src/bindings/CashKind';
import type { VoucherType } from '../../src/bindings/VoucherType';
import type { OpeningsToCashPreview } from '../../src/bindings/OpeningsToCashPreview';
import { money, ZERO } from './accounts';

/** The one-time fix of old openings with nothing to do (PRD FR-10.9). */
export function nothingToMove(): OpeningsToCashPreview {
  return {
    rows: [],
    total_dr_paisa: money(0),
    total_cr_paisa: money(0),
    cash_change_paisa: money(0),
    cash_before_paisa: money(0),
    cash_after_paisa: money(0),
    needs_cash_confirm: false,
    opening_difference_before: ZERO,
    opening_difference_after: ZERO,
    reopens_from: null,
  };
}

export const CASH_GROUPS: readonly (readonly [CashGroup, readonly CashKind[]])[] = [
  ['receipt', ['receipt']],
  ['payment', ['payment']],
  ['advance', ['advance', 'buyer_advance']],
  ['lent', ['cash_lent', 'money_lent', 'money_lent_return']],
  ['expense', ['shop_expense', 'personal_expense', 'personal_return']],
  ['bikri', ['bikri_paid', 'bikri_seller_cash', 'bikri_advance']],
  ['garden', ['garden_installment', 'garden_cost', 'garden_other_sale', 'garden_refund']],
  ['trip', ['purchase', 'trip_cost', 'trip_other_sale']],
  ['capital', ['capital_in', 'capital_out', 'opening_capital']],
  ['khata', ['khata_out', 'khata_in']],
  ['opening', ['opening_cash', 'opening_balance']],
  ['other', ['general', 'fruit', 'other']],
];

export function groupOfKind(kind: CashKind): CashGroup {
  return CASH_GROUPS.find(([, kinds]) => kinds.includes(kind))?.[0] ?? 'other';
}

/** The kind of a plain voucher row of the fake books. */
export function kindOfType(type: VoucherType): CashKind {
  return type;
}
