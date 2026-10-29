// Types of the باغات module (PRD 12.11, FR-12.34 to FR-12.38). What the backend owns comes from
// the generated bindings (Code Standards F2). What the user types is text (CLAUDE.md rule 1): the
// forms hold the drafts below, and the amounts the backend sends are `Paisa`, shown and never
// worked out.

import type { ArrivalCancelInput } from '../../bindings/ArrivalCancelInput';
import type { ArrivalCancelled } from '../../bindings/ArrivalCancelled';
import type { ArrivalInput } from '../../bindings/ArrivalInput';
import type { ArrivalRow } from '../../bindings/ArrivalRow';
import type { ArrivalSaved } from '../../bindings/ArrivalSaved';
import type { ArrivalStateFilter } from '../../bindings/ArrivalStateFilter';
import type { ArrivalsList } from '../../bindings/ArrivalsList';
import type { ArrivalsListInput } from '../../bindings/ArrivalsListInput';
import type { ArrivalWriteOffDone } from '../../bindings/ArrivalWriteOffDone';
import type { ArrivalWriteOffInput } from '../../bindings/ArrivalWriteOffInput';
import type { ArrivalWriteOffUndoInput } from '../../bindings/ArrivalWriteOffUndoInput';
import type { ArrivalWriteOffUndone } from '../../bindings/ArrivalWriteOffUndone';
import type { EntryPartInput } from '../../bindings/EntryPartInput';
import type { SellDateInput } from '../../bindings/SellDateInput';
import type { SellDateSet } from '../../bindings/SellDateSet';
import type { GardenCloseInput } from '../../bindings/GardenCloseInput';
import type { GardenCloseNumbers } from '../../bindings/GardenCloseNumbers';
import type { GardenClosePreview } from '../../bindings/GardenClosePreview';
import type { GardenClosed } from '../../bindings/GardenClosed';
import type { GardenCreatePreview } from '../../bindings/GardenCreatePreview';
import type { GardenCreated } from '../../bindings/GardenCreated';
import type { GardenDefaults } from '../../bindings/GardenDefaults';
import type { GardenDetail } from '../../bindings/GardenDetail';
import type { GardenEntryCancelInput } from '../../bindings/GardenEntryCancelInput';
import type { GardenEntryCancelled } from '../../bindings/GardenEntryCancelled';
import type { GardenEntryInput } from '../../bindings/GardenEntryInput';
import type { GardenEntryNumbers } from '../../bindings/GardenEntryNumbers';
import type { GardenEntryPreview } from '../../bindings/GardenEntryPreview';
import type { GardenEntrySaved } from '../../bindings/GardenEntrySaved';
import type { GardenGetInput } from '../../bindings/GardenGetInput';
import type { GardenHistoryKind } from '../../bindings/GardenHistoryKind';
import type { GardenHistoryRow } from '../../bindings/GardenHistoryRow';
import type { GardenInput } from '../../bindings/GardenInput';
import type { GardenPartnerResult } from '../../bindings/GardenPartnerResult';
import type { GardenReopenInput } from '../../bindings/GardenReopenInput';
import type { GardenReopened } from '../../bindings/GardenReopened';
import type { GardenRow } from '../../bindings/GardenRow';
import type { GardenSetPartnersInput } from '../../bindings/GardenSetPartnersInput';
import type { GardenSummaryRow } from '../../bindings/GardenSummaryRow';
import type { GardensList } from '../../bindings/GardensList';
import type { GardensListInput } from '../../bindings/GardensListInput';
import type { GardensSummary } from '../../bindings/GardensSummary';
import type { GardensTotals } from '../../bindings/GardensTotals';
import type { MoneySource } from '../../bindings/MoneySource';
import type { VenturePartnerInput } from '../../bindings/VenturePartnerInput';
import type { VenturePartnerRow } from '../../bindings/VenturePartnerRow';
import type { VentureStatus } from '../../bindings/VentureStatus';
import type { NamePairValue, PickerValue } from '../../shared/ui';

