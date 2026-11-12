// Test double for plan 16: the commands of goods left, later sale, edit, cancel, undo and write-off
// answered the way a healthy backend would, with the canned data of bikri-plan16-data.ts. The
// wrappers must have been mocked with vi.mock in the test file. `fakeBikri()` (the plan 12 and 13
// commands) is separate.

import { vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { CancelPreview } from '../../src/bindings/CancelPreview';
import type { EditChange } from '../../src/bindings/EditChange';
import type { EditPreview } from '../../src/bindings/EditPreview';
import type { EditableBikri } from '../../src/bindings/EditableBikri';
import type { StockRow } from '../../src/bindings/StockRow';
import { CommandError } from '../../src/shared/api';
import { EVENT1_REQUEST, previewFor } from './bikri-data';
import {
  cancelPreview,
  editable,
  editableLater,
  laterSaleAnswer,
  stockList,
  stockRow,
  undoResult,
  withIds,
} from './bikri-plan16-data';
import { sliceOf } from './paged';
import { refusal } from './ui-error';

export * from './bikri-plan16-data';

/** The paisa of a rate as typed ("610" or "1,200"), the way the backend reads a whole rupee. */
function paisaOf(text: string): number {
  return Number(text.replaceAll(',', '')) * 100;
}

/**
 * What the backend's review of an edit says (plan 44, PRD FR-7.70), for the one thing these tests
 * change: the rate of a line, or a line taken away. Like the real review it compares the draft with
 * the entry as loaded and lists each difference with its place; a changed day is not asked about.
 * The reason is required when the entry is not of today (2026-10-10 in these tests).
 */
export function fakeEditReview(baseline: EditableBikri, draft: BikriDraft): EditPreview {
  const changes: EditChange[] = [];
  baseline.draft.lots.forEach((lot, lotIndex) => {
    lot.sales.forEach((line, lineIndex) => {
      const now = draft.lots[lotIndex]?.sales[lineIndex];
      const place = {
        kind: 'line' as const,
        lot_no: lotIndex + 1,
        lot_counter: null,
        item_ur: null,
        item_en: null,
        line_no: lineIndex + 1,
        label_ur: null,
        label_en: null,
      };
      if (now === undefined) {
        changes.push({
          place,
          field: 'qty',
          before: { type: 'number', value: Number(line.qty) },
          after: { type: 'empty' },
        });
      } else if (now.seller_rate !== line.seller_rate) {
        changes.push({
          place,
          field: 'seller_rate',
          before: { type: 'money', paisa: paisaOf(line.seller_rate) as never },
          after: { type: 'money', paisa: paisaOf(now.seller_rate) as never },
        });
      }
    });
  });
  return {
    errors: [],
    review: {
      entry_date: baseline.entry_date,
      changes,
      effect: { parties: [], cash: null, partners: [], partners_locked: false },
      reason_required: baseline.entry_date !== '2026-10-10',
      needs_reopen: false,
    },
  };
}

export interface FakePlan16 {
  /**
   * Like the real backend, a save that takes cash below zero is refused until the user has said
   * yes (PRD FR-6.7). A test turns this on together with an answer of the preview that says so.
   */
  cashConfirmRequired: boolean;
  /** Every later sale the screen asked to preview: lot, day and the lines sent. */
  laterPreviews: { lotId: number; date: string; sales: unknown[] }[];
  laterSaves: { lotId: number; date: string; sales: unknown[]; confirm: boolean }[];
  editPreviews: { entryId: number; draft: BikriDraft }[];
  editSaves: {
    entryId: number;
    version: number;
    draft: BikriDraft;
    reason: string | null;
    confirm: boolean;
  }[];
  cancels: { id: number; version: number; reason: string; confirm: boolean }[];
  undos: { id: number; confirm: boolean }[];
  writeOffs: { lotId: number; qty: string; reason: string }[];
  writeOffUndos: { moveId: number; reason: string }[];
}

/**
 * Answers the commands of plan 16 as a healthy backend would. The wrappers must have been mocked
 * with vi.mock in the test file. `fakeBikri()` (the plan 12 and 13 commands) is separate.
 */
export function fakePlan16(
  options: {
    rows?: StockRow[];
    editable?: EditableBikri;
    /** The entry open for edit when it is asked for by id: entry 9 is the later sale. */
    editableLater?: EditableBikri;
    cancelPreview?: CancelPreview;
  } = {},
): FakePlan16 {
  const store: FakePlan16 = {
    cashConfirmRequired: false,
    laterPreviews: [],
    laterSaves: [],
    editPreviews: [],
    editSaves: [],
    cancels: [],
    undos: [],
    writeOffs: [],
    writeOffUndos: [],
  };
  const rows = options.rows ?? [stockRow()];
  vi.mocked(bikriApi.bikriStockList).mockImplementation((input) => {
    const matching = rows.filter(
      (row) =>
        (input.lot_id === null || row.lot_id === input.lot_id) &&
        (input.seller_id === null || row.seller.id === input.seller_id) &&
        (input.item_id === null || row.item.id === input.item_id) &&
        (row.balance > 0 || input.include_written_off),
    );
    // A page of the lots, with the count and the balance of the whole filter (PRD FR-19.11).
    return Promise.resolve({
      ...stockList(matching),
      rows: sliceOf(matching, input.offset, input.limit),
    });
  });
  vi.mocked(bikriApi.bikriLaterSalePreview).mockImplementation((lotId, date, sales) => {
    store.laterPreviews.push({ lotId, date, sales: [...sales] });
    return Promise.resolve(laterSaleAnswer(sales, rows[0]?.balance ?? 0));
  });
  const unconfirmed = (confirm: boolean) => {
    if (store.cashConfirmRequired && !confirm) {
      throw refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED');
    }
  };
  vi.mocked(bikriApi.bikriLaterSaleSave).mockImplementation((lotId, date, sales, confirm) => {
    unconfirmed(confirm);
    store.laterSaves.push({ lotId, date, sales: [...sales], confirm });
    const draft = withIds(EVENT1_REQUEST.draft);
    return Promise.resolve({
      bikri_id: 41,
      number: 1,
      lot_id: lotId,
      entry_id: 9,
      preview: previewFor(draft),
    });
  });
  const open = options.editable ?? editable();
  vi.mocked(bikriApi.bikriOpenForEdit).mockImplementation((_bikriId, entryId) =>
    Promise.resolve(entryId === 9 ? (options.editableLater ?? editableLater()) : open),
  );
  vi.mocked(bikriApi.bikriEditPreview).mockImplementation((entryId, draft) => {
    store.editPreviews.push({ entryId, draft });
    return Promise.resolve(previewFor(draft));
  });
  vi.mocked(bikriApi.bikriEditReview).mockImplementation((entryId, draft) =>
    Promise.resolve(
      fakeEditReview(entryId === 9 ? (options.editableLater ?? editableLater()) : open, draft),
    ),
  );
  vi.mocked(bikriApi.bikriEditSave).mockImplementation(
    (entryId, version, draft, reason, confirm) => {
      unconfirmed(confirm);
      store.editSaves.push({ entryId, version, draft, reason, confirm });
      return Promise.resolve({
        bikri_id: 41,
        number: 1,
        version: version + 1,
        old_entry_id: entryId,
        reversal_entry_id: 10,
        entry_id: 11,
        preview: previewFor(draft),
      });
    },
  );
  vi.mocked(bikriApi.bikriCancelPreview).mockResolvedValue(
    options.cancelPreview ?? cancelPreview(),
  );
  vi.mocked(bikriApi.bikriCancel).mockImplementation((id, version, reason, confirm) => {
    unconfirmed(confirm);
    store.cancels.push({ id, version, reason, confirm });
    return Promise.resolve({ bikri_id: id, number: 1, reversal_entry_ids: [10] });
  });
  vi.mocked(bikriApi.bikriUndo).mockImplementation((id, confirm) => {
    unconfirmed(confirm);
    store.undos.push({ id, confirm });
    return Promise.resolve(undoResult());
  });
  vi.mocked(bikriApi.bikriWriteOff).mockImplementation((lotId, qty, reason) => {
    store.writeOffs.push({ lotId, qty, reason });
    return Promise.resolve({
      lot_id: lotId,
      move_id: 70,
      qty: Number(qty),
      qty_written_off: Number(qty),
      balance: 0,
    });
  });
  vi.mocked(bikriApi.bikriWriteOffUndo).mockImplementation((moveId, reason) => {
    store.writeOffUndos.push({ moveId, reason });
    return Promise.resolve({
      lot_id: 40,
      reversal_move_id: 71,
      qty_written_off: 0,
      balance: 5,
    });
  });
  return store;
}

/** A refusal for one problem, as the backend sends it. */
export function refused(
  code: string,
  field: string | null = null,
  values: Record<string, string | number> = {},
) {
  return new CommandError(refusal([{ code, field, values }]));
}
