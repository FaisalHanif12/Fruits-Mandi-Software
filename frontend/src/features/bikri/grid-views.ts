// What the grids show from the preview and the problems (plan 12 contract 1, 4, 11). Pure: the
// numbers are the backend's, written as text; the problems are placed on the cell their field
// path names. The lots grid shows every lot; the sale lines grid shows the selected lot.

import type { Working } from '../../bindings/Working';
import type { Grouping } from '../../shared/format';
import { lineView, lotView, type Say } from './preview-view';
import { SALE } from './types';
import { LINE_GRID_FIELD, LOT_GRID_FIELD, type Problem } from './problems';
import type { BikriPreview } from './types';

type PerRow<T> = Record<string, Record<string, T>>;

/** What one grid takes from the preview: calculated text, workings, and problems per cell. */
export interface GridView {
  calc: PerRow<string>;
  workings: PerRow<Working>;
  errors: PerRow<string>;
  warnings: PerRow<string>;
}

function emptyView(): GridView {
  return { calc: {}, workings: {}, errors: {}, warnings: {} };
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
    }
  }
  return { lots, sales };
}
