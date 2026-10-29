// The item lines of one entry (PRD FR-12.41): a trip purchase entry and a garden's goods arrived
// entry each hold several lines, saved together. Each line is its own purchase or arrival. This file
// is the text side of the lines grid: the columns of the grid, what makes a line blank, the paths
// the backend's problems use for a line, and the cells they point at. Nothing here works out money
// or a quantity (CLAUDE.md rule 1): a split is the text the user typed, and the backend works out
// what is left for our shop.
//
// The lines live in a row store above the grid (shared/grid `createEntryStore`), never only in the
// grid, so a language switch loses nothing (Code Standards F39).

import type { ArrivalLineInput } from '../../bindings/ArrivalLineInput';
import type { PurchaseLineInput } from '../../bindings/PurchaseLineInput';
import { FIT_MIN, type EntryColumn, type EntryRow, type EntryStore } from '../../shared/grid';
import type { LayoutStep } from '../../shared/hooks/useLayoutStep';

/** A line of a trip purchase entry, or of a garden's goods arrived entry. */
export type GoodsKind = 'purchase' | 'arrival';

/** The keys of the cells of a line, in `typed` and `picked` of its row. */
export const LINE = {
  item: 'item',
  packing: 'packing',
  qty: 'qty',
  weight: 'weight',
  rate: 'rate',
  freight: 'freight',
  labour: 'labour',
  otherCost: 'otherCost',
  /** The quantity of the line that is for another shop. Empty: all of it is for our shop. */
  otherQty: 'otherQty',
  /** The name of the other shop, as typed. */
  shop: 'shop',
  /** Calculated: the quantity left for our shop, from the backend's preview. */
  ourQty: 'ourQty',
  /** Calculated: the value of the line, from the backend's preview (a purchase only). */
  value: 'value',
} as const;

const EMPTY_ROWS: ReadonlySet<string> = new Set();

/**
 * The columns of the lines grid, in reading order. At the narrowest window (1024) the value of the
 * line is left out so the grid fits without a sideways scroll (NFR-USE-15); every cell that can be
 * typed in is always there and always editable (PRD FR-12.41, Umar's decision D3).
 *
 * Enter stops only where a line needs its answer: the item, the packing, the quantity and, for a
 * purchase, the rate, and the weight of a line whose item is sold by weight. From the last stop it
 * goes to the item of the next line. Freight, labour, other cost, the part for another shop and the
 * shop's name are optional: they are reached with a click or the arrow keys, and what is typed in
 * them is sent as before. The two calculated cells are shown from the backend's preview and are
 * never typed. The item and the packing are chosen from their lists; the shop is free text, as
 * typed (FR-7.25).
 *
 * A row holds an item sold by weight when the item chosen says so (`soldByWeight`, from its search
 * hit, known the moment it is chosen). `weightRows` is the fallback: the rows the backend has told
 * are by weight (`ERR_WEIGHT_REQUIRED`, `working-weight-value`). It is read at the moment Enter is
 * pressed, so the columns stay the same while it changes.
 */
