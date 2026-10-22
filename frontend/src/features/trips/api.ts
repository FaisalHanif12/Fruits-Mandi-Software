// Typed wrappers around the مارکیٹ سفر commands (plan 16b; module `trips`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.
// Every command takes one `{ input }` object, except the two that take nothing.

import { callCommand } from '../../shared/api';
import type { TripGetInput } from '../../bindings/TripGetInput';
import type { TripSetPartnersInput } from '../../bindings/TripSetPartnersInput';
import type {
  OtherSaleInput,
  OtherSalePreview,
  OtherSaleSaved,
  TripCostInput,
  TripCostPreview,
  TripCostSaved,
  TripDefaults,
  TripDetail,
  TripEntryCancelInput,
  TripEntryCancelled,
  TripInput,
  TripRow,
  TripsList,
  TripsListInput,
  TripsSummary,
} from './types';

/** The form of a new trip, filled from the last trip (PRD A56). */
export function tripsDefaults(): Promise<TripDefaults> {
  return callCommand<TripDefaults>('trips_defaults');
}

/** Makes a trip: the shares total 100% and each partner has a source (PRD FR-12.29). */
export function tripsCreate(input: TripInput): Promise<TripRow> {
  return callCommand<TripRow>('trips_create', { input });
}

/** New shares for a trip that has no entry yet. */
export function tripsSetPartners(input: TripSetPartnersInput): Promise<TripRow> {
  return callCommand<TripRow>('trips_set_partners', { input });
}

/** One page of trips and the totals over the whole filter (PRD FR-19.11). */
export function tripsList(input: TripsListInput): Promise<TripsList> {
  return callCommand<TripsList>('trips_list', { input });
}

/** One trip with its purchases, one page of its history and each partner's money. */
export function tripsGet(input: TripGetInput): Promise<TripDetail> {
  return callCommand<TripDetail>('trips_get', { input });
}

/** The data of the dashboard tile مارکیٹ سفر (PRD 9.1). */
export function tripsSummary(): Promise<TripsSummary> {
  return callCommand<TripsSummary>('trips_summary');
}

/** Works out a trip cost as typed, with each partner's part and how it is spread (C23, C24). */
export function tripsCostPreview(input: TripCostInput): Promise<TripCostPreview> {
  return callCommand<TripCostPreview>('trips_cost_preview', { input });
}

export function tripsCostSave(input: TripCostInput): Promise<TripCostSaved> {
  return callCommand<TripCostSaved>('trips_cost_save', { input });
}

export function tripsCostCancel(input: TripEntryCancelInput): Promise<TripEntryCancelled> {
  return callCommand<TripEntryCancelled>('trips_cost_cancel', { input });
}

/** Works out a sale in another shop as typed: the cost, the result and each partner's part (C25). */
export function tripsOtherSalePreview(input: OtherSaleInput): Promise<OtherSalePreview> {
  return callCommand<OtherSalePreview>('trips_other_sale_preview', { input });
}

export function tripsOtherSaleSave(input: OtherSaleInput): Promise<OtherSaleSaved> {
  return callCommand<OtherSaleSaved>('trips_other_sale_save', { input });
}

export function tripsOtherSaleCancel(input: TripEntryCancelInput): Promise<TripEntryCancelled> {
  return callCommand<TripEntryCancelled>('trips_other_sale_cancel', { input });
}
