// What the grids show from the preview and the problems (plan 12 contract 1, 4, 11). Pure: the
// numbers are the backend's, written as text; the problems are placed on the cell their field
// path names. The lots grid shows every lot; the sale lines grid shows the selected lot.

import type { Working } from '../../bindings/Working';
import type { Grouping } from '../../shared/format';
import { lineView, lotView, type Say } from './preview-view';
import { SALE } from './types';
import {
  LINE_GRID_FIELD,
  LINE_NEEDS,
  LOT_GRID_FIELD,
  saysEmpty,
  whereOf,
  type Problem,
} from './problems';
import type { BikriPreview } from './types';

type PerRow<T> = Record<string, Record<string, T>>;

/**
 * A problem of the quantity of a line, written in plain view under the sale lines (PRD FR-12.6):
 * the line sells more than the lot has left, or the quantity is not a number. The cell turns red as
 * always, but a tooltip is not enough to learn why a line shows no figures.
 */
export interface QtyNote {
  /** The row of the line, for a key. */
  rowId: string;
  /** The lot and the line it is about, from 0 (the screen counts from 1). */
  lot: number;
  line: number;
  /** The problem as words in the app language. */
  words: string;
}

/**
 * What a line that has something typed on it still needs (D8 Bug 1): the buyer, the quantity or the
 * rate. It is told in words under the sale lines at once, so a line with no figures is never left
 * without a reason. A blank row (nothing typed) has none.
 */
export interface MissingNote {
  rowId: string;
  lot: number;
  line: number;
  /** What is missing, as words in the app language. */
  words: string;
}

/** What one grid takes from the preview: calculated text, workings, and problems per cell. */
export interface GridView {
  calc: PerRow<string>;
  workings: PerRow<Working>;
  errors: PerRow<string>;
  warnings: PerRow<string>;
  /** The quantity problems of the lines this grid shows, in line order. */
  qtyNotes: QtyNote[];
  /** What the incomplete lines this grid shows still need, in line order. */
  missingNotes: MissingNote[];
  /**
   * The lines the backend refused for a quantity over what the lot has left (`ERR_QTY_EXCEEDS_LOT`), by
   * row, with the text of the quantity the answer was for. Only while the answer is for the text on
   * screen: the moment the text changes the hold is lifted, until the next answer says again. The screen
   * keeps the cursor on that quantity cell (D8 Bug 1, B2). It only reacts to the backend's error.
   */
  holds: Record<string, string>;
}

function emptyView(): GridView {
  return {
    calc: {},
    workings: {},
    errors: {},
    warnings: {},
    qtyNotes: [],
    missingNotes: [],
    holds: {},
  };
}

/** Puts the words on a cell. The first problem of a cell stays; the list shows them all. */
function put(target: PerRow<string>, rowId: string, field: string, words: string): void {
  const row = (target[rowId] ??= {});
  row[field] ??= words;
}

interface GridViewInput {
  preview: BikriPreview | undefined;
  problems: readonly Problem[];
  /** The keys of the lots, in draft order: `lots[i]` of a field path is `lotKeys[i]`. */
  lotKeys: readonly string[];
  /** The selected lot (the one the sale lines grid shows). */
  selectedLot: string | null;
  /** The keys of the lines of each lot, by lot index (the order of `lotKeys`). */
  lineKeysByLot: readonly (readonly string[])[];
  /** The whole-sale view: the sale lines grid shows the lines of every lot, each with its lot. */
  showAll: boolean;
  /** The words of the lot column for the lot at this index (its number and seller). */
  lotLabel: (lotIndex: number) => string;
  grouping: Grouping;
  say: Say;
  /** One problem as words in the app language. */
  words: (problem: Problem) => string;
  /** The answer is for the draft on screen now (not an older one still shown). Default: yes. */
  current?: boolean;
  /**
   * Buyer text typed on a line and never chosen, by the row of the line (shared/grid leftText.ts). Only
   * rows that have no buyer chosen are in it. The line says so in words (D8 Bug 1).
   */
  buyerLeft?: Readonly<Record<string, string>>;
}

