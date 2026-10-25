// What a report remembers for the Owner (PRD FR-7.45): the filters he last used on it. They are kept
// as text under the screen name `report.<id>` (for example `report.r20`), through the shared screen
// memory. A remembered filter is a convenience: a value that cannot be read any more (an old shape,
// a record that is gone) is left out and the box starts empty. The one-time person is not kept: he is
// picked from what the account has now. Nothing about money is kept (CLAUDE.md rule 1).

import type { ReportId } from '../../bindings/ReportId';
import {
  choiceOr,
  flagOf,
  idOrNull,
  isRecord,
  pickedOrNull,
  pickedText,
} from '../../shared/screen-state';
import {
  EMPTY_FILTERS,
  GARDEN_VIEW_CHOICES,
  GROUP_BY_CHOICES,
  PAID_CHOICES,
  PERIOD_CHOICES,
  TRIP_VIEW_CHOICES,
  VOUCHER_CHOICES,
  type ReportFilterState,
} from './filters';

/** The screen name of a report: `report.r20` for R20. */
export function reportScreen(id: ReportId): string {
  return `report.${id.toLowerCase()}`;
}

function dateText(value: unknown): string {
  return typeof value === 'string' && value.length <= 20 ? value : '';
}

/**
 * The filters to keep, or null when nothing is chosen (every box empty and the layout the first
 * one), so a cleared report forgets what it kept.
 */
export function rememberedFilters(state: ReportFilterState): Record<string, unknown> | null {
  const base = EMPTY_FILTERS;
  const kept: Record<string, unknown> = {};
  if (state.period !== base.period) kept['period'] = state.period;
  if (state.from !== '') kept['from'] = state.from;
  if (state.to !== '') kept['to'] = state.to;
  if (state.asOf !== '') kept['asOf'] = state.asOf;
  for (const key of ['party', 'seller', 'zamindar', 'item', 'city', 'adda'] as const) {
    const value = state[key];
    if (value !== null) kept[key] = pickedText(value);
  }
  if (state.detail) kept['detail'] = true;
  if (state.paid !== base.paid) kept['paid'] = state.paid;
  if (state.groupBy !== base.groupBy) kept['groupBy'] = state.groupBy;
  if (state.partner !== null) kept['partner'] = state.partner;
  if (state.guestsOnly) kept['guestsOnly'] = true;
  if (state.voucherType !== base.voucherType) kept['voucherType'] = state.voucherType;
  if (state.tripView !== base.tripView) kept['tripView'] = state.tripView;
  if (state.garden !== null) kept['garden'] = state.garden;
  if (state.gardenView !== base.gardenView) kept['gardenView'] = state.gardenView;
  return Object.keys(kept).length === 0 ? null : kept;
}

/** The filters a text kept, or null when it holds nothing this screen can read. */
export function filtersFromRemembered(raw: unknown): ReportFilterState | null {
  if (!isRecord(raw)) return null;
  const base = EMPTY_FILTERS;
  return {
    ...base,
    period: choiceOr(raw['period'], PERIOD_CHOICES, base.period),
    from: dateText(raw['from']),
    to: dateText(raw['to']),
    asOf: dateText(raw['asOf']),
    party: pickedOrNull(raw['party']),
    seller: pickedOrNull(raw['seller']),
    zamindar: pickedOrNull(raw['zamindar']),
    item: pickedOrNull(raw['item']),
    city: pickedOrNull(raw['city']),
    adda: pickedOrNull(raw['adda']),
    detail: flagOf(raw['detail']),
    paid: choiceOr(raw['paid'], PAID_CHOICES, base.paid),
    groupBy: choiceOr(raw['groupBy'], GROUP_BY_CHOICES, base.groupBy),
    partner: idOrNull(raw['partner']),
    guestsOnly: flagOf(raw['guestsOnly']),
    voucherType: choiceOr(raw['voucherType'], VOUCHER_CHOICES, base.voucherType),
    tripView: choiceOr(raw['tripView'], TRIP_VIEW_CHOICES, base.tripView),
    garden: idOrNull(raw['garden']),
    gardenView: choiceOr(raw['gardenView'], GARDEN_VIEW_CHOICES, base.gardenView),
  };
}
