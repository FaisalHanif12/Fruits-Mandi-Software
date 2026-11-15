// The commission page as hooks (PRD FR-9.3, Code Standards F6, plan 77). The period and the party come from
// the address (`commission-search.ts`); `commissionInput` turns them into what the backend is asked, with the
// days of "the latest" worked out from the backend's today. The last answer stays on screen while the next
// one is read, so the page never flashes empty between two periods. A save changes the figures, and every save
// asks all queries to read again.

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import type { CommissionMonthInput } from '../../bindings/CommissionMonthInput';
import type { CommissionParty } from '../../bindings/CommissionParty';
import type { CommissionPeriod } from '../../bindings/CommissionPeriod';
import { monthBounds, monthOf } from '../../shared/format';
import { dashboardCommissionDay, dashboardCommissionMonth } from './api';
import { yearOf, type CommissionFilters } from './commission-search';

/** One page of the postings behind a day: more than this is told as a count (a day rarely has so many). */
const DAY_ENTRIES = 100;

/** The party of the filters as the backend reads it, or null when no party is picked. */
export function commissionParty(filters: CommissionFilters): CommissionParty | null {
  if (filters.side === null || filters.party === null) return null;
  return {
    side: filters.side,
    account_id: filters.party.id,
    ...(filters.person === null ? {} : { chukta_person_key: filters.person }),
  };
}

/**
 * The period as the backend reads it. A kind with nothing chosen is the latest of its kind, which needs the
 * backend's today (the month is the one that can go without it: none is the current month). Null while
 * today is not known.
 */
export function commissionPeriod(
  filters: CommissionFilters,
  today: string | null,
): CommissionPeriod | null {
  switch (filters.kind) {
    case 'month':
      return { kind: 'month', ...(filters.month === null ? {} : { month: filters.month }) };
    case 'day':
      return today === null ? null : { kind: 'day', date: filters.day ?? today };
    case 'year':
      return today === null ? null : { kind: 'year', year: Number(filters.year ?? yearOf(today)) };
    case 'range':
      return today === null
        ? null
        : {
            kind: 'range',
            from: filters.from ?? monthBounds(monthOf(today)).from,
            to: filters.to ?? today,
          };
  }
}

/**
 * What the backend is asked for the filters and the page, or null while it cannot be asked yet. A month
 * with no party is the request of plan 76 (`{ month }` alone), so nothing changes for it.
 */
export function commissionInput(
  filters: CommissionFilters,
  today: string | null,
  page: { offset: number; limit: number } | null,
): CommissionMonthInput | null {
  const period = commissionPeriod(filters, today);
  if (period === null) return null;
  const party = commissionParty(filters);
  return {
    month: filters.kind === 'month' ? filters.month : null,
    ...(filters.kind === 'month' ? {} : { period }),
    ...(party === null ? {} : { party }),
    ...(page ?? {}),
  };
}

export function useCommissionRead(input: CommissionMonthInput | null) {
  return useQuery({
    queryKey: ['dashboard', 'commission_month', input],
    queryFn: () => {
      if (input === null) throw new Error('commission read without a period');
      return dashboardCommissionMonth(input);
    },
    enabled: input !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** The postings of one day, read when its row is opened (`day` is `YYYY-MM-DD`), of one party when there is one. */
export function useCommissionDay(day: string, party: CommissionParty | null, enabled: boolean) {
  return useQuery({
    queryKey: ['dashboard', 'commission_day', day, party],
    queryFn: () =>
      dashboardCommissionDay({
        date: day,
        offset: 0,
        limit: DAY_ENTRIES,
        ...(party === null ? {} : { party }),
      }),
    enabled,
    staleTime: 0,
  });
}
