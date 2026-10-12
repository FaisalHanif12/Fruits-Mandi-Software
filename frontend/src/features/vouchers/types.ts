// INTERIM (plan 10 T3): the shape of the voucher commands as the plan's contract and the proposal
// sent to backend describe them (`vouchers_save_advance`, `vouchers_preview_advances`,
// `assist_advances`). When backend sends the generated bindings, `api.ts` is the only file that
// changes. Code Standards F2: types of backend data come from bindings. What the user types is
// text; amounts the backend sends are `Paisa` and `Balance`, shown and never worked out.

import type { LineError } from '../../bindings/LineError';
import type { Balance } from '../../shared/accounts';
import type { Paisa } from '../../shared/format';
import type { PickerValue } from '../../shared/ui';

/** How the money moves (PRD 15.1): cash, a bank account, or a cheque on a bank account. */
export type VoucherMode = 'cash' | 'bank' | 'cheque';

/** One voucher as the form holds it, all as text (CLAUDE.md rule 1). */
export interface AdvanceDraft {
  /** `DD-MM-YYYY` as typed. */
  date: string;
  seller: PickerValue | null;
  /** Whole rupees as typed. A simple sum such as `5000+2000` is read by the backend (FR-7.48). */
  amount: string;
  mode: VoucherMode;
  bank: PickerValue | null;
  cheque_no: string;
  /** `DD-MM-YYYY` as typed. */
  cheque_date: string;
  expected_goods: string;
  narration: string;
}

/** The answer of saving one voucher (PRD 15.1): its number and the seller's balance after it. */
export interface VoucherSaved {
  voucher_type: 'advance';
  /** The number in the series of its type (پ-1, پ-2 ...). Never reused. */
  number: number;
  entry_id: number;
  new_balance: Balance;
}

/** One line of the preview of a list of vouchers: what the backend makes of it. */
export interface AdvancePreviewRow {
  /** What is wrong with the line, as backend errors with their field. Empty if nothing. */
  errors: LineError[];
  /** The amount as the backend read it. Zero while there is none. */
  amount_paisa: Paisa;
  /** The seller's balance before this voucher, after the vouchers above it in the list. */
  current_balance: Balance;
  /** The seller's balance after this voucher. */
  new_balance: Balance;
}

/** The preview of the whole list: one row per voucher, and the total the backend worked out. */
export interface AdvancePreview {
  rows: AdvancePreviewRow[];
  total_paisa: Paisa;
}

/** One advance of a seller and how much of it is still open (PRD FR-7.60, FR-15.8). */
export interface AdvanceItem {
  /** `YYYY-MM-DD`. */
  date: string;
  number: number;
  amount_paisa: Paisa;
  adjusted_paisa: Paisa;
  open_paisa: Paisa;
}

/** A seller's advances for his card: what is open, the last one, and each one. */
export interface AdvancesSummary {
  open_paisa: Paisa;
  last: { date: string; number: number; amount_paisa: Paisa } | null;
  items: AdvanceItem[];
}
