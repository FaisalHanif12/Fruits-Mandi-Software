// Typed wrappers around the voucher commands (plan 10; module `vouchers`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.

import type { AdvanceInput } from '../../bindings/AdvanceInput';
import { callCommand } from '../../shared/api';
import type { AdvancesInfo, AdvancesPreview, ReadyAdvance, VoucherSaved } from './types';

/** What goes to the backend for one voucher: the text typed and the ids of what was chosen. */
function advanceInput(draft: ReadyAdvance): AdvanceInput {
  const bank = draft.mode !== 'cash';
  const cheque = draft.mode === 'cheque';
  return {
    date: draft.date,
    seller_account_id: draft.seller.id,
    amount: draft.amount,
    mode: draft.mode,
    bank_account_id: bank ? (draft.bank?.id ?? null) : null,
    cheque_no: cheque ? draft.cheque_no : '',
    cheque_date: cheque ? draft.cheque_date : '',
    expected_goods: draft.expected_goods,
    narration: draft.narration,
  };
}

/** Saves one advance: one posting event and one transaction (PRD 15.4, NFR-SPD-9). */
export function vouchersSaveAdvance(draft: ReadyAdvance): Promise<VoucherSaved> {
  return callCommand<VoucherSaved>('vouchers_save_advance', { input: advanceInput(draft) });
}

/**
 * Checks a list of advances and says, for each, the balance before and after and what is wrong,
 * and the total of the list. Saves nothing (the same function as the save, Code Standards B5).
 */
export function vouchersPreviewAdvances(drafts: readonly ReadyAdvance[]): Promise<AdvancesPreview> {
  return callCommand<AdvancesPreview>('vouchers_preview_advances', {
    input: { lines: drafts.map(advanceInput) },
  });
}

/** A seller's open advances and his last one, for his card (PRD FR-7.60). Read only. */
export function assistAdvances(sellerId: number): Promise<AdvancesInfo> {
  return callCommand<AdvancesInfo>('assist_advances', { input: { seller_id: sellerId } });
}
