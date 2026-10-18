// What the user has changed in a posting that is open for edit (PRD FR-12.18, plan 16). The screen
// keeps a copy of the loaded text of every cell (the baseline) and compares it with the text typed
// now: a cell is changed when the two texts differ. It works out nothing and adds no money. The
// answer marks the changed cells in the grids and fills the list "What you changed" (the audit
// comparison, old text next to new text), so the user sees before he saves exactly what the
// reversal and the new posting will carry.
//
// Cells the app keeps for itself (the "filled once" marks and the ids of saved rows) are not the
// user's: they never count as a change.

import type { EntryRow, PickedRecord } from '../../shared/grid';
import type { BikriDraftStores } from './draft-store';
import { LOT, SALE } from './types';

/** Hidden cells of a lot row. */
const LOT_HIDDEN: ReadonlySet<string> = new Set([LOT.sellerFilled, LOT.memoryFilledFor, LOT.lotId]);
/** Hidden cells of a sale row. */
const SALE_HIDDEN: ReadonlySet<string> = new Set([SALE.qtyFilled, SALE.saleId]);

interface RowCopy {
  typed: Readonly<Record<string, string>>;
  picked: Readonly<Record<string, PickedRecord>>;
  /** Its place in its lot (a line) or in the sale (a lot), from 0. */
  index: number;
  lotKey: string;
}

/** The loaded text of every row, by row id. */
export interface Baseline {
  lots: ReadonlyMap<string, RowCopy>;
  sales: ReadonlyMap<string, RowCopy>;
  header: Readonly<Record<string, string>>;
}

export function captureBaseline(stores: BikriDraftStores): Baseline {
  const lots = new Map<string, RowCopy>();
  const sales = new Map<string, RowCopy>();
  stores.lots.getState().rows.forEach((row, index) => {
    lots.set(row.rowId, { typed: row.typed, picked: row.picked ?? {}, index, lotKey: row.rowId });
    stores
      .salesFor(row.rowId)
      .getState()
      .rows.forEach((line, lineIndex) => {
        sales.set(line.rowId, {
          typed: line.typed,
          picked: line.picked ?? {},
          index: lineIndex,
          lotKey: row.rowId,
        });
      });
  });
  return { lots, sales, header: headerText(stores) };
}

/** The text of the header fields, to compare. The records are compared by id. */
function headerText(stores: BikriDraftStores): Record<string, string> {
  const header = stores.header.getState();
  return {
    entryDate: header.entryDate,
    billDate: header.billDate,
    book: header.book,
    truckNo: header.truckNo,
    zamindar: String(header.zamindar?.id ?? ''),
    adda: String(header.adda?.id ?? ''),
    driverName: header.driverName,
    driverPhone: header.driverPhone,
    totalFreight: header.totalFreight,
    store: String(header.store?.id ?? ''),
    note: header.note,
  };
}

/** One change, for the list. `before` and `after` are text, or a record for a picker cell. */
export interface ChangeItem {
  kind: 'changed' | 'added' | 'removed';
  scope: 'header' | 'lot' | 'line';
  rowId: string;
  /** The lot number (from 1) of a lot or a line. */
  lotNo: number;
  /** The line number (from 1) of a line. */
  lineNo: number | null;
  /** The cell that changed. Null for a whole line that was added or removed. */
  field: string | null;
  before: string;
  after: string;
  beforeRecord: PickedRecord | null;
  afterRecord: PickedRecord | null;
}

/** What a changed cell was. */
export interface CellWas {
  text: string;
  record: PickedRecord | null;
}

export interface EditDiff {
  items: ChangeItem[];
  /** Changed cells of the lots grid and of the sale lines grid: row id, then field. */
  lotCells: Record<string, Record<string, CellWas>>;
  saleCells: Record<string, Record<string, CellWas>>;
  /** True when nothing differs from what was loaded. */
  unchanged: boolean;
}

const SAME = (a: string | undefined, b: string | undefined) =>
  (a ?? '').trim() === (b ?? '').trim();

function fieldsOf(before: RowCopy | undefined, row: EntryRow, hidden: ReadonlySet<string>) {
  const names = new Set<string>([
    ...Object.keys(before?.typed ?? {}),
    ...Object.keys(row.typed),
    ...Object.keys(before?.picked ?? {}),
    ...Object.keys(row.picked ?? {}),
  ]);
  return [...names].filter((name) => !hidden.has(name));
}

