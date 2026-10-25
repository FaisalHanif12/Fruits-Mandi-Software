// Types of the پھل module (PRD 16.7, FR-16.10 to FR-16.12). What the backend owns comes from the
// generated bindings (Code Standards F2). What the user types is text (CLAUDE.md rule 1): the form
// holds the draft below, and the amounts the backend sends are `Paisa`, shown and never worked out.

import type { FruitCancelInput } from '../../bindings/FruitCancelInput';
import type { FruitCancelled } from '../../bindings/FruitCancelled';
import type { FruitCredit } from '../../bindings/FruitCredit';
import type { FruitEditInput } from '../../bindings/FruitEditInput';
import type { FruitEdited } from '../../bindings/FruitEdited';
import type { FruitGetInput } from '../../bindings/FruitGetInput';
import type { FruitInput } from '../../bindings/FruitInput';
import type { FruitList } from '../../bindings/FruitList';
import type { FruitListInput } from '../../bindings/FruitListInput';
import type { FruitLotBalance } from '../../bindings/FruitLotBalance';
import type { FruitLotRef } from '../../bindings/FruitLotRef';
import type { FruitLotRow } from '../../bindings/FruitLotRow';
import type { FruitLots } from '../../bindings/FruitLots';
import type { FruitLotsInput } from '../../bindings/FruitLotsInput';
import type { FruitNumbers } from '../../bindings/FruitNumbers';
import type { FruitPart } from '../../bindings/FruitPart';
import type { FruitPartRow } from '../../bindings/FruitPartRow';
import type { FruitPreview } from '../../bindings/FruitPreview';
import type { FruitRow } from '../../bindings/FruitRow';
import type { FruitSaved } from '../../bindings/FruitSaved';
import type { FruitSource } from '../../bindings/FruitSource';
import type { FruitStatus } from '../../bindings/FruitStatus';
import type { FruitTotals } from '../../bindings/FruitTotals';
import type { PickerValue } from '../../shared/ui';

export type {
  FruitCancelInput,
  FruitCancelled,
  FruitCredit,
  FruitEditInput,
  FruitEdited,
  FruitGetInput,
  FruitInput,
  FruitList,
  FruitListInput,
  FruitLotBalance,
  FruitLotRef,
  FruitLotRow,
  FruitLots,
  FruitLotsInput,
  FruitNumbers,
  FruitPart,
  FruitPartRow,
  FruitPreview,
  FruitRow,
  FruitSaved,
  FruitSource,
  FruitStatus,
  FruitTotals,
};

/** The value of the "for" list that means the shop's guests (PRD 16.7). Partners use their id. */
export const FOR_GUESTS = 'guests';

/** The lot chosen for fruit from our shop, kept with the row the picker found. */
export interface ChosenLot {
  /** What the picker shows for the field (both names, so a language switch needs no ask). */
  value: PickerValue;
  /** The lot as `fruit_lots` answered it, or null for the lot of a saved entry being edited. */
  row: FruitLotRow | null;
}

/**
 * The form as the user typed it. Everything is text or a choice; the backend reads and checks it
 * (Code Standards 4.1). The item and packing of fruit from our shop come from the lot.
 */
export interface FruitDraft {
  /** `''` (not chosen), `FOR_GUESTS`, or the id of a partner as text. */
  forWhom: string;
  date: string;
  source: FruitSource;
  lot: ChosenLot | null;
  agent: PickerValue | null;
  item: PickerValue | null;
  packing: PickerValue | null;
  qty: string;
  weight: string;
  rate: string;
  note: string;
  /** Only when an entry is being edited: why (PRD FR-16.12). */
  reason: string;
}

/** Which entry the form is editing: the id and version read, so the backend can tell a stale edit. */
export interface EditingFruit {
  id: number;
  version: number;
  number: number;
}

/** What a save sends: the entry, and the id, version and reason when it is an edit. */
export interface FruitSaveRequest {
  entry: FruitInput;
  edit: { id: number; version: number; reason: string } | null;
}
