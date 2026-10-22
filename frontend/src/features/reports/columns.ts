// The columns of a report as the list grid wants them (PRD 19.1, NFR-USE-15, Code Standards F18,
// F22). The backend sends the columns of each report with a kind; the kind picks the grid's column
// type, so the same kind has the same width on every report. A balance has its side word in a slot
// after the figure, so the figures keep one right edge (NFR-USE-19). The names and the goods of a
// ledger line grow into the free width; the narration keeps its own width and waits for a wider
// window, so nothing scrolls sideways.

import type { ReportCell } from '../../bindings/ReportCell';
import type { ReportColumn } from '../../bindings/ReportColumn';
import type { ReportRow } from '../../bindings/ReportRow';
import type { ListColumn, ListContext } from '../../shared/grid';
import { cellText, sideText } from './cells';

/** One row of the grid: its place in the whole report is its key (two equal lines are possible). */
export interface ReportRowView {
  key: string;
  row: ReportRow;
}

/** The totals row under the table: the backend's cells, written the same way as the rows. */
export interface ReportTotals {
  labelId: string;
  cells: readonly ReportCell[];
}

function gridColumn(column: ReportColumn, index: number): ListColumn<ReportRowView> {
  const cell = (view: ReportRowView, context: ListContext) => {
    const found = view.row.cells[index];
    return found === undefined ? '' : cellText(found, context);
  };
  const base = { field: column.key, headerId: column.title_id, value: cell };
  switch (column.kind) {
    case 'date':
    case 'month':
      return { ...base, type: 'date' };
    case 'money':
      return { ...base, type: 'money' };
    case 'number':
      return { ...base, type: 'number' };
    case 'balance':
      return {
        ...base,
        type: 'balance',
        unit: (view, context) => {
          const found = view.row.cells[index];
          return found === undefined ? '' : sideText(found, context);
        },
      };
    case 'party':
      return { ...base, type: 'name', grow: true };
    case 'names':
      return { ...base, type: 'name', grow: true };
    case 'doc':
      // A kind and a number ("Personal Drawing Return 12") may be longer than the column.
      return { ...base, type: 'status', wrap: true };
    case 'detail':
      return { ...base, type: 'text', grow: true, wrap: true };
    case 'label':
      return { ...base, type: 'text' };
    case 'text':
      // The narration is long and less used: it comes with the wider windows. A code is short.
      return column.key === 'code'
        ? { ...base, type: 'code' }
        : { ...base, type: 'text', from: 'wide' };
  }
}

export function reportColumns(columns: readonly ReportColumn[]): ListColumn<ReportRowView>[] {
  return columns.map(gridColumn);
}

/**
 * The figures of the totals row, keyed by column. A column with no total has no key. The word of
 * the row goes under the key of the first column. A balance's side word is under `<key>:unit`.
 */
export function totalsFor(
  columns: readonly ReportColumn[],
  totals: ReportTotals,
  context: ListContext,
): { labelField: string; values: Record<string, string> } {
  const values: Record<string, string> = {};
  columns.forEach((column, index) => {
    const cell = totals.cells[index];
    if (cell === undefined || cell.type === 'empty') return;
    values[column.key] = cellText(cell, context);
    if (cell.type === 'balance') values[`${column.key}:unit`] = sideText(cell, context);
  });
  const first = columns[0]?.key ?? '';
  values[first] = values[first] ?? context.text(totals.labelId);
  return { labelField: first, values };
}
