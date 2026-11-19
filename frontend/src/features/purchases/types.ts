// Types of the خریداری part of a trip (PRD 12.10). What the backend owns comes from the generated
// bindings (Code Standards F2). What the user types is text (CLAUDE.md rule 1): the form holds
// `PurchaseEntryFields` and the rows of the lines grid, and the amounts the backend sends are `Paisa`, shown and never worked out.

import type { OwnStockRow } from '../../bindings/OwnStockRow';
import type { PurchaseCancelInput } from '../../bindings/PurchaseCancelInput';
import type { PurchaseCancelled } from '../../bindings/PurchaseCancelled';
import type { PurchaseEntryInput } from '../../bindings/PurchaseEntryInput';
import type { PurchaseEntryPreview } from '../../bindings/PurchaseEntryPreview';
import type { PurchaseEntrySaved } from '../../bindings/PurchaseEntrySaved';
import type { PurchaseInput } from '../../bindings/PurchaseInput';
import type { PurchaseLotRef } from '../../bindings/PurchaseLotRef';
import type { PurchaseNumbers } from '../../bindings/PurchaseNumbers';
import type { PurchasePreview } from '../../bindings/PurchasePreview';
import type { PurchaseRow } from '../../bindings/PurchaseRow';
import type { PurchaseSaved } from '../../bindings/PurchaseSaved';
import type { PurchaseStateFilter } from '../../bindings/PurchaseStateFilter';
import type { PurchaseWriteOffDone } from '../../bindings/PurchaseWriteOffDone';
import type { PurchaseWriteOffUndone } from '../../bindings/PurchaseWriteOffUndone';
import type { PurchasesList } from '../../bindings/PurchasesList';
import type { PurchasesListInput } from '../../bindings/PurchasesListInput';
import type { SplitInput } from '../../bindings/SplitInput';
import type { SplitSet } from '../../bindings/SplitSet';
import type { TripRef } from '../../bindings/TripRef';
import type { WriteOffRow } from '../../bindings/WriteOffRow';

export type {
  OwnStockRow,
  PurchaseCancelInput,
  PurchaseCancelled,
  PurchaseEntryInput,
  PurchaseEntryPreview,
  PurchaseEntrySaved,
  PurchaseInput,
  PurchaseLotRef,
  PurchaseNumbers,
  PurchasePreview,
  PurchaseRow,
  PurchaseSaved,
  PurchaseStateFilter,
  PurchaseWriteOffDone,
  PurchaseWriteOffUndone,
  PurchasesList,
  PurchasesListInput,
  SplitInput,
  SplitSet,
  TripRef,
  WriteOffRow,
};

/**
 * The fields of a purchase entry that are not on a line (PRD FR-12.41), all as text (CLAUDE.md
 * rule 1). The day, the note and the sources of the partners are the same on every type of the
 * trip screen, so the screen holds them once (PRD FR-7.67). The item lines are rows in the grid's
 * store, not here.
 */
export interface PurchaseEntryFields {
  /** Who the goods were bought from, as typed (PRD FR-7.25). Nothing is owed to him (A53). */
  supplier: string;
  /**
   * The day to sell (PRD FR-12.39) as typed, `DD-MM-YYYY`. Empty means the day of the purchase:
   * the form then shows that day in the box, and sends nothing. It applies to every line.
   */
  sellDate: string;
}

/**
 * What the write-off boxes need of a trip stock record. A purchase row and a row of باقی مال both
 * give it, so one pair of boxes serves the trip screen and the list of باقی مال.
 */
export interface OwnStockTarget {
  purchaseId: number;
  number: number;
  item: PurchaseRow['item'];
  packing: PurchaseRow['packing'];
  /** نگ left, from the backend. */
  qtyLeft: number;
  writeOffs: readonly WriteOffRow[];
}
