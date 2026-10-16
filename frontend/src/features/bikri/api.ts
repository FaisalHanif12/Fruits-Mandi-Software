// Typed wrappers around the بکری commands (plans 12 and 13; module `bikri` and the `assist`
// reads, Tech Stack 5.2). Components never call invoke() (Code Standards F4). The only file that knows the
// wire shape. Every command takes one `{ input }` object.

import type { BikriDetail } from '../../bindings/BikriDetail';
import type { BikriFindInput } from '../../bindings/BikriFindInput';
import type { BikriGetInput } from '../../bindings/BikriGetInput';
import type { BikriListInput } from '../../bindings/BikriListInput';
import type { BikriListRow } from '../../bindings/BikriListRow';
import type { BikriNavigateInput } from '../../bindings/BikriNavigateInput';
import type { BikriPreviewInput } from '../../bindings/BikriPreviewInput';
import type { BikriSaveInput } from '../../bindings/BikriSaveInput';
import type { BikriSaved } from '../../bindings/BikriSaved';
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