export function buildGridViews(input: GridViewInput): { lots: GridView; sales: GridView } {
  const { preview, problems, lotKeys, selectedLot, lineKeysByLot, showAll, lotLabel } = input;
  const { grouping, say, words } = input;
  const lots = emptyView();
  const sales = emptyView();
  for (const lot of preview?.lots ?? []) {
    const view = lotView(lot, grouping);
    lots.calc[lot.key] = view.calc;
    lots.workings[lot.key] = view.workings;
    if (!showAll && lot.key !== selectedLot) continue;
    for (const line of lot.lines) {
      const lineCells = lineView(line, grouping, say);
      sales.calc[line.key] = lineCells.calc;
      sales.workings[line.key] = lineCells.workings;
    }
  }
  if (showAll) {
    // The lot column comes from the draft, not from the answer, so it is there before the first one.
    lineKeysByLot.forEach((keys, lotIndex) => {
      for (const key of keys) {
        const cells = (sales.calc[key] ??= {});
        cells[SALE.lot] = lotLabel(lotIndex);
      }
    });
  }
  const selectedIndex = selectedLot === null ? -1 : lotKeys.indexOf(selectedLot);
  for (const problem of problems) {
    const into = problem.severity === 'error' ? 'errors' : 'warnings';
    const { where } = problem;
    if (where.kind === 'lot') {
      const column = LOT_GRID_FIELD[where.sub];
      const key = lotKeys[where.lot];
      if (column !== undefined && key !== undefined) put(lots[into], key, column, words(problem));
    } else if (where.kind === 'line' && (showAll || where.lot === selectedIndex)) {
      const column = LINE_GRID_FIELD[where.sub];
      const key = lineKeysByLot[where.lot]?.[where.line];
      if (column !== undefined && key !== undefined) put(sales[into], key, column, words(problem));
      if (column === SALE.qty && key !== undefined && problem.severity === 'error') {
        sales.qtyNotes.push({
          rowId: key,
          lot: where.lot,
          line: where.line,
          words: words(problem),
        });
      }
    }
  }
  // A line with something typed on it and no figures always says why in its row (D8 Bug 1, real cause:
  // a line with a buyer, a quantity and a rate showed no figures and no word, because the one error the
  // backend sent was one of the "this is empty" answers that wait for the save, and none of the three
  // cells it knew). This is a rule, not a list of codes: whichever error the backend sent, and even when
  // it sent none, the row tells. Blank rows stay silent (PRD FR-7.52).
  const errorsOfLine = new Map<string, Problem[]>();
  for (const item of preview?.errors ?? []) {
    const where = whereOf(item.field);
    if (where.kind !== 'line') continue;
    if (!showAll && where.lot !== selectedIndex) continue;
    const key = lineKeysByLot[where.lot]?.[where.line];
    if (key === undefined) continue;
    const list = errorsOfLine.get(key) ?? [];
    list.push({ item, severity: 'error', where });
    errorsOfLine.set(key, list);
  }
  const lineAt = new Map<string, { lot: number; line: number }>();
  lineKeysByLot.forEach((keys, lot) => {
    keys.forEach((key, line) => {
      lineAt.set(key, { lot, line });
    });
  });
  const note = (rowId: string, text: string, column?: string) => {
    const at = lineAt.get(rowId);
    if (at === undefined) return;
    if (sales.missingNotes.some((n) => n.rowId === rowId && n.words === text)) return;
    sales.missingNotes.push({ rowId, lot: at.lot, line: at.line, words: text });
    if (column !== undefined) put(sales.warnings, rowId, column, text);
  };
  const buyerLeft = input.buyerLeft ?? {};
  for (const [rowId, list] of errorsOfLine) {
    for (const problem of list) {
      if (!saysEmpty(problem.item.code) || problem.where.kind !== 'line') continue;
      const need = LINE_NEEDS[problem.where.sub];
      if (need === 'buyer' && buyerLeft[rowId] !== undefined) continue;
      if (need !== undefined) {
        note(rowId, say(`bikri-line-missing-${need}`), LINE_GRID_FIELD[problem.where.sub]);
      } else {
        // An "empty" answer for a part of the line that has no cell of its own (a walk-in buyer's name):
        // the backend's own words, at once, not held back until the save.
        note(rowId, words(problem));
      }
    }
  }
  // A buyer typed in the cell and never chosen: said in words, even when the backend sees a blank row.
  for (const [rowId, text] of Object.entries(buyerLeft)) {
    const at = lineAt.get(rowId);
    if (at === undefined || (!showAll && at.lot !== selectedIndex)) continue;
    note(rowId, say('bikri-line-buyer-left', { text }), LINE_GRID_FIELD['buyer_account_id']);
  }
  if (input.current !== false) {
    for (const lot of preview?.lots ?? []) {
      lot.lines.forEach((line) => {
        if (line.is_blank || line.calc !== null) return;
        const at = lineAt.get(line.key);
        if (at === undefined || (!showAll && at.lot !== selectedIndex)) return;
        const own = errorsOfLine.get(line.key) ?? [];
        // Told already: a refusal the lists show, a note above, or the buyer text that was left.
        const told =
          own.length > 0 ||
          sales.missingNotes.some((n) => n.rowId === line.key) ||
          buyerLeft[line.key] !== undefined;
        if (told) return;
        // The lot or the header has its own problem that explains it: it is shown where it belongs.
        const lotOrHeader = (preview?.errors ?? []).some((item) => {
          const where = whereOf(item.field);
          return where.kind === 'header' || (where.kind === 'lot' && where.lot === at.lot);
        });
        if (lotOrHeader) return;
        note(line.key, say('bikri-line-no-figures'));
      });
    }
  }
  sales.missingNotes.sort((a, b) => a.lot - b.lot || a.line - b.line);
  // A line the answer refuses for a quantity over what the lot has left holds the cursor on its quantity
  // (D8 Bug 1, B2). Only the answer of the preview says so, and only while it is for the text on screen.
  if (input.current !== false) {
    for (const item of preview?.errors ?? []) {
      if (item.code !== 'ERR_QTY_EXCEEDS_LOT') continue;
      const where = whereOf(item.field);
      if (where.kind !== 'line' || where.sub !== 'qty') continue;
      if (!showAll && where.lot !== selectedIndex) continue;
      const key = lineKeysByLot[where.lot]?.[where.line];
      if (key !== undefined) sales.holds[key] = SALE.qty;
    }
  }
  return { lots, sales };
}
