// Opening a report from outside the report area (PRD FR-7.65, FR-5.4): the party card's buttons and
// the global search. Both live in shared code and the app frame, which can be drawn where no router
// exists (a component test, a gallery page); there the function is null and nothing shows.

import { useRouter } from '@tanstack/react-router';
import { useMemo } from 'react';
import type { ReportId } from '../../bindings/ReportId';
import type { ReportSearch } from './search';

/** Opens a report with some of its filters already in the address. */
export type OpenReport = (id: ReportId, search?: ReportSearch) => void;

/**
 * What the party card's buttons ask for (PRD FR-7.65): the account, and the whole history. The
 * range is "already filled" in the sense that nothing has to be typed: an empty `from` is the first
 * day of data (R14: "from the first date of data"), an empty `to` the last. The report line under
 * the filters says "All dates". Change the range here, in one place.
 */
export function partyReportSearch(accountId: number): ReportSearch {
  return { party: accountId };
}

/** The function that opens a report, or null when no router is around. */
export function useOpenReport(): OpenReport | null {
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  return useMemo<OpenReport | null>(
    () =>
      router === undefined
        ? null
        : (id, search = {}) => {
            void router.navigate({ to: '/reports/$id', params: { id }, search });
          },
    [router],
  );
}
