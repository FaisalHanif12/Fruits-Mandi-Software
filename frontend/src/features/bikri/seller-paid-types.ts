// INTERIM shapes of plan 91 (a seller's money paid on the spot), copied field for field from the backend's
// `core/bikri/input.rs` and `preview.rs` (`SellerPaidInput`, `SellerPaidSpreadInput`, `SellerPaidLines`,
// `LotCashSpread`, `SellerPaidSpread`) until `pnpm gen:types` has made them. When the bindings exist this file is
// deleted and the imports point at `src/bindings` (Code Standards F2: the frontend never hand-writes what the backend owns).

import type { BalanceSide } from '../../bindings/BalanceSide';
import type { BikriDraft as BackendDraft } from '../../bindings/BikriDraft';
import type { BikriPreview as BackendPreview } from '../../bindings/BikriPreview';
import type { Working } from '../../bindings/Working';
import type { Paisa } from '../../shared/format';

/** What one seller takes on the spot: the amount, as text, to spread over his lots' نقد. */
export interface SellerPaidInput {
  seller_account_id: number;
  /** Whole rupees as typed; a simple sum is taken; empty is ignored by the backend; `0` puts 0 on all his lots. */
  amount: string;
}

/** Input of `bikri_seller_paid_spread`. */
export interface SellerPaidSpreadInput {
  /** The draft as the screen holds it (for a mixed screen: the day and only the new lots). */
  draft: BackendDraft;
  /** The posting being edited, or null for a new بکری. */
  entry_id?: number | null;
  seller_paid: SellerPaidInput[];
}

/** The نقد of one lot, to write into its field. */
export interface LotCashSpread {
  key: string;
  cash_paid_paisa: Paisa;
}

/** The answer of `bikri_seller_paid_spread`. */
export interface SellerPaidSpread {
  lots: LotCashSpread[];
  preview: BikriPreviewWithSellers;
}

/** One seller's summary lines (PRD FR-12.23 v1.53). `dr` is بنام (he owes the shop), `cr` is جمع (the shop owes him). */
export interface SellerPaidLines {
  seller_account_id: number;
  previous_paisa: Paisa;
  previous_side: BalanceSide;
  credit_paisa: Paisa;
  paid_paisa: Paisa;
  cut_paisa: Paisa;
  added_to_credit_paisa: Paisa;
  over_paid_paisa: Paisa;
  increase_owed_paisa: Paisa;
  new_paisa: Paisa;
  new_side: BalanceSide;
  limit_paisa: Paisa;
  needs_confirm: boolean;
  working: Working;
}

/** The preview with the sellers' lines the backend now adds to it. */
export type BikriPreviewWithSellers = BackendPreview & { sellers?: SellerPaidLines[] };
