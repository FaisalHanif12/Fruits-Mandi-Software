// The dues pages as hooks (PRD FR-9.4, FR-9.6, FR-9.7, FR-9.8, FR-19.11, Code Standards F6): one page at a
// time, with the count and the totals of the whole filter, sorted by the backend (the default is by name). A
// save changes them, and every save asks all queries to read again.

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import type { SellerDuesSide } from '../../bindings/SellerDuesSide';
import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import {
  dashboardAdvancesOpen,
  dashboardBuyerDues,
  dashboardCashSalesDay,
  dashboardCashSalesMonth,
  dashboardDeadAmount,
  dashboardLentOpen,
  dashboardSellerDues,
  dashboardTotalDues,
  dashboardTotalWorth,
} from './api';
import type { TotalWorthSide } from '../../bindings/TotalWorthSide';

/** A short pause so a burst of key presses in the search box asks once (Code Standards F35). */
const FILTER_DELAY_MS = 150;

/** One page of the buyers behind a day: a day rarely has more, and the count says when it does. */
const DAY_BUYERS = 100;

/** One page of the buyers of the total dues page and of the buyer dues page. `partyId` null is سب (PRD FR-19.1). */
export function useBuyerDues(filter: { search: string; partyId: number | null }) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'dashboard.buyer_dues',
    queryKey: ['dashboard', 'buyer_dues', settled],
    fetchPage: (offset, limit, sort) =>
      dashboardBuyerDues({
        party_id: settled.partyId,
        search: settled.search === '' ? null : settled.search,
        sort,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/**
 * One page of the sellers (PRD FR-9.6). `partyId` null is سب. The side totals are over the party, not cut by
 * side, so they stay put when the side changes. `screen` names the list for its page size: the seller page and
 * the sellers of the total dues page remember their own.
 */
export function useSellerDues(
  filter: { side: SellerDuesSide; partyId: number | null },
  screen = 'dashboard.seller_dues',
) {
  return usePagedQuery({
    screen,
    queryKey: ['dashboard', 'seller_dues', filter],
    fetchPage: (offset, limit, sort) =>
      dashboardSellerDues({
        side: filter.side,
        party_id: filter.partyId,
        search: null,
        sort,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** One page of the people who borrowed and still owe (PRD FR-9.1, FR-16.5). */
export function useLentOpen(screen = 'dashboard.lent_open') {
  return usePagedQuery({
    screen,
    queryKey: ['dashboard', 'lent_open'],
    fetchPage: (offset, limit, sort) => dashboardLentOpen({ search: null, sort, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** One page of the people in the dead amount who still owe, for the dead section of کل اگراہی (PRD FR-9.7). */
export function useDeadOwed() {
  return usePagedQuery({
    screen: 'dashboard.dead_owed',
    queryKey: ['dashboard', 'dead_owed'],
    fetchPage: (offset, limit) =>
      dashboardDeadAmount({ filter: 'owed', search: '', offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.row_count,
    keepPrevious: true,
  });
}

/** One page of the sellers with an open advance (PRD FR-9.1, FR-15.8). The day is today: an empty as-of day. */
export function useAdvancesOpen() {
  return usePagedQuery({
    screen: 'dashboard.advances_open',
    queryKey: ['dashboard', 'advances_open'],
    fetchPage: (offset, limit) =>
      dashboardAdvancesOpen({
        seller_id: null,
        as_of: '',
        search: null,
        only_open: true,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** The four parts of کل اگراہی and the grand total, in one read (PRD FR-9.7). */
export function useTotalDues() {
  return useQuery({
    queryKey: ['dashboard', 'total_dues'],
    queryFn: dashboardTotalDues,
    staleTime: 0,
  });
}

/** One month of نقدی (PRD FR-9.8). `month` null is the current month; the last answer stays while the next is read. */
export function useCashSalesMonth(month: string | null) {
  return useQuery({
    queryKey: ['dashboard', 'cash_sales_month', month],
    queryFn: () => dashboardCashSalesMonth({ month }),
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** The buyers of one day of نقدی, read when its row is opened (`date` is `YYYY-MM-DD`). */
export function useCashSalesDay(date: string, enabled: boolean) {
  return useQuery({
    queryKey: ['dashboard', 'cash_sales_day', date],
    queryFn: () => dashboardCashSalesDay({ date, offset: 0, limit: DAY_BUYERS }),
    enabled,
    staleTime: 0,
  });
}

/**
 * One page of one side of کل مالیت (PRD FR-9.10): the money to have or the money to give. Every page carries the
 * totals of both sides and of each source, from one read of the books.
 */
export function useTotalWorth(side: TotalWorthSide) {
  return usePagedQuery({
    screen: `dashboard.total_worth.${side}`,
    queryKey: ['dashboard', 'total_worth', side],
    fetchPage: (offset, limit) => dashboardTotalWorth({ side, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
