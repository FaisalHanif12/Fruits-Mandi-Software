// What the three load buttons of the بکری screen have to list (PRD FR-12.50, plan 78 C1): the counts of lots with
// goods left, trip goods and garden goods for our shop, and the rows of the card of one kind. The backend counts and
// lists them; the screen shows a button only when its count is above 0 and counts nothing itself. The answers are
// read again after anything that changes stock (a save, an arrival, a write-off): every such screen reads again the
// queries that start with `bikri`.

import { useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import { usePagedQuery } from '../../../shared/hooks';
import type { GoodsSources } from '../../../bindings/GoodsSources';
import type { VentureKind } from '../../../bindings/VentureKind';
import { bikriGoodsSources, bikriGoodsToLoad } from '../api';
import { readyStockOf, type ReadyStock } from '../ready-stock';

/** The counts, or undefined while they are not known (no button is shown until then). */
export function useGoodsSources(enabled: boolean): GoodsSources | undefined {
  const answer = useQuery({
    queryKey: ['bikri', 'goods-sources'],
    queryFn: bikriGoodsSources,
    enabled,
    // The goods left change with every sale: never keep an old answer for a new sale.
    staleTime: 0,
    gcTime: 0,
  });
  return answer.data;
}

/**
 * The rows of the card of one kind, a page at a time (PRD FR-12.50, FR-19.11, plan 80 R2-5), or undefined while they
 * are not known. Nothing is asked while the card is shut. `pager` is the backend's count of the whole card, so a card
 * of more than 100 rows is reached with the page buttons and nothing is left out.
 */
export function useGoodsToLoad(kind: VentureKind, enabled: boolean) {
  const list = usePagedQuery({
    screen: `bikri.goods_to_load.${kind}`,
    queryKey: ['bikri', 'goods-to-load', kind],
    fetchPage: (offset, limit) => bikriGoodsToLoad(kind, offset, limit),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    enabled,
  });
  const page = list.data?.pages[0];
  const rows: ReadyStock[] | undefined = useMemo(
    () => (page === undefined ? undefined : readyStockOf(page)),
    [page],
  );
  return { query: list, rows, pager: list.pager };
}
