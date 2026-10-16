// Typed wrappers around the بکری commands (plan 12; module `bikri` and the `assist` reads, Tech
// Stack 5.2). Components never call invoke() (Code Standards F4). The only file that knows the
// wire shape. Every command takes one `{ input }` object.

import type { BikriPreviewInput } from '../../bindings/BikriPreviewInput';
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
