// The sales page as hooks (PRD FR-9.9, Code Standards F6, plan 83). The period and the filters come from the address
// (`sales-search.ts`); `salesInput` turns them into what the backend is asked, with the days of "the latest" worked
// out from the backend's today. The last answer stays on screen while the next one is read, so the page never
// flashes empty between two periods. A save changes the figures, and every save asks all queries to read again.

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { dashboardSalesDay, dashboardSalesPeriod } from './api';
import { commissionPeriod } from './commission-hooks';
import type { SalesPeriodInput } from '../../bindings/SalesPeriodInput';
import type { SalesFilters } from './sales-search';

/** What both reads are asked for besides the period or the day: one party, one item and the goods. */
export type SalesFilterInput = Pick<SalesPeriodInput, 'party' | 'item_id' | 'goods'>;

/** One page of the بکری behind a day: more than this is told as a count (a day rarely has so many). */
export const SALES_DAY_ROWS = 100;

/**
 * The party, the item and the goods as the backend reads them: a filter left at its start (every party, every item,
 * all goods) is asked as nothing, so the plain page sends the plain request. A side with nobody picked has no party:
 * the page asks for him instead of reading (`salesNeedsParty`), so no read with a side and no party is ever sent.
 */
export function salesFilterInput(filters: SalesFilters): SalesFilterInput {
  return {
    ...(filters.side !== null && filters.party !== null
      ? { party: { side: filters.side, account_id: filters.party.id } }
      : {}),
    ...(filters.item === null ? {} : { item_id: filters.item.id }),
    ...(filters.goods === 'all' ? {} : { goods: filters.goods }),
  };
}

/** True when a side is chosen and nobody of it is: the page has nothing to read yet (PRD FR-9.9). */
export function salesNeedsParty(filters: SalesFilters): boolean {
  return filters.side !== null && filters.party === null;
}

/**
 * What the backend is asked for the filters and the page, or null while it cannot be asked yet (today is not
 * known). Only a range is paged; the other kinds hold few rows and are read whole.
 */
export function salesInput(
  filters: SalesFilters,
  today: string | null,
  page: { offset: number; limit: number } | null,
): SalesPeriodInput | null {
  const period = commissionPeriod(filters, today);
  if (period === null) return null;
  return { period, ...salesFilterInput(filters), ...(page ?? {}) };
}

export function useSalesRead(input: SalesPeriodInput | null) {
  return useQuery({
    queryKey: ['dashboard', 'sales_period', input],
    queryFn: () => {
      if (input === null) throw new Error('sales read without a period');
      return dashboardSalesPeriod(input);
    },
    enabled: input !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** The بکری of one day, read when its row is opened (`date` is `YYYY-MM-DD`), in the filters of the page. */
export function useSalesDay(date: string, filter: SalesFilterInput, enabled: boolean) {
  return useQuery({
    queryKey: ['dashboard', 'sales_day', date, filter],
    queryFn: () => dashboardSalesDay({ date, ...filter, offset: 0, limit: SALES_DAY_ROWS }),
    enabled,
    staleTime: 0,
  });
}