export function goodsLineColumns(
  kind: GoodsKind,
  step: LayoutStep,
  weightRows: () => ReadonlySet<string> = () => EMPTY_ROWS,
): readonly EntryColumn[] {
  const goods: readonly EntryColumn[] = [
    {
      field: LINE.item,
      type: 'picker',
      picker: { kind: 'master', scope: 'item' },
      headerId: 'glossary-qism-maal',
    },
    {
      field: LINE.packing,
      type: 'picker',
      picker: { kind: 'master', scope: 'packing' },
      headerId: 'glossary-packing',
    },
    { field: LINE.qty, type: 'qty', headerId: 'goods-line-col-qty' },
    {
      field: LINE.weight,
      type: 'weight',
      headerId: 'goods-line-col-weight',
      // A weight item needs its weight: the item chosen says so (its search hit), so no answer is
      // waited for; the backend's answer is the fallback. A weight already typed is a stop too.
      enterStop: (row) =>
        row.picked?.[LINE.item]?.soldByWeight === true ||
        weightRows().has(row.rowId) ||
        lineText(row, LINE.weight) !== '',
    },
  ];
  const costs: readonly EntryColumn[] =
    kind === 'purchase'
      ? [
          { field: LINE.rate, type: 'rate', headerId: 'goods-line-col-rate' },
          { field: LINE.freight, type: 'money', headerId: 'glossary-kiraya', enterStop: false },
          { field: LINE.labour, type: 'money', headerId: 'glossary-mazdoori', enterStop: false },
          {
            field: LINE.otherCost,
            type: 'money',
            headerId: 'purchases-field-other-cost',
            enterStop: false,
          },
        ]
      : [];
  const split: readonly EntryColumn[] = [
    {
      field: LINE.otherQty,
      type: 'qty',
      headerId: 'goods-line-col-other-qty',
      minWidth: FIT_MIN.heading,
      enterStop: false,
    },
    {
      field: LINE.shop,
      type: 'text',
      headerId: 'goods-line-col-shop',
      minWidth: FIT_MIN.heading,
      enterStop: false,
    },
    {
      field: LINE.ourQty,
      type: 'calc',
      headerId: 'goods-line-col-our-qty',
      minWidth: FIT_MIN.figure,
    },
  ];
  const value: readonly EntryColumn[] =
    kind === 'purchase' && step !== 'compact'
      ? [{ field: LINE.value, type: 'calc', headerId: 'goods-line-col-value' }]
      : [];
  return [...goods, ...costs, ...split, ...value];
}

/** The typed cells of a line, without the two the backend works out. */
const TYPED_CELLS = [
  LINE.qty,
  LINE.weight,
  LINE.rate,
  LINE.freight,
  LINE.labour,
  LINE.otherCost,
  LINE.otherQty,
  LINE.shop,
] as const;

/**
 * A line with nothing typed or chosen in it. A blank line is not sent and is not an error: the
 * grid keeps an empty row at the end for the next line, like the sale lines (PRD FR-7.52).
 */
export function lineIsBlank(row: EntryRow): boolean {
  if (Object.keys(row.picked ?? {}).length > 0) return false;
  return TYPED_CELLS.every((field) => (row.typed[field] ?? '').trim() === '');
}

/** The lines that hold something: the ones the entry sends. */
export function filledLines(rows: readonly EntryRow[]): EntryRow[] {
  return rows.filter((row) => !lineIsBlank(row));
}

/** True once any line holds something, so leaving the entry asks first (PRD FR-7.9). */
export function linesAreFilled(rows: readonly EntryRow[]): boolean {
  return rows.some((row) => !lineIsBlank(row));
}

/** The text of a cell, trimmed; empty when there is none. */
export function lineText(row: EntryRow, field: string): string {
  return (row.typed[field] ?? '').trim();
}

/** The text of a cell, or null when it is empty, so the backend reads it as not typed. */
export function lineTextOrNull(row: EntryRow, field: string): string | null {
  const text = lineText(row, field);
  return text === '' ? null : text;
}

/** The column of the grid each field of a line, as the backend names it, belongs to. */
export const LINE_ERROR_COLUMN: Readonly<Record<string, string>> = {
  item_id: LINE.item,
  packing_id: LINE.packing,
  qty: LINE.qty,
  weight: LINE.weight,
  rate: LINE.rate,
  freight: LINE.freight,
  labour: LINE.labour,
  other_cost: LINE.otherCost,
  other_qty: LINE.otherQty,
  shop_name: LINE.shop,
};

const LINE_PATH = /^lines\[(\d+)\](?:\.(.+))?$/;

/** Where a field path of the backend points: a line and the field of it, or nothing of a line. */
export function lineOfPath(path: string): { line: number; sub: string } | null {
  const found = LINE_PATH.exec(path);
  return found === null ? null : { line: Number(found[1]), sub: found[2] ?? '' };
}

type PerRow<T> = Readonly<Record<string, Readonly<Record<string, T>>>>;

/**
 * What the backend refused on the lines, placed on the cell it is about: the row of the line (the
 * backend counts the lines that were sent, which are the filled ones, in order) and the column of
 * the field. The words come from `text`. A problem on a part of a line that has no cell is left to
 * the list under the form.
 *
 * `sent` are the rows that were sent, in the order they were sent (`filledLines`).
 */
