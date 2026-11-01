// What the address of an analytics view can carry (plan 49, PRD FR-7.74): `#/analytics/A?item=5&period=
// this_month&compare=previous` opens view A with those filters already in the boxes, the way a button
// under an answer of the question panel does. Everything is optional, and anything that does not read
// is left out. With no extra parameter the view opens as it always did (the filters it remembers).
//
// A record (item, seller, buyer, city, اڈا) is its whole picker value or only its id; a bare id has no
// names yet, and the view's first answer names it (the page already does that for a saved view).

import { pickedOrNull } from '../../shared/screen-state';
import type { PickerValue } from '../../shared/ui';
import {
  COMPARE_CHOICES,
  EMPTY_ANALYTICS_FILTERS,
  GOODS_CHOICES,
  GROUP_BY_CHOICES,
  PERIOD_CHOICES,
  SIDE_CHOICES,
  TRIP_VIEW_CHOICES,
  type AnalyticsFilterState,
  type PeriodChoice,
} from './filters';
import type {
  AnalyticsCompare,
  AnalyticsGoods,
  AnalyticsGroupBy,
  AnalyticsSide,
  AnalyticsTripView,
} from './types';

export interface AnalyticsSearch {
  /** A period word of PRD 18.1 (never `custom`: the dates below are that). */
  period?: Exclude<PeriodChoice, 'custom'>;
  /** `DD-MM-YYYY` as typed (the backend also reads `10` and `10-9`). */
  from?: string;
  to?: string;
  group_by?: AnalyticsGroupBy;
  item?: PickerValue;
  seller?: PickerValue;
  buyer?: PickerValue;
  city?: PickerValue;
  adda?: PickerValue;
  side?: AnalyticsSide;
  compare?: AnalyticsCompare;
  goods?: AnalyticsGoods;
  trip_view?: AnalyticsTripView;
  /** A partner, a trip or a garden, by id. */
  partner?: number;
  trip?: number;
  garden?: number;
}

function text(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() !== '' ? value.trim() : undefined;
}

/** A positive whole number, from a number or from the text of one (an address carries text). */
function positiveId(value: unknown): number | undefined {
  const id = typeof value === 'string' && value.trim() !== '' ? Number(value) : value;
  return typeof id === 'number' && Number.isSafeInteger(id) && id > 0 ? id : undefined;
}

/** A record of the address: a whole picker value, or only an id, which has no names yet. */
function readPicked(value: unknown): PickerValue | undefined {
  const whole = pickedOrNull(value);
  if (whole !== null) return whole;
  const id = positiveId(value);
  return id === undefined ? undefined : { id, name_ur: '', name_en: '', code: undefined };
}

function oneOf<T extends string>(choices: readonly T[], value: unknown): T | undefined {
  return choices.find((choice) => choice === value);
}

/** Reads the address. A word must be one the screen knows; an id must be a positive whole number. */
export function parseAnalyticsSearch(search: Record<string, unknown>): AnalyticsSearch {
  const period = oneOf(PERIOD_CHOICES, search['period']);
  const from = text(search['from']);
  const to = text(search['to']);
  const groupBy = oneOf(GROUP_BY_CHOICES, search['group_by']);
  const item = readPicked(search['item']);
  const seller = readPicked(search['seller']);
  const buyer = readPicked(search['buyer']);
  const city = readPicked(search['city']);
  const adda = readPicked(search['adda']);
  const side = oneOf(SIDE_CHOICES, search['side']);
  const compare = oneOf(COMPARE_CHOICES, search['compare']);
  const goods = oneOf(GOODS_CHOICES, search['goods']);
  const tripView = oneOf(TRIP_VIEW_CHOICES, search['trip_view']);
  const partner = positiveId(search['partner']);
  const trip = positiveId(search['trip']);
  const garden = positiveId(search['garden']);
  return {
    ...(period === undefined || period === 'custom' ? {} : { period }),
    ...(from === undefined ? {} : { from }),
    ...(to === undefined ? {} : { to }),
    ...(groupBy === undefined ? {} : { group_by: groupBy }),
    ...(item === undefined ? {} : { item }),
    ...(seller === undefined ? {} : { seller }),
    ...(buyer === undefined ? {} : { buyer }),
    ...(city === undefined ? {} : { city }),
    ...(adda === undefined ? {} : { adda }),
    ...(side === undefined ? {} : { side }),
    ...(compare === undefined ? {} : { compare }),
    ...(goods === undefined ? {} : { goods }),
    ...(tripView === undefined ? {} : { trip_view: tripView }),
    ...(partner === undefined ? {} : { partner }),
    ...(trip === undefined ? {} : { trip }),
    ...(garden === undefined ? {} : { garden }),
  };
}

/** True when the address names at least one filter: only then does it replace the remembered ones. */
export function hasAnalyticsSearch(search: AnalyticsSearch): boolean {
  return Object.keys(search).length > 0;
}

/** The filter boxes the address starts with. A box it does not name starts empty. */
export function filtersFromSearch(search: AnalyticsSearch): AnalyticsFilterState {
  return {
    ...EMPTY_ANALYTICS_FILTERS,
    period: search.period ?? 'custom',
    from: search.from ?? '',
    to: search.to ?? '',
    groupBy: search.group_by ?? 'month',
    item: search.item ?? null,
    seller: search.seller ?? null,
    buyer: search.buyer ?? null,
    city: search.city ?? null,
    adda: search.adda ?? null,
    side: search.side ?? 'both',
    compare: search.compare ?? 'off',
    goods: search.goods ?? 'all',
    tripView: search.trip_view ?? 'trip',
    partner: search.partner ?? null,
    trip: search.trip ?? null,
    garden: search.garden ?? null,
  };
}
