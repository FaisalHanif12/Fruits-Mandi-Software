// "تقسیم": splits the header's total freight over the lots by their نگ, leftover to the lot with
// the most نگ (PRD C17, FR-12.2). The backend does the splitting; the screen puts each part, as
// the text a user would type, in the freight cell of its lot, where it can still be changed.

import { useMutation } from '@tanstack/react-query';
import { bikriSplitFreight } from '../api';
import { plainRupees } from '../draft';
import type { BikriDraftStores } from '../draft-store';
import { LOT } from '../types';

interface SplitRequest {
  stores: BikriDraftStores;
  totalFreight: string;
}

export function useSplitFreight() {
  return useMutation({
    mutationFn: async ({ stores, totalFreight }: SplitRequest) => {
      const rows = stores.lots.getState().rows;
      const parts = await bikriSplitFreight(
        totalFreight.trim(),
        rows.map((row) => ({ key: row.rowId, qty_received: (row.typed[LOT.qty] ?? '').trim() })),
      );
      for (const part of parts) {
        stores.lots.getState().updateTyped(part.key, LOT.freight, plainRupees(part.freight_paisa));
      }
      return parts;
    },
  });
}
