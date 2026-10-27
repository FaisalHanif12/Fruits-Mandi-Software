// The stock that is ready for a new بکری (PRD FR-12.39): the one place that knows which backend
// command answers. Today it is the purchases list, asked for the purchases bought on or before the
// day of the بکری that still have goods left; a purchase's date to sell starts as its day. When the
// backend's list of ready stock for both kinds is there (plan 33 v1.1), only this file changes.

import { useMemo } from 'react';
import { usePurchasesReadyBy } from '../../purchases';
import type { ReadyStock } from '../ready-stock';

/** The ready stock of a day, `YYYY-MM-DD`, or undefined while it is not known. Null day: nothing is asked. */
export function useReadyStock(iso: string | null): ReadyStock[] | undefined {
  const purchases = usePurchasesReadyBy(iso);
  const rows = purchases.data?.rows;
  return useMemo(
    () =>
      rows?.flatMap((purchase) =>
        // The backend lists saved purchases with goods left. The screen still checks it, so a wrong
        // answer never makes a lot.
        purchase.status === 'saved' && purchase.qty_left > 0
          ? [
              {
                kind: 'trip' as const,
                id: purchase.id,
                number: purchase.number,
                sellDate: purchase.entry_date,
                item: purchase.item,
                packing: purchase.packing,
                qtyLeft: purchase.qty_left,
              },
            ]
          : [],
      ),
    [rows],
  );
}