/** A sale line with no buyer and no rate typed. */
function isBlankLine(row: EntryRow): boolean {
  return row.picked?.[SALE.buyer] === undefined && (row.typed[SALE.rate] ?? '').trim() === '';
}

interface Collect {
  items: ChangeItem[];
  cells: Record<string, Record<string, CellWas>>;
}

function compareRow(
  row: EntryRow,
  before: RowCopy | undefined,
  hidden: ReadonlySet<string>,
  context: {
    scope: 'lot' | 'line';
    lotNo: number;
    lineNo: number | null;
  },
  into: Collect,
): void {
  const added = before === undefined;
  // A new line with no buyer and no rate is a blank row: the preview and the save ignore it, so
  // it is not a change (PRD FR-7.52).
  if (added && context.scope === 'line' && isBlankLine(row)) return;
  for (const field of fieldsOf(before, row, hidden)) {
    const wasRecord = before?.picked[field] ?? null;
    const nowRecord = row.picked?.[field] ?? null;
    const wasText = before?.typed[field] ?? '';
    const nowText = row.typed[field] ?? '';
    const changed =
      wasRecord !== null || nowRecord !== null
        ? wasRecord?.id !== nowRecord?.id
        : !SAME(wasText, nowText);
    if (!changed) continue;
    (into.cells[row.rowId] ??= {})[field] = { text: wasText, record: wasRecord };
    into.items.push({
      kind: added ? 'added' : 'changed',
      scope: context.scope,
      rowId: row.rowId,
      lotNo: context.lotNo,
      lineNo: context.lineNo,
      field,
      before: wasText,
      after: nowText,
      beforeRecord: wasRecord,
      afterRecord: nowRecord,
    });
  }
}

/** Compares the stores with what was loaded. */
export function diffEdit(stores: BikriDraftStores, baseline: Baseline): EditDiff {
  const lots: Collect = { items: [], cells: {} };
  const sales: Collect = { items: [], cells: {} };
  const items: ChangeItem[] = [];

  // The header: only the fields that are text; a record is compared by its id.
  const headerNow = headerText(stores);
  for (const [field, now] of Object.entries(headerNow)) {
    if (SAME(baseline.header[field], now)) continue;
    items.push({
      kind: 'changed',
      scope: 'header',
      rowId: 'header',
      lotNo: 0,
      lineNo: null,
      field,
      before: baseline.header[field] ?? '',
      after: now,
      beforeRecord: null,
      afterRecord: null,
    });
  }

  const present = new Set<string>();
  stores.lots.getState().rows.forEach((lot, lotIndex) => {
    present.add(lot.rowId);
    compareRow(
      lot,
      baseline.lots.get(lot.rowId),
      LOT_HIDDEN,
      { scope: 'lot', lotNo: lotIndex + 1, lineNo: null },
      lots,
    );
    stores
      .salesFor(lot.rowId)
      .getState()
      .rows.forEach((line, lineIndex) => {
        present.add(line.rowId);
        compareRow(
          line,
          baseline.sales.get(line.rowId),
          SALE_HIDDEN,
          { scope: 'line', lotNo: lotIndex + 1, lineNo: lineIndex + 1 },
          sales,
        );
      });
  });

  // A loaded line that is gone: removed by the user.
  const lotOrder = stores.lots.getState().rows.map((row) => row.rowId);
  const removed: ChangeItem[] = [];
  for (const [rowId, copy] of baseline.sales) {
    if (present.has(rowId)) continue;
    removed.push({
      kind: 'removed',
      scope: 'line',
      rowId,
      lotNo: Math.max(lotOrder.indexOf(copy.lotKey), 0) + 1,
      lineNo: copy.index + 1,
      field: null,
      before: '',
      after: '',
      beforeRecord: copy.picked[SALE.buyer] ?? null,
      afterRecord: null,
    });
  }

  const all = [...items, ...lots.items, ...sales.items, ...removed];
  return { items: all, lotCells: lots.cells, saleCells: sales.cells, unchanged: all.length === 0 };
}
