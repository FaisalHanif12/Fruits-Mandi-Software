// What the address of the sales page carries (PRD FR-9.9, plan 83): the period (one date, a month, a year or a range
// of dates, the same keys as the commission page), the party (a buyer or a seller), one item and the goods.
// `#/sales?period=year&year=2026&side=seller&party=...&goods=garden` opens the page for the year 2026 of one seller's
// garden goods. Everything is optional, and anything that does not read is left out, so `#/sales` alone opens the
// current month of every party, which is what the tile opens. The page keeps its own filters in the address, so a
// language switch, a reload and the way back from a بکری land on the same page (like `useCommissionFilters`).
// Dates are text as the backend sends them (`YYYY-MM-DD`, `YYYY-MM`, `YYYY`); there is no money here.

import { useNavigate, useSearch } from '@tanstack/react-router';
import { useCallback, useMemo } from 'react';
import type { AnalyticsGoods } from '../../bindings/AnalyticsGoods';
import { pickedOrNull } from '../../shared/screen-state';
import type { PickerValue } from '../../shared/ui';
import {
  COMMISSION_GOODS,
  COMMISSION_SIDES,
  parsePeriodSearch,
  periodFiltersOfSearch,
  searchOfPeriod,
  type CommissionSearch,
  type CommissionSide,
  type PeriodFilters,
} from './commission-search';

/** The goods filter, in the order the Owner sees it: the same four choices as the commission page (PRD FR-9.9). */
export const SALES_GOODS: readonly AnalyticsGoods[] = COMMISSION_GOODS;

/** The side of the party filter. No side is every party. */
export type SalesSide = CommissionSide;

/** The address, as it is kept: every key is optional. */
export interface SalesSearch extends Pick<
  CommissionSearch,
  'period' | 'month' | 'date' | 'year' | 'from' | 'to'
> {
  /** The side and the party. A party without a side is not read. */
  side?: SalesSide;
  party?: PickerValue;
  /** One item; none is every item. */
  item?: PickerValue;
  /** Which goods the sales are of; none is all goods. */
  goods?: AnalyticsGoods;
}

function oneOf<T extends string>(choices: readonly T[], value: unknown): T | undefined {
  return choices.find((choice) => choice === value);
}

/**
 * Reads the address. A word must be one the page knows; a date must be a real one. A party needs its side: a
 * side with no party is kept (the picker is then empty), a party with no side is dropped.
 */
export function parseSalesSearch(search: Record<string, unknown>): SalesSearch {
  const side = oneOf(COMMISSION_SIDES, search['side']);
  const party = side === undefined ? null : pickedOrNull(search['party']);
  const item = pickedOrNull(search['item']);
  const goods = oneOf(SALES_GOODS, search['goods']);
  return {
    ...parsePeriodSearch(search),
    ...(goods === undefined || goods === 'all' ? {} : { goods }),
    ...(side === undefined ? {} : { side }),
    ...(party === null ? {} : { party }),
    ...(item === null ? {} : { item }),
  };
}

/** What the page shows: the period with the values of its kind, the party, the item and the goods. */
export interface SalesFilters extends PeriodFilters {
  side: SalesSide | null;
  party: PickerValue | null;
  item: PickerValue | null;
  goods: AnalyticsGoods;
}

/** The filters the address asks for. */
export function salesFiltersOfSearch(search: SalesSearch): SalesFilters {
  return {
    ...periodFiltersOfSearch(search),
    side: search.side ?? null,
    party: search.party ?? null,
    item: search.item ?? null,
    goods: search.goods ?? 'all',
  };
}

/** The address for the filters: the period, the party with his side, the item and the goods. */
export function searchOfSalesFilters(filters: SalesFilters): SalesSearch {
  return {
    ...searchOfPeriod(filters),
    ...(filters.goods === 'all' ? {} : { goods: filters.goods }),
    ...(filters.side === null ? {} : { side: filters.side }),
    ...(filters.side !== null && filters.party !== null ? { party: filters.party } : {}),
    ...(filters.item === null ? {} : { item: filters.item }),
  };
}

/** True when the filters are the page the tile opens: the current month of every party, item and kind of goods. */
export function isPlainSales(filters: SalesFilters): boolean {
  return (
    filters.kind === 'month' &&
    filters.side === null &&
    filters.item === null &&
    filters.goods === 'all'
  );
}

/**
 * The filters of the page, kept in its address, and the way to change some of them. The new address replaces the
 * entry in the history, so Back leaves the page and does not step through periods.
 */
export function useSalesFilters(): readonly [
  SalesFilters,
  (change: Partial<SalesFilters>) => void,
] {
  const navigate = useNavigate();
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const filters = useMemo(() => salesFiltersOfSearch(parseSalesSearch(raw)), [raw]);
  const change = useCallback(
    (part: Partial<SalesFilters>) => {
      const next: SalesFilters = { ...filters, ...part };
      void navigate({
        to: '.',
        // A plain record, like `useCommissionFilters`: the route types of the other screens share some key names.
        search: (): Record<string, unknown> => ({ ...searchOfSalesFilters(next) }),
        replace: true,
      });
    },
    [filters, navigate],
  );
  return [filters, change];
}