export type {
  ArrivalCancelInput,
  ArrivalCancelled,
  ArrivalInput,
  ArrivalRow,
  ArrivalSaved,
  ArrivalStateFilter,
  ArrivalsList,
  ArrivalsListInput,
  ArrivalWriteOffDone,
  ArrivalWriteOffInput,
  ArrivalWriteOffUndoInput,
  ArrivalWriteOffUndone,
  EntryPartInput,
  GardenCloseInput,
  GardenCloseNumbers,
  GardenClosePreview,
  GardenClosed,
  GardenCreatePreview,
  GardenCreated,
  GardenDefaults,
  GardenDetail,
  GardenEntryCancelInput,
  GardenEntryCancelled,
  GardenEntryInput,
  GardenEntryNumbers,
  GardenEntryPreview,
  GardenEntrySaved,
  GardenGetInput,
  GardenHistoryKind,
  GardenHistoryRow,
  GardenInput,
  GardenPartnerResult,
  GardenReopenInput,
  GardenReopened,
  GardenRow,
  GardenSetPartnersInput,
  GardenSummaryRow,
  GardensList,
  GardensListInput,
  GardensSummary,
  GardensTotals,
  MoneySource,
  SellDateInput,
  SellDateSet,
  VenturePartnerInput,
  VenturePartnerRow,
  VentureStatus,
};

/** The types of the type list on the garden screen (PRD FR-7.67), in the order they are shown. */
export type EntryType = 'installment' | 'cost' | 'other_sale' | 'arrival';

export const ENTRY_TYPES: readonly EntryType[] = ['installment', 'cost', 'other_sale', 'arrival'];

/** The new garden form as text (PRD FR-12.29): the shares first, then the sources, then the details. */
export interface GardenDraft {
  /** One row for every active partner, in the order the partners were created. */
  partners: GardenDraftPartner[];
  /** The two names, with the words of them the user still has to check (PRD FR-7.66). */
  names: NamePairValue;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** The main fruit, or null. */
  item: PickerValue | null;
  /** The full rent for the season, whole rupees as typed. */
  rent: string;
  /** The first installment as typed, or empty. */
  paidNow: string;
  /**
   * Why the first installment was paid (PRD FR-12.35). The screen fills it with the standard words
   * like a typed value when the amount is typed (Code Standards F43); the Owner can change it.
   */
  paidNowReason: string;
  /** `DD-MM-YYYY` as typed, or empty. */
  seasonEnd: string;
  note: string;
}

export interface GardenDraftPartner {
  partner: VenturePartnerRow['partner'];
  /** The % as typed, for example "40" or "33.33". Empty means he is not in the garden. */
  share: string;
  source: MoneySource;
}

/**
 * Everything typed on the garden screen's entry form. The day, the amount and the note are the
 * same on every type, so changing the type keeps them and clears the rest (PRD FR-7.67). The
 * sources of the partners are shared by the two payments (an installment and a cost).
 */
export interface EntryDraft {
  type: EntryType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** The payment, or the money received for a sale in another shop. May be a simple sum. */
  amount: string;
  note: string;
  /** This entry's choice of source by partner id. A missing one means the garden's source. */
  sources: Record<number, MoneySource>;
  /** Why: an installment or a running cost. */
  reason: string;
  /** The shop, as typed, and the نگ sold: a sale in another shop. */
  shop: string;
  /** The نگ sold in another shop, or the نگ that arrived (goods arrived, PRD FR-12.40). */
  qty: string;
  /** Goods arrived: the item and packing, the weight of an item sold by weight, and the day to sell. */
  item: PickerValue | null;
  packing: PickerValue | null;
  weight: string;
  /** The day to sell as typed, `DD-MM-YYYY`. Empty means the day of the arrival (PRD FR-12.40). */
  sellDate: string;
}
