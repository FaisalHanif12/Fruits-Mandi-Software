// Types of the خریداری part of a trip (PRD 12.10). What the backend owns comes from the generated
// bindings (Code Standards F2). What the user types is text (CLAUDE.md rule 1): the form holds
// `PurchaseFields`, and the amounts the backend sends are `Paisa`, shown and never worked out.

import type { OwnStockRow } from '../../bindings/OwnStockRow';
import type { PurchaseCancelInput } from '../../bindings/PurchaseCancelInput';
import type { PurchaseCancelled } from '../../bindings/PurchaseCancelled';
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
import type { TripRef } from '../../bindings/TripRef';
import type { WriteOffRow } from '../../bindings/WriteOffRow';
import type { PickerValue } from '../../shared/ui';

export type {
  OwnStockRow,
  PurchaseCancelInput,
  PurchaseCancelled,
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
  TripRef,
  WriteOffRow,
};

/**
 * The fields only a purchase has, all as text (CLAUDE.md rule 1). The day, the note and the
 * sources of the partners are the same on every type of the trip screen, so the screen holds
 * them once (PRD FR-7.67).
 */
export interface PurchaseFields {
  /** Who the goods were bought from, as typed (PRD FR-7.25). Nothing is owed to him (A53). */
  supplier: string;
  item: PickerValue | null;
  packing: PickerValue | null;
  /** نگ as typed. A simple sum such as `20+25` is read by the backend (FR-7.48). */
  qty: string;
  /** Kilograms, for an item sold by weight. */
  weight: string;
  /** Whole rupees as typed, for each نگ or each kg. */
  rate: string;
  freight: string;
  labour: string;
  otherCost: string;
  /**
   * The day to sell (PRD FR-12.39) as typed, `DD-MM-YYYY`. Empty means the day of the purchase:
   * the form then shows that day in the box, and sends nothing.
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