export function lineCellErrors<T extends { field: string | null }>(
  items: readonly T[],
  sent: readonly EntryRow[],
  text: (item: T) => string,
): PerRow<string> {
  const cells: Record<string, Record<string, string>> = {};
  for (const item of items) {
    if (item.field === null) continue;
    const where = lineOfPath(item.field);
    if (where === null) continue;
    const row = sent[where.line];
    const column = LINE_ERROR_COLUMN[where.sub];
    if (row === undefined || column === undefined) continue;
    const here = cells[row.rowId] ?? {};
    // The first problem of a cell is the one shown; the list under the form has them all.
    here[column] ??= text(item);
    cells[row.rowId] = here;
  }
  return cells;
}

/** The ids and the text of the lines, as one text: what changes when anything on a line changes. */
export function linesText(rows: readonly EntryRow[]): string {
  return JSON.stringify(
    rows.map((row) => [
      row.typed,
      Object.fromEntries(Object.entries(row.picked ?? {}).map(([field, hit]) => [field, hit.id])),
    ]),
  );
}

/**
 * What the purchase entry sends for its lines: the filled ones, in the order of the grid, with the
 * ids of the item and the packing and the text typed (CLAUDE.md rule 1). `note` is the entry's
 * note, which every line carries. An empty cell is sent as nothing, so the backend reads it as
 * not typed; the split is the text typed, and the backend works out what is left for our shop.
 */
export function purchaseLineInputs(rows: readonly EntryRow[], note: string): PurchaseLineInput[] {
  const entryNote = note.trim() === '' ? null : note.trim();
  return filledLines(rows).map((row) => ({
    // The row's own id: the preview echoes it, so its numbers find their row (PRD FR-12.41).
    key: row.rowId,
    item_id: row.picked?.[LINE.item]?.id ?? null,
    packing_id: row.picked?.[LINE.packing]?.id ?? null,
    qty: lineText(row, LINE.qty),
    weight: lineTextOrNull(row, LINE.weight),
    rate: lineText(row, LINE.rate),
    freight: lineTextOrNull(row, LINE.freight),
    labour: lineTextOrNull(row, LINE.labour),
    other_cost: lineTextOrNull(row, LINE.otherCost),
    note: entryNote,
    other_qty: lineTextOrNull(row, LINE.otherQty),
    shop_name: lineTextOrNull(row, LINE.shop),
  }));
}

/** What the goods arrived entry sends for its lines. The same rules as `purchaseLineInputs`. */
export function arrivalLineInputs(rows: readonly EntryRow[], note: string): ArrivalLineInput[] {
  const entryNote = note.trim() === '' ? null : note.trim();
  return filledLines(rows).map((row) => ({
    key: row.rowId,
    item_id: row.picked?.[LINE.item]?.id ?? null,
    packing_id: row.picked?.[LINE.packing]?.id ?? null,
    qty: lineText(row, LINE.qty),
    weight: lineTextOrNull(row, LINE.weight),
    note: entryNote,
    other_qty: lineTextOrNull(row, LINE.otherQty),
    shop_name: lineTextOrNull(row, LINE.shop),
  }));
}

/** Starts the lines again: one empty row, no focus to restore. */
export function resetLines(store: EntryStore): void {
  store.setState({
    rows: [{ rowId: globalThis.crypto.randomUUID(), typed: {} }],
    focus: null,
    pendingFocus: null,
  });
}

/**
 * The rows whose item is sold by weight, as the backend tells it: `ERR_WEIGHT_REQUIRED` on the
 * weight of a line (it comes as soon as the item is chosen and no weight is typed), or the working
 * of the value of a line that is by weight (`working-weight-value`, once the weight is typed).
 * `sent` are the rows that were sent, in the order they were sent (`filledLines`); `weightValueKeys`
 * are the keys of the lines whose value is worked out from a weight. Nothing is worked out here.
 */
export function weightRowIds(
  errors: readonly { code: string; field: string | null }[],
  sent: readonly EntryRow[],
  weightValueKeys: Iterable<string> = [],
): Set<string> {
  const rows = new Set<string>(weightValueKeys);
  for (const item of errors) {
    if (item.code !== 'ERR_WEIGHT_REQUIRED' || item.field === null) continue;
    const at = lineOfPath(item.field);
    const row = at?.sub === 'weight' ? sent[at.line] : undefined;
    if (row !== undefined) rows.add(row.rowId);
  }
  return rows;
}
