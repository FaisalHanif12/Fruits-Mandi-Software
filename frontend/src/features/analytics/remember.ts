// What the analytics area remembers for the Owner (PRD FR-7.45): the filters he last used, under the
// screen name `analytics.filters`. The area has one set of filters that stays while he moves from
// one view to another, so one value is kept for all of them. A value that cannot be read any more is
// left out and the box starts empty. The one-time person is not kept. Nothing about money is kept.

import { choiceOr, isRecord, pickedOrNull, pickedText } from '../../shared/screen-state';
import {
  COMPARE_CHOICES,
  EMPTY_ANALYTICS_FILTERS,
  GROUP_BY_CHOICES,
  PERIOD_CHOICES,
  SIDE_CHOICES,
  type AnalyticsFilterState,
} from './filters';

/** The name under which the analytics filters are kept. */
export const ANALYTICS_SCREEN = 'analytics.filters';

function dateText(value: unknown): string {
  return typeof value === 'string' && value.length <= 20 ? value : '';
}

/** The filters to keep, or null when nothing is chosen. */
export function rememberedAnalytics(state: AnalyticsFilterState): Record<string, unknown> | null {
  const base = EMPTY_ANALYTICS_FILTERS;
  const kept: Record<string, unknown> = {};
  if (state.period !== base.period) kept['period'] = state.period;
  if (state.from !== '') kept['from'] = state.from;
  if (state.to !== '') kept['to'] = state.to;
  if (state.groupBy !== base.groupBy) kept['groupBy'] = state.groupBy;
  for (const key of ['item', 'seller', 'buyer', 'city', 'adda'] as const) {
    const value = state[key];
    if (value !== null) kept[key] = pickedText(value);
  }
  if (state.side !== base.side) kept['side'] = state.side;
  if (state.compare !== base.compare) kept['compare'] = state.compare;
  return Object.keys(kept).length === 0 ? null : kept;
}

/** The filters a text kept, or null when it holds nothing this area can read. */
export function analyticsFromRemembered(raw: unknown): AnalyticsFilterState | null {
  if (!isRecord(raw)) return null;
  const base = EMPTY_ANALYTICS_FILTERS;
  return {
    ...base,
    period: choiceOr(raw['period'], PERIOD_CHOICES, base.period),
    from: dateText(raw['from']),
    to: dateText(raw['to']),
    groupBy: choiceOr(raw['groupBy'], GROUP_BY_CHOICES, base.groupBy),
    item: pickedOrNull(raw['item']),
    seller: pickedOrNull(raw['seller']),
    buyer: pickedOrNull(raw['buyer']),
    city: pickedOrNull(raw['city']),
    adda: pickedOrNull(raw['adda']),
    side: choiceOr(raw['side'], SIDE_CHOICES, base.side),
    compare: choiceOr(raw['compare'], COMPARE_CHOICES, base.compare),
  };
}
