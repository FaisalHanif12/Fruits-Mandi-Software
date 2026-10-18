// Typed wrappers around the بکری commands (plans 12 and 13; module `bikri` and the `assist`
// reads, Tech Stack 5.2). Components never call invoke() (Code Standards F4). The only file that knows the
// wire shape. Every command takes one `{ input }` object.

import type { BikriCancelled } from '../../bindings/BikriCancelled';
import type { BikriDetail } from '../../bindings/BikriDetail';
import type { BikriEdited } from '../../bindings/BikriEdited';
import type { BikriFindInput } from '../../bindings/BikriFindInput';
import type { BikriGetInput } from '../../bindings/BikriGetInput';
import type { BikriListInput } from '../../bindings/BikriListInput';
import type { BikriListRow } from '../../bindings/BikriListRow';
import type { BikriNavigateInput } from '../../bindings/BikriNavigateInput';
import type { BikriPreviewInput } from '../../bindings/BikriPreviewInput';
import type { BikriSaveInput } from '../../bindings/BikriSaveInput';
import type { BikriSaved } from '../../bindings/BikriSaved';
import type { CancelInput } from '../../bindings/CancelInput';
import type { CancelPreview } from '../../bindings/CancelPreview';
import type { CancelPreviewInput } from '../../bindings/CancelPreviewInput';
import type { EditableBikri } from '../../bindings/EditableBikri';
import type { EditOpenInput } from '../../bindings/EditOpenInput';
import type { EditPreviewInput } from '../../bindings/EditPreviewInput';
import type { EditSaveInput } from '../../bindings/EditSaveInput';
import type { LaterSaleInput } from '../../bindings/LaterSaleInput';
import type { LaterSaleSaved } from '../../bindings/LaterSaleSaved';
import type { LaterSaleSaveInput } from '../../bindings/LaterSaleSaveInput';
import type { SaleInput } from '../../bindings/SaleInput';
import type { StockList } from '../../bindings/StockList';
import type { StockListInput } from '../../bindings/StockListInput';
import type { UndoInput } from '../../bindings/UndoInput';
import type { UndoResult } from '../../bindings/UndoResult';
import type { WriteOffDone } from '../../bindings/WriteOffDone';
import type { WriteOffInput } from '../../bindings/WriteOffInput';
import type { WriteOffUndoInput } from '../../bindings/WriteOffUndoInput';
import type { WriteOffUndone } from '../../bindings/WriteOffUndone';
import type { NavigateTo } from '../../bindings/NavigateTo';
import type { FreightLotInput } from '../../bindings/FreightLotInput';
import type { FreightPart } from '../../bindings/FreightPart';
import type { SellerMemoryInput } from '../../bindings/SellerMemoryInput';
import type { SplitFreightInput } from '../../bindings/SplitFreightInput';
import type { TruckMemoryInput } from '../../bindings/TruckMemoryInput';
import { callCommand } from '../../shared/api';
import type { BikriDraft, BikriPreview, SellerMemory, TruckMemory } from './types';

/**
 * Works out the whole بکری as typed and says every number, the live partner split, the cash after
 * the entry, and what is wrong. Saves nothing. It is the same function as the save (Code Standards
 * B5), so the screen and the books never differ (PRD FR-6.1).
 */
export function bikriPreview(draft: BikriDraft): Promise<BikriPreview> {
  const input: BikriPreviewInput = { draft };
  return callCommand<BikriPreview>('bikri_preview', { input });
}

/** Splits the header's total freight over the lots by their نگ (PRD C17). Read only. */
export function bikriSplitFreight(
  totalFreight: string,
  lots: readonly FreightLotInput[],
): Promise<FreightPart[]> {
  const input: SplitFreightInput = { total_freight: totalFreight, lots: [...lots] };
  return callCommand<FreightPart[]>('bikri_split_freight', { input });
}

/** What the app remembers about a truck number and the same-day notice (PRD FR-7.49, FR-7.50). */
export function assistTruckMemory(truckNo: string, date: string): Promise<TruckMemory> {
  const input: TruckMemoryInput = { truck_no: truckNo, date };
  return callCommand<TruckMemory>('assist_truck_memory', { input });
}

/** What the app remembers about a seller: item, packing and مارکہ of his last lot (FR-7.51). */
export function assistSellerMemory(sellerAccountId: number): Promise<SellerMemory> {
  const input: SellerMemoryInput = { seller_account_id: sellerAccountId };
  return callCommand<SellerMemory>('assist_seller_memory', { input });
}

/**
 * Saves the بکری as one action, all or nothing (PRD FR-12.8). The draft goes as the text typed and
 * the backend works it out again with the same function as the preview (Code Standards B5).
 * `confirmCashBelowZero` is true only after the user said yes to "is this entry correct?"
 * (PRD FR-6.7). It is always sent, true or false.
 */
export function bikriSave(draft: BikriDraft, confirmCashBelowZero: boolean): Promise<BikriSaved> {
  const input: BikriSaveInput = { draft, confirm_cash_below_zero: confirmCashBelowZero };
  return callCommand<BikriSaved>('bikri_save', { input });
}

/** A saved بکری with the values stored when it was saved, never worked out again. */
export function bikriGet(id: number): Promise<BikriDetail> {
  const input: BikriGetInput = { id };
  return callCommand<BikriDetail>('bikri_get', { input });
}

