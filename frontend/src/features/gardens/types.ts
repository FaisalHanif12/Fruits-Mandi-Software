// Types of the باغات module (PRD 12.11, FR-12.34 to FR-12.38). What the backend owns comes from
// the generated bindings (Code Standards F2). What the user types is text (CLAUDE.md rule 1): the
// forms hold the drafts below, and the amounts the backend sends are `Paisa`, shown and never
// worked out.

import type { EntryPartInput } from '../../bindings/EntryPartInput';
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
  VenturePartnerInput,
  VenturePartnerRow,
  VentureStatus,
};

/** The types of the type list on the garden screen (PRD FR-7.67), in the order they are shown. */
export type EntryType = 'installment' | 'cost' | 'other_sale';

export const ENTRY_TYPES: readonly EntryType[] = ['installment', 'cost', 'other_sale'];

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
  qty: string;
}
