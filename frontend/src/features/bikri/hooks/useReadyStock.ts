// The stock that is ready for a new بکری (PRD FR-12.39, FR-12.40): the one place that knows which
// backend command answers. `bikri_ready_goods` lists, for the day of the بکری, every trip purchase
// and every garden goods arrived entry with goods left whose date to sell is on or before that day,
// oldest first. The answer is asked again after anything that changes stock (a save, an arrival, a
// write-off): every such screen reads again the queries that start with `bikri`.

import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import { isoToDisplayDate } from '../../../shared/format';
import { bikriReadyGoods } from '../api';
import type { ReadyStock } from '../ready-stock';

/** The query key of the list for a day. The day is `YYYY-MM-DD`. */
export const readyGoodsKey = (iso: string | null) => ['bikri', 'ready-goods', iso] as const;

/** The ready stock of a day, `YYYY-MM-DD`, or undefined while it is not known. Null day: nothing is asked. */
export function useReadyStock(iso: string | null): ReadyStock[] | undefined {
  const answer = useQuery({
    queryKey: readyGoodsKey(iso),
    queryFn: () => bikriReadyGoods(isoToDisplayDate(iso ?? '')),
    enabled: iso !== null,
    // The goods left change with every sale: never keep an old answer for a new sale.
    staleTime: 0,
    gcTime: 0,
  });
  const rows = answer.data?.rows;
  return useMemo(
    () =>
      rows?.flatMap((row): ReadyStock[] => {
        // The backend lists goods left. The screen still checks it, so a wrong answer never makes
        // a lot: a trip row has its trip, a garden row its garden.
        if (row.qty_left <= 0) return [];
        const base = {
          id: row.id,
          number: row.number,
          sellDate: row.sell_date,
          item: row.item,
          packing: row.packing,
          qtyLeft: row.qty_left,
        };
        if (row.kind === 'trip' && row.trip !== null) {
          return [{ ...base, kind: 'trip', garden: null }];
        }
        if (row.kind === 'garden' && row.garden !== null) {
          return [{ ...base, kind: 'garden', garden: row.garden }];
        }
        return [];
      }),
    [rows],
  );
}
