// Typed wrappers around the voucher commands (plans 10 and 15; module `vouchers`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.

import { callCommand } from '../../shared/api';
import type { AdvancesInfo } from '../../bindings/AdvancesInfo';
import type { CollectionList } from '../../bindings/CollectionList';
import type { CollectionListInput } from '../../bindings/CollectionListInput';
import type { OpenBills } from '../../bindings/OpenBills';
import type { OpenBillsInput } from '../../bindings/OpenBillsInput';
import type { OpenPayables } from '../../bindings/OpenPayables';
import type { OpenPayablesInput } from '../../bindings/OpenPayablesInput';
import type { ReceiptInput } from '../../bindings/ReceiptInput';
import type { ReceiptsSaved } from '../../bindings/ReceiptsSaved';
import type { VouchersList } from '../../bindings/VouchersList';
import type { VouchersListInput } from '../../bindings/VouchersListInput';
import type { BuiltVoucher } from './draft';
import type {
  VoucherCancelled,
  VoucherEntry,
  VoucherPreview,
  VoucherRow,
  VoucherSaved,
} from './types';

/**
 * Checks an entry and says what saving it would do: the balance before and after, the bills a
 * receipt settles, the payable, the warnings and the cash after (PRD FR-7.58, FR-7.59, FR-6.7).
 * Saves nothing (the same function as the save, Code Standards B5). `replacing` is the voucher an
 * edit would take back first.
 */
export function vouchersPreview(
  entry: VoucherEntry,
  replacing: number | null,
): Promise<VoucherPreview> {
  return callCommand<VoucherPreview>('vouchers_preview', {
    input: { entry, replacing_voucher_id: replacing },
  });
}

/** Saves one voucher: one posting event and one transaction (PRD 15.2 to 15.4, NFR-SPD-9). */
export function vouchersSave(built: BuiltVoucher): Promise<VoucherSaved> {
  switch (built.type) {
    case 'receipt':
      return callCommand<VoucherSaved>('vouchers_save_receipt', { input: built.input });
    case 'payment':
      return callCommand<VoucherSaved>('vouchers_save_payment', { input: built.input });
    case 'advance':
      return callCommand<VoucherSaved>('vouchers_save_advance', { input: built.input });
  }
}

/**
 * Edits a saved voucher while its day is open (PRD FR-15.3): the old posting is reversed and the
 * new one posted in one transaction. `version` is the one read with the voucher.
 */
export function vouchersEdit(input: {
  id: number;
  version: number;
  entry: VoucherEntry;
  reason: string;
  confirm: boolean;
}): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_edit', {
    input: {
      id: input.id,
      version: input.version,
      entry: input.entry,
      reason: input.reason,
      confirm_cash_below_zero: input.confirm,
    },
  });
}

/** Cancels a saved voucher with a reason, by reversal (PRD FR-15.3). */
export function vouchersCancel(input: {
  id: number;
  version: number;
  reason: string;
  confirm: boolean;
}): Promise<VoucherCancelled> {
  return callCommand<VoucherCancelled>('vouchers_cancel', {
    input: {
      id: input.id,
      version: input.version,
      reason: input.reason,
      confirm_cash_below_zero: input.confirm,
    },
  });
}

/** One saved voucher, with its version and whether it can still be edited. */
export function vouchersGet(id: number): Promise<VoucherRow> {
  return callCommand<VoucherRow>('vouchers_get', { input: { id } });
}

/** The vouchers of a kind in a date range, newest first (the list under the form). */
export function vouchersList(input: VouchersListInput): Promise<VouchersList> {
  return callCommand<VouchersList>('vouchers_list', { input });
}

/** A buyer's open bills, oldest first (PRD FR-7.58). Read only. */
export function assistOpenBills(input: OpenBillsInput): Promise<OpenBills> {
  return callCommand<OpenBills>('assist_open_bills', { input });
}

/** A seller's payable postings, oldest first, with the advance already adjusted (FR-7.59). */
export function assistOpenPayables(input: OpenPayablesInput): Promise<OpenPayables> {
  return callCommand<OpenPayables>('assist_open_payables', { input });
}

/** A seller's open advances and his last one, for his card (PRD FR-7.60). Read only. */
export function assistAdvances(sellerId: number): Promise<AdvancesInfo> {
  return callCommand<AdvancesInfo>('assist_advances', { input: { seller_id: sellerId } });
}

/** The اگراہی فہرست of a day: the buyers planned for it, by collector and city (FR-15.6, FR-7.61). */
export function assistCollectionList(input: CollectionListInput): Promise<CollectionList> {
  return callCommand<CollectionList>('assist_collection_list', { input });
}

/**
 * Saves the receipts of the collection round, each as its own posting event. A row with no amount
 * is skipped. The answer has one result for each row, with its problems (PRD FR-7.61, FR-15.1).
 */
export function vouchersSaveReceiptsBatch(rows: readonly ReceiptInput[]): Promise<ReceiptsSaved> {
  return callCommand<ReceiptsSaved>('vouchers_save_receipts_batch', { input: { rows } });
}
