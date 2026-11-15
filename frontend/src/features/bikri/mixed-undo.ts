// The values of a mixed save that was taken back at once, as a draft to correct and save again (PRD
// FR-7.47, plan 76 point 1). The backend gives the new lots as a draft of the usual shape, and each
// carried lot as its id and the lines it had. This puts both on the stores of a new بکری: the new lots as
// any restored draft, and each carried lot as a locked lot row (read again from the goods left, so its
// seller, item and the number of its بکری are the backend's) with its lines. The result goes through the
// same door as a draft restored at start-up.

import type { LotInput } from '../../bindings/LotInput';
import type { MixedUndoDraft } from '../../bindings/MixedUndoDraft';
import type { StockRow } from '../../bindings/StockRow';
import { bikriStockList } from './api';
import { carriedRowOf } from './carried-lots';
import { applyBackendDraft, resolveDraftRecords, saleRow } from './draft-load';
import { createBikriDraft } from './draft-store';
import { parseSnapshot, serialiseDraft } from './draft-snapshot';
import type { BikriDraft } from './types';

/** A lot with nothing but its lines: enough to find the records of the lines (the buyers). */
function shell(key: string, sales: LotInput['sales']): LotInput {
  return {
    key,
    seller_account_id: null,
    item_id: null,
    packing_id: null,
    marka_id: null,
    store_id: null,
    qty_received: '',
    weight_received: null,
    freight: null,
    freight_paid_by: 'firm',
    labour: null,
    munshiana: null,
    phone: null,
    cash_paid: null,
    store_rent: null,
    adjust_advance: true,
    sales,
  };
}

/** The lot as the list of goods left shows it, or null when it is not there (any more). */
async function stockOf(lotId: number): Promise<StockRow | null> {
  const answer = await bikriStockList({
    date: null,
    from: null,
    to: null,
    seller_id: null,
    item_id: null,
    lot_id: lotId,
    include_written_off: true,
    offset: 0,
    limit: 1,
    search: null,
    sort: null,
  });
  return answer.rows.find((row) => row.lot_id === lotId) ?? null;
}

/** The sale book as a snapshot, or null when it cannot be made. */
export async function snapshotOfMixedUndo(draft: MixedUndoDraft) {
  const stores = createBikriDraft(draft.date);
  const newDraft: BikriDraft | null = draft.new_draft;
  const stocks = await Promise.all(draft.carried.map((carried) => stockOf(carried.lot_id)));
  // The records of every line: the new lots' and the carried lots'.
  const whole: BikriDraft = {
    header: newDraft?.header ?? { entry_date: draft.date },
    lots: [
      ...(newDraft?.lots ?? []),
      ...draft.carried.map((carried) => shell(String(carried.lot_id), carried.sales)),
    ],
  };
  const records = await resolveDraftRecords(whole, { accounts: [], masters: new Map() });
  if (newDraft !== null) applyBackendDraft(stores, newDraft, records);
  // The carried lots come first, as the backend lists them.
  const carriedRows = draft.carried.flatMap((carried, index) => {
    const stock = stocks[index];
    if (stock === null || stock === undefined) return [];
    const row = carriedRowOf(stock);
    return [{ row, lines: carried.sales.map((sale) => saleRow(sale, records)) }];
  });
  if (carriedRows.length > 0) {
    const rest = newDraft === null ? [] : stores.lots.getState().rows;
    stores.lots.setState({ rows: [...carriedRows.map((one) => one.row), ...rest] });
    for (const { row, lines } of carriedRows) {
      if (lines.length > 0) stores.salesFor(row.rowId).setState({ rows: lines });
    }
    stores.view.getState().select(carriedRows[0]?.row.rowId ?? null);
  }
  return parseSnapshot(serialiseDraft(stores));
}
