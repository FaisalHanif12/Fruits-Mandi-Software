// The columns of a report as the list grid wants them (PRD 19.1, NFR-USE-15, Code Standards F18,
// F22). The backend sends the columns of each report with a kind; the kind picks the grid's column
// type, so the same kind has the same width on every report. A balance has its side word in a slot
// after the figure, so the figures keep one right edge (NFR-USE-19). The names and the goods of a
// ledger line grow into the free width; the narration keeps its own width and waits for a wider
// window, so nothing scrolls sideways.

import type { ReportCell } from '../../bindings/ReportCell';
import type { ReportColumn } from '../../bindings/ReportColumn';
import type { ReportId } from '../../bindings/ReportId';
import type { ReportRow } from '../../bindings/ReportRow';
import type { ListColumn, ListContext } from '../../shared/grid';
import { isFromStep, type LayoutStep } from '../../shared/hooks/useLayoutStep';
import { cellText, emptyNameMessageId, sideText } from './cells';

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

/**
 * The layout steps a column is shown at (Code Standards F22, NFR-USE-15). A report has more columns
 * than a 1024 px window holds, and a table never scrolls sideways, so the less used columns wait
 * for a wider step; the most used come first. The widths of the column types decide what fits: the
 * grid is about 890 px wide at the compact step, 1,070 px at the normal and 1,530 px at the wide
 * step, and a growing name column keeps at least 160 px. The print holds every column.
 * `from` is the first step the column shows at (default compact), `until` the last (default wide).
 */
interface StepRange {
  readonly from?: LayoutStep;
  readonly until?: LayoutStep;
}

/** A column that is in the print and the backend's answer but has no room on the screen. */
const NEVER: StepRange = { from: 'wide', until: 'compact' };

const COLUMN_STEPS: Partial<Record<ReportId, Readonly<Record<string, StepRange>>>> = {
  // Sales register: date, buyer, item, quantity, rate and the buyer's total always; the rest follow.
  R2: { doc: { from: 'normal' }, value: { from: 'normal' }, city: { from: 'wide' } },
  R3: { doc: { from: 'normal' }, value: { from: 'normal' }, city: { from: 'wide' } },
  R4: { doc: { from: 'normal' }, value: { from: 'normal' }, city: { from: 'wide' } },
  // Lots (the PRD's columns): the charges that make up the deductions need the wide step, where
  // the quantity gives its room to them. The item and the store rent are in the print only.
  R6: {
    qty: { from: 'normal', until: 'normal' },
    item: NEVER,
    store_rent: NEVER,
    commission: { from: 'normal' },
    freight: { from: 'wide' },
    labour: { from: 'wide' },
    munshiana: { from: 'wide' },
    cash_paid: { from: 'wide' },
    phone_charge: { from: 'wide' },
  },
  R7: { avg_rate: { from: 'normal' } },
  R9: {
    doc: { from: 'normal' },
    qty: { from: 'normal' },
    side: { from: 'wide' },
    phone: { from: 'wide' },
    commission: { from: 'wide' },
  },
  R20: {
    age_0_7: { from: 'normal' },
    age_8_30: { from: 'normal' },
    city: { from: 'wide' },
    phone: { from: 'wide' },
  },
  R21: {
    oldest_date: { from: 'normal' },
    age_0_7: { from: 'wide' },
    age_8_30: { from: 'normal' },
    phone: { from: 'wide' },
  },
  R22: {
    oldest_date: { from: 'normal' },
    age_0_7: { from: 'wide' },
    age_8_30: { from: 'normal' },
    city: { from: 'wide' },
    phone: { from: 'wide' },
  },
  R24: { last_date: { from: 'normal' } },
  R27: { bill_date: { from: 'normal' } },
  R30: { returned: { from: 'normal' } },
};

/** True when the column of this report is shown at the step. */
function shownAt(id: ReportId, key: string, step: LayoutStep): boolean {
  const range = COLUMN_STEPS[id]?.[key];
  if (range === undefined) return true;
  return isFromStep(step, range.from ?? 'compact') && isFromStep(range.until ?? 'wide', step);
}

function gridColumn(id: ReportId, column: ReportColumn, index: number): ListColumn<ReportRowView> {
  const cell = (view: ReportRowView, context: ListContext) => {
    const found = view.row.cells[index];
    if (found === undefined) return '';
    // A name left empty on purpose (no city, no collector, no اڈا): say what it is.
    const none =
      index === 0 && found.type === 'empty' ? emptyNameMessageId(id, view.row.kind) : null;
    return none === null ? cellText(found, context) : context.text(none);
  };
  const base = { field: column.key, headerId: column.title_id, value: cell };
  switch (column.kind) {
    case 'date':
      return { ...base, type: 'date' };
    case 'month':
      // A month is written with its name ("October 2026"), which is wider than a date.
      return { ...base, type: 'text' };
    case 'money':
    case 'percent':
      return { ...base, type: 'figure' };
    case 'number':
      return { ...base, type: 'count' };
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
    case 'names':
      // A code with a name can be longer than a narrow column: it wraps instead of being cut, so
      // no name is ever shown half (the row grows with it).
      return { ...base, type: 'name', grow: true, wrap: true };
    case 'doc':
      // A kind and a number ("Personal Drawing Return 12") may be longer than the column.
      return { ...base, type: 'status', wrap: true };
    case 'detail':
      return { ...base, type: 'text', grow: true, wrap: true };
    case 'label':
      // The heading of a list in R24 is a sentence and may take two lines; a status is one word.
      return column.key === 'section'
        ? { ...base, type: 'name', grow: true, wrap: true }
        : { ...base, type: 'status' };
    case 'text':
      // The narration is long and less used: it comes with the wider windows. A code and a phone
      // number are short.
      if (column.key === 'code') return { ...base, type: 'code' };
      if (column.key === 'phone') return { ...base, type: 'short' };
      return { ...base, type: 'text', from: 'wide' };
  }
}

/**
 * The columns of a report for the layout step. A column keeps its place in the backend's list, so
 * its cell is found at the same index whatever is hidden (the totals use the key).
 */
export function reportColumns(
  id: ReportId,
  columns: readonly ReportColumn[],
  step: LayoutStep,
): ListColumn<ReportRowView>[] {
  return columns.flatMap((column, index) =>
    shownAt(id, column.key, step) ? [gridColumn(id, column, index)] : [],
  );
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
