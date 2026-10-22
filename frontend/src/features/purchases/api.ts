// Typed wrappers around the خریداری commands (plan 16b; module `purchases`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.
// Every command takes one `{ input }` object.

import { callCommand } from '../../shared/api';
import type { PurchaseGetInput } from '../../bindings/PurchaseGetInput';
import type { PurchaseWriteOffInput } from '../../bindings/PurchaseWriteOffInput';
import type { PurchaseWriteOffUndoInput } from '../../bindings/PurchaseWriteOffUndoInput';
import type {
  PurchaseCancelInput,
  PurchaseCancelled,
  PurchaseInput,
  PurchasePreview,
  PurchaseRow,
  PurchaseSaved,
  PurchaseWriteOffDone,
  PurchaseWriteOffUndone,
  PurchasesList,
  PurchasesListInput,
} from './types';

/**
 * Works out a purchase as typed: its cost lines with their workings, each partner's part and the
 * cash after it. Saves nothing. The same function as the save (Code Standards B5).
 */
export function purchasesPreview(input: PurchaseInput): Promise<PurchasePreview> {
  return callCommand<PurchasePreview>('purchases_preview', { input });
}

/** Saves one purchase: one posting event, one transaction (PRD FR-12.24). */
export function purchasesSave(input: PurchaseInput): Promise<PurchaseSaved> {
  return callCommand<PurchaseSaved>('purchases_save', { input });
}

/** Cancels a purchase with a reason, by reversal, while nothing of it was sold or written off. */
export function purchasesCancel(input: PurchaseCancelInput): Promise<PurchaseCancelled> {
  return callCommand<PurchaseCancelled>('purchases_cancel', { input });
}

/** One page of purchases and the totals over the whole filter (PRD FR-19.10, FR-19.11). */
export function purchasesList(input: PurchasesListInput): Promise<PurchasesList> {
  return callCommand<PurchasesList>('purchases_list', { input });
}

/** One purchase, with the lots that sold it and its write-offs (PRD FR-12.25, FR-12.27). */
export function purchasesGet(id: number): Promise<PurchaseRow> {
  const input: PurchaseGetInput = { id };
  return callCommand<PurchaseRow>('purchases_get', { input });
}

/** Writes trip goods off with a reason; nothing is posted (PRD FR-12.28). */
export function purchasesWriteOff(
  purchaseId: number,
  qty: string,
  reason: string,
): Promise<PurchaseWriteOffDone> {
  const input: PurchaseWriteOffInput = { purchase_id: purchaseId, qty, reason };
  return callCommand<PurchaseWriteOffDone>('purchases_write_off', { input });
}

/** Undoes one write-off of trip goods with a reason (PRD FR-12.28). */
export function purchasesWriteOffUndo(
  moveId: number,
  reason: string,
): Promise<PurchaseWriteOffUndone> {
  const input: PurchaseWriteOffUndoInput = { move_id: moveId, reason };
  return callCommand<PurchaseWriteOffUndone>('purchases_write_off_undo', { input });
}
