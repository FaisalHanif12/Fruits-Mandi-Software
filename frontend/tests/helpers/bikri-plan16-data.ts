// Test data for plan 16 (goods left, later sale, edit, cancel, undo): canned answers of the new
// commands, in the exact shapes of the generated types. They are a test double built from the
// backend's own Event 1 files (the request draft with the keys and ids an edit carries); the
// numbers are the PRD's (E2E-1 to E2E-4, E2E-16). The screens only show them. This file imports
// nothing from Vitest, so the development page can use it too (dev-bridge.ts).

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { CancelPreview } from '../../src/bindings/CancelPreview';
import type { CashCheck } from '../../src/bindings/CashCheck';
import type { EditableBikri } from '../../src/bindings/EditableBikri';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { SaleInput } from '../../src/bindings/SaleInput';
import type { StockList } from '../../src/bindings/StockList';
import type { StockRow } from '../../src/bindings/StockRow';
import type { UndoResult } from '../../src/bindings/UndoResult';
import { EVENT1_REQUEST, IDS, previewFor } from './bikri-data';
import { savedEvent1 } from './bikri-saved';
import { paisaForTest } from './paisa';

const rupees = (value: number) => paisaForTest(value * 100);

/** A cash check: `needsConfirm` means cash in hand would go below zero (PRD FR-6.7). */
export function cash(before: number, after: number, needsConfirm = false): CashCheck {
  return {
    cash_before_paisa: rupees(before),
    change_paisa: rupees(after - before),
    cash_after_paisa: rupees(after),
    below_zero: after < 0,
    lowers_cash: after < before,
    needs_confirm: needsConfirm,
  };
}

const RAHIM_REF: RecordRef = {
  id: IDS.rahim,
  code: '0010',
  name_ur: 'رحیم',
  name_en: 'Rahim',
};
const BANANA_REF: RecordRef = { id: IDS.banana, code: '0001', name_ur: 'کیلا', name_en: 'Banana' };
const BOX_REF: RecordRef = { id: IDS.box, code: '0001', name_ur: 'پیٹی', name_en: 'Box' };

/** Lot 40 of بکری 41: 50 received, 45 sold, 5 left, one day waiting (E2E-16). */
export function stockRow(overrides: Partial<StockRow> = {}): StockRow {
  return {
    lot_id: 40,
    lot_no: 1,
    bikri_id: 41,
    bikri_number: 1,
    received_date: '2026-10-09',
    seller: RAHIM_REF,
    pukhta: null,
    item: BANANA_REF,
    packing: BOX_REF,
    qty_received: 50,
    qty_sold: 45,
    qty_written_off: 0,
    qty_taken: 0,
    balance: 5,
    days_waiting: 1,
    write_offs: [],
    ...overrides,
  };
}

export function stockList(rows: StockRow[] = [stockRow()]): StockList {
  return {
    rows,
    total_count: rows.length,
    balance_total: rows.reduce((sum, row) => sum + row.balance, 0),
    own_rows: [],
    own_balance_total: 0,
    own_cost_left_paisa: paisaForTest(0),
    garden_rows: [],
    garden_balance_total: 0,
  };
}

/** A copy of a draft the way `bikri_open_for_edit` sends it: ids on the lots and the lines. */
export function withIds(draft: BikriDraft): BikriDraft {
  const copy = structuredClone(draft);
  copy.header.entry_date = '10-10-2026';
  copy.lots = copy.lots.map((lot) => ({
    ...lot,
    key: 'lot:40',
    lot_id: 40,
    // The backend's own draft of a saved lot says it has no purchase and no goods arrived entry.
    purchase_id: null,
    arrival_id: null,
    sales: lot.sales
      .filter((sale) => sale.buyer_account_id !== null)
      .map((sale, index) => ({
        ...sale,
        key: `sale:${String(101 + index)}`,
        sale_id: 101 + index,
      })),
  }));
  return copy;
}

/** The first posting of بکری 41 open for edit (E2E-13 starts from this). */
export function editable(overrides: Partial<EditableBikri> = {}): EditableBikri {
  const saved = savedEvent1();
  return {
    bikri_id: 41,
    number: 1,
    version: 3,
    entry_id: 7,
    posting_type: 'bikri_first',
    entry_date: '2026-10-10',
    editable: true,
    draft: withIds(EVENT1_REQUEST.draft),
    // The stored lot is the lot of the draft (the saved Event 1 lot has its own id).
    stored: saved.lots.map((lot) => ({ ...lot, id: 40 })),
    postings: saved.postings,
    ...overrides,
  };
}

/**
 * A later sale (entry 9) open for edit (E2E-16b): one lot and one line, Arshad 5 at 560. A later
 * posting has no header to change and no lot charges (PRD FR-12.17).
 */
export function editableLater(overrides: Partial<EditableBikri> = {}): EditableBikri {
  const base = editable();
  const draft = structuredClone(base.draft);
  const lot = draft.lots[0];
  if (lot !== undefined) {
    const line = lot.sales[1];
    lot.sales =
      line === undefined
        ? []
        : [{ ...line, key: 'sale:105', sale_id: 105, qty: '5', seller_rate: '560' }];
  }
  return { ...base, entry_id: 9, posting_type: 'bikri_later', draft, ...overrides };
}

export function cancelPreview(overrides: Partial<CancelPreview> = {}): CancelPreview {
  const saved = savedEvent1();
  return {
    bikri_id: 41,
    number: 1,
    version: 3,
    status: 'saved',
    postings: saved.postings,
    cash: cash(58_484, 58_484),
    closed_dates: [],
    can_cancel: true,
    ...overrides,
  };
}

/** The values of an undone save: the Event 1 draft with no ids (PRD FR-7.47). */
export function undoResult(): UndoResult {
  const draft = structuredClone(EVENT1_REQUEST.draft);
  draft.header.entry_date = '10-10-2026';
  draft.lots = draft.lots.map((lot) => ({
    ...lot,
    key: 'lot-1',
    sales: lot.sales
      .filter((sale) => sale.buyer_account_id !== null)
      .map((sale, index) => ({ ...sale, key: `line-${String(index + 1)}` })),
  }));
  return { bikri_id: 41, number: 1, reversal_entry_ids: [9], draft };
}

/**
 * The preview of a later sale for the lines sent: the double of `previewFor`, with a blank line
 * starting at what the lot has left (the backend's `remaining_qty`, here the lot's balance).
 */
export function laterSaleAnswer(sales: readonly SaleInput[], left: number): BikriPreview {
  const draft = withIds(EVENT1_REQUEST.draft);
  const lot = draft.lots[0];
  if (lot !== undefined) lot.sales = [...sales];
  const answer = previewFor(draft);
  for (const lotPreview of answer.lots) {
    for (const line of lotPreview.lines) {
      if (line.is_blank) line.remaining_qty = left;
    }
  }
  return answer;
}
