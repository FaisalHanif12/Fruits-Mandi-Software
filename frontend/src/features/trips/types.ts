// Types of the مارکیٹ سفر module (PRD 12.10). What the backend owns comes from the generated
// bindings (Code Standards F2). What the user types is text (CLAUDE.md rule 1): the forms hold
// the drafts below, and the amounts the backend sends are `Paisa`, shown and never worked out.

import type { EntryPartInput } from '../../bindings/EntryPartInput';
import type { MoneySource } from '../../bindings/MoneySource';
import type { OtherSaleInput } from '../../bindings/OtherSaleInput';
import type { OtherSaleNumbers } from '../../bindings/OtherSaleNumbers';
import type { OtherSalePreview } from '../../bindings/OtherSalePreview';
import type { OtherSaleSaved } from '../../bindings/OtherSaleSaved';
import type { TripCostInput } from '../../bindings/TripCostInput';
import type { TripCostNumbers } from '../../bindings/TripCostNumbers';
import type { TripCostPreview } from '../../bindings/TripCostPreview';
import type { TripCostSaved } from '../../bindings/TripCostSaved';
import type { TripDefaults } from '../../bindings/TripDefaults';
import type { TripDetail } from '../../bindings/TripDetail';
import type { TripEntryCancelInput } from '../../bindings/TripEntryCancelInput';
import type { TripEntryCancelled } from '../../bindings/TripEntryCancelled';
import type { TripEntryKind } from '../../bindings/TripEntryKind';
import type { TripEntryRow } from '../../bindings/TripEntryRow';
import type { TripInput } from '../../bindings/TripInput';
import type { TripPartnerResult } from '../../bindings/TripPartnerResult';
import type { TripRow } from '../../bindings/TripRow';
import type { TripsList } from '../../bindings/TripsList';
import type { TripsListInput } from '../../bindings/TripsListInput';
import type { TripsSummary } from '../../bindings/TripsSummary';
import type { VenturePartnerInput } from '../../bindings/VenturePartnerInput';
import type { VenturePartnerRow } from '../../bindings/VenturePartnerRow';
import type { VentureStatus } from '../../bindings/VentureStatus';
import type { PurchaseFields } from '../purchases';

export type {
  EntryPartInput,
  MoneySource,
  OtherSaleInput,
  OtherSaleNumbers,
  OtherSalePreview,
  OtherSaleSaved,
  TripCostInput,
  TripCostNumbers,
  TripCostPreview,
  TripCostSaved,
  TripDefaults,
  TripDetail,
  TripEntryCancelInput,
  TripEntryCancelled,
  TripEntryKind,
  TripEntryRow,
  TripInput,
  TripPartnerResult,
  TripRow,
  TripsList,
  TripsListInput,
  TripsSummary,
  VenturePartnerInput,
  VenturePartnerRow,
  VentureStatus,
};

/** The types of the type list on the trip screen (PRD FR-7.67), in the order they are shown. */
export type EntryType = 'purchase' | 'cost' | 'other_sale';

export const ENTRY_TYPES: readonly EntryType[] = ['purchase', 'cost', 'other_sale'];

/** The new trip form as text (PRD FR-12.29): the shares first, then the sources, then the details. */
export interface TripDraft {
  /** One row for every active partner, in the order the partners were created. */
  partners: TripDraftPartner[];
  /** `DD-MM-YYYY` as typed. */
  date: string;
  market: string;
  sentPerson: string;
  note: string;
}

export interface TripDraftPartner {
  partner: VenturePartnerRow['partner'];
  /** The % as typed, for example "40" or "33.33". Empty means he is not in the trip. */
  share: string;
  source: MoneySource;
}

/** The fields only a trip cost has. */
export interface CostFields {
  reason: string;
}

/** The fields only a sale in another shop has. */
export interface OtherSaleFields {
  shop: string;
  /** The id of the purchase the goods come from, as text, or empty. */
  purchaseId: string;
  qty: string;
}

/**
 * Everything typed on the trip screen's entry form. The day, the amount and the note are the
 * same on every type, so changing the type keeps them and clears the rest (PRD FR-7.67). The
 * sources of the partners are shared by the two payments (a purchase and a cost).
 */
export interface EntryDraft {
  type: EntryType;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** The amount of a cost, or the money received for a sale in another shop. */
  amount: string;
  note: string;
  /** This entry's choice of source by partner id. A missing one means the trip's source. */
  sources: Record<number, MoneySource>;
  purchase: PurchaseFields;
  cost: CostFields;
  sale: OtherSaleFields;
}