/** The list of بکری (PRD FR-12.7). Dates are text as typed; empty means no limit. */
export function bikriList(input: BikriListInput): Promise<BikriListRow[]> {
  return callCommand<BikriListRow[]>('bikri_list', { input });
}

/** Finds بکری by number, truck, seller, day or item (PRD FR-12.13). Newest first, at most 50. */
export function bikriFind(input: BikriFindInput): Promise<BikriListRow[]> {
  return callCommand<BikriListRow[]>('bikri_find', { input });
}

/** The id of the first, previous, next or last بکری by number, or null when there is none. */
export function bikriNavigate(currentId: number | null, to: NavigateTo): Promise<number | null> {
  const input: BikriNavigateInput = { current_id: currentId, to };
  return callCommand<number | null>('bikri_navigate', { input });
}

// Plan 16: goods left in stock, later sales, edit, cancel, undo and write-off (PRD 12.8, 12.9,
// FR-7.47). Every number comes back from the backend; the screen sends only text and ids.

/** The lots with goods left, oldest first (PRD FR-12.14, FR-19.10). Read only. */
export function bikriStockList(input: StockListInput): Promise<StockList> {
  return callCommand<StockList>('bikri_stock_list', { input });
}

/** Works out new sale lines of a lot dated `date`. Saves nothing (PRD FR-12.17, Code Standards B5). */
export function bikriLaterSalePreview(
  lotId: number,
  date: string,
  sales: readonly SaleInput[],
): Promise<BikriPreview> {
  const input: LaterSaleInput = { lot_id: lotId, date, sales: [...sales] };
  return callCommand<BikriPreview>('bikri_later_sale_preview', { input });
}

/** Saves the later sale as its own posting. `confirmCashBelowZero` is always sent (PRD FR-6.7). */
export function bikriLaterSaleSave(
  lotId: number,
  date: string,
  sales: readonly SaleInput[],
  confirmCashBelowZero: boolean,
): Promise<LaterSaleSaved> {
  const input: LaterSaleSaveInput = {
    lot_id: lotId,
    date,
    sales: [...sales],
    confirm_cash_below_zero: confirmCashBelowZero,
  };
  return callCommand<LaterSaleSaved>('bikri_later_sale_save', { input });
}

/** Loads one posting of a saved بکری as a draft to change (PRD FR-12.18). */
export function bikriOpenForEdit(bikriId: number, entryId: number): Promise<EditableBikri> {
  const input: EditOpenInput = { bikri_id: bikriId, entry_id: entryId };
  return callCommand<EditableBikri>('bikri_open_for_edit', { input });
}

/** The edited draft worked out as the save will. Saves nothing. */
export function bikriEditPreview(entryId: number, draft: BikriDraft): Promise<BikriPreview> {
  const input: EditPreviewInput = { entry_id: entryId, draft };
  return callCommand<BikriPreview>('bikri_edit_preview', { input });
}

/** Saves an edit: a reversal of the posting plus a new posting on the same day (PRD FR-12.19). */
export function bikriEditSave(
  entryId: number,
  version: number,
  draft: BikriDraft,
  reason: string | null,
  confirmCashBelowZero: boolean,
): Promise<BikriEdited> {
  const input: EditSaveInput = {
    entry_id: entryId,
    version,
    draft,
    reason,
    confirm_cash_below_zero: confirmCashBelowZero,
  };
  return callCommand<BikriEdited>('bikri_edit_save', { input });
}

/** What a cancel would do, for the confirm box (PRD FR-7.8, FR-6.7). Read only. */
export function bikriCancelPreview(id: number): Promise<CancelPreview> {
  const input: CancelPreviewInput = { id };
  return callCommand<CancelPreview>('bikri_cancel_preview', { input });
}

/** Cancels the whole بکری by reversal. A reason is required (PRD FR-12.11, FR-12.22). */
export function bikriCancel(
  id: number,
  version: number,
  reason: string,
  confirmCashBelowZero: boolean,
): Promise<BikriCancelled> {
  const input: CancelInput = {
    id,
    version,
    reason,
    confirm_cash_below_zero: confirmCashBelowZero,
  };
  return callCommand<BikriCancelled>('bikri_cancel', { input });
}

/** Takes back a بکری just saved and gives its draft back (PRD FR-7.47). */
export function bikriUndo(id: number, confirmCashBelowZero: boolean): Promise<UndoResult> {
  const input: UndoInput = { id, confirm_cash_below_zero: confirmCashBelowZero };
  return callCommand<UndoResult>('bikri_undo', { input });
}

/** Writes goods off a lot with a reason (PRD FR-12.16). The quantity goes as typed. */
export function bikriWriteOff(lotId: number, qty: string, reason: string): Promise<WriteOffDone> {
  const input: WriteOffInput = { lot_id: lotId, qty, reason };
  return callCommand<WriteOffDone>('bikri_write_off', { input });
}

/** Undoes one write-off with a reason (PRD FR-12.16). */
export function bikriWriteOffUndo(moveId: number, reason: string): Promise<WriteOffUndone> {
  const input: WriteOffUndoInput = { move_id: moveId, reason };
  return callCommand<WriteOffUndone>('bikri_write_off_undo', { input });
}
