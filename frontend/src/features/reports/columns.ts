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
import type { ReportRowKind } from '../../bindings/ReportRowKind';
import type { ListColumn, ListContext } from '../../shared/grid';
import { isNegativePaisa, isPositivePaisa } from '../../shared/format';
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

/**
 * The gardens report (R36) has two layouts with some keys in common (the garden, what the shop and the
 * pockets paid), so its steps are chosen by the layout: the partner layout is the one that has a
 * `partner` column. By garden has 15 columns: the garden, its state, the rent, the installments, the
 * money in, the running balance and the rent due always; the costs come at the normal step; what the
 * shop and the pockets paid and the sales (our shop, other shops) at the wide step; the day, the
 * other money in and the commission are in the print only. By partner has 7: his percent and his
 * money always; what he paid from his pocket from the normal step.
 */
const GARDEN_LAYOUT_STEPS: Readonly<Record<string, StepRange>> = {
  costs: { from: 'normal' },
  paid_shop: { from: 'wide' },
  paid_pocket: { from: 'wide' },
  sales_our_shop: { from: 'wide' },
  sales_other_shops: { from: 'wide' },
  date: NEVER,
  other_in: NEVER,
  commission: NEVER,
};
const GARDEN_PARTNER_LAYOUT_STEPS: Readonly<Record<string, StepRange>> = {
  paid_pocket: { from: 'normal' },
};

/** The steps of a report's columns; most reports have one map, R36 has one for each layout. */
function stepsOf(
  id: ReportId,
  columns: readonly Pick<ReportColumn, 'key'>[],
): Readonly<Record<string, StepRange>> | undefined {
  if (id !== 'R36') return COLUMN_STEPS[id];
  return columns.some((column) => column.key === 'partner')
    ? GARDEN_PARTNER_LAYOUT_STEPS
    : GARDEN_LAYOUT_STEPS;
}

const COLUMN_STEPS: Partial<Record<ReportId, Readonly<Record<string, StepRange>>>> = {
  // The sale book (R1) has the 15 columns of the seller's bill and a table never scrolls sideways: the
  // seller, the item, the quantity, the gross, the deductions and the net always; the day and the
  // commission from the normal step; the document, the freight, the labour and the cash paid at the
  // wide step. The munshiana, the store rent and the phone charge are in the print, which holds the
  // whole bill (and in the deductions, which is the sum of them all).
  R1: {
    date: { from: 'normal' },
    commission: { from: 'normal' },
    doc: { from: 'wide' },
    freight: { from: 'wide' },
    labour: { from: 'wide' },
    cash_paid: { from: 'wide' },
    munshiana: NEVER,
    store_rent: NEVER,
    phone_charge: NEVER,
  },
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
  // The party day sheet (R35): the short code, party, balance at the start, the day's debit and
  // credit and what is left always. The collector's blank column is for the print and the widest
  // step. The slip of one party (quantity, item, rate, amount) fits at every step.
  R35: { collector: { from: 'wide' } },
  R30: { returned: { from: 'normal' } },
  // Money lent (R32): the person, what is lent, what is still open, the return-by day and the days
  // past it always; the two ways of returning wait for the normal step, the job and the oldest open
  // day for the wide step. The print holds every column.
  R32: {
    staff: { from: 'wide' },
    returned_cash: { from: 'normal' },
    returned_salary: { from: 'normal' },
    oldest_open: { from: 'wide' },
  },
  // Stock (R10): the date, who sent it, the item, what came, what is left and for how long always;
  // the document and what was sold wait for the normal step, the packing and the write-offs for the
  // wide step.
  R10: {
    doc: { from: 'normal' },
    sold: { from: 'normal' },
    packing: { from: 'wide' },
    written_off: { from: 'wide' },
  },
  // The receipts register (R23) has a collector the payments register does not: it comes with the
  // normal step. (R15 and R25 fit at every step; their narration waits for the wide step like any
  // narration.)
  R23: { collector: { from: 'normal' } },
  // Trip profit or loss (R33) has up to 15 columns and a table never scrolls sideways, so the most
  // used come first and the rest wait for a wider step. Keys that two layouts share are one entry
  // here: the trip layout (trip, purchase, date, item and the figures), the item, day and month
  // layouts (the same figures), and the partner layout (trip, partner, share and his money). The
  // print holds every column.
  R33: {
    purchase: { from: 'normal' },
    sold_qty: { from: 'normal' },
    money_back: { from: 'normal' },
    commission: { from: 'wide' },
    cost_sold: { from: 'wide' },
    left_cost: { from: 'wide' },
    money_in_pocket: { from: 'wide' },
    date: NEVER,
    bought_qty: NEVER,
    written_off_qty: NEVER,
    written_off_cost: NEVER,
  },
};

/**
 * The columns whose figure can be a loss, by report: they get a red tone, a sign and a word ("Loss",
 * "Profit"), never colour alone (NFR-USE-12, PRD E50). In R33 these are the result of a trip or a
 * purchase, and his part of it so far in the partner layout. Other reports keep a plain figure.
 */
const SIGNED_COLUMNS: Partial<Record<ReportId, ReadonlySet<string>>> = {
  R33: new Set(['result', 'net']),
};

/** The columns of a report that carry a loss word, for the totals row too. */
export function signedColumnsOf(id: ReportId): ReadonlySet<string> {
  return SIGNED_COLUMNS[id] ?? NO_KEYS;
}

const NO_KEYS: ReadonlySet<string> = new Set();

/** The words of a result: "Loss" under zero, "Profit" over zero, nothing at zero. */
function resultWord(cell: ReportCell, context: ListContext): string {
  if (cell.type !== 'money') return '';
  if (isNegativePaisa(cell.paisa)) return context.text('report-result-loss');
  if (isPositivePaisa(cell.paisa)) return context.text('report-result-profit');
  return '';
}

/** A result that is a loss is red. */
function resultTone(cell: ReportCell): 'due' | undefined {
  return cell.type === 'money' && isNegativePaisa(cell.paisa) ? 'due' : undefined;
}

/** The label columns that hold one short word (a state, a side, a mark, the kind of an entry). */
const SHORT_LABEL_KEYS: ReadonlySet<string> = new Set(['status', 'side', 'staff', 'heading']);

/** True when the column of this report is shown at the step. */
function shownAt(
  steps: Readonly<Record<string, StepRange>> | undefined,
  key: string,
  step: LayoutStep,
): boolean {
  const range = steps?.[key];
  if (range === undefined) return true;
  return isFromStep(step, range.from ?? 'compact') && isFromStep(range.until ?? 'wide', step);
}

/** What a column needs of a row of the grid: its cells and what kind of row it is. */
export interface RowLike {
  row: { cells: readonly ReportCell[]; kind: ReportRowKind };
}

/**
 * One column of a table of cells as the list grid wants it, by the kind of the backend's column. The
 * reports and the analytics views draw their tables the same way (NFR-USE-10). `emptyName` gives the
 * words for a first cell the backend left empty on purpose (no city, no collector, no اڈا).
 */
export function listColumnOf<V extends RowLike>(
  column: Pick<ReportColumn, 'key' | 'title_id' | 'kind'>,
  index: number,
  emptyName: (rowKind: ReportRowKind) => string | null = () => null,
  signed = false,
  shortSale = false,
): ListColumn<V> {
  const cell = (view: V, context: ListContext) => {
    const found = view.row.cells[index];
    if (found === undefined) return '';
    // A name left empty on purpose: say what it is.
    const none = index === 0 && found.type === 'empty' ? emptyName(view.row.kind) : null;
    return none === null ? cellText(found, context, shortSale) : context.text(none);
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
      // A result can be a loss: the figure is red, with its sign and a word after it (PRD E50).
      if (signed && column.kind === 'money') {
        return {
          ...base,
          type: 'balance',
          unit: (view: V, context: ListContext) => {
            const found = view.row.cells[index];
            return found === undefined ? '' : resultWord(found, context);
          },
          tone: (view: V) => {
            const found = view.row.cells[index];
            return found === undefined ? undefined : resultTone(found);
          },
        };
      }
      return { ...base, type: 'figure' };
    case 'number':
      return { ...base, type: 'count' };
    case 'balance':
      return {
        ...base,
        type: 'balance',
        unit: (view: V, context: ListContext) => {
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
    case 'message':
      // A sentence from a message (a partner statement line, PRD FR-17.10) is as long as its words.
      return { ...base, type: 'text', grow: true, wrap: true };
    case 'label':
      // A status, a side, a staff mark or a kind of line is one short word; any other label (the
      // heading of a list in R24, the name of a line of the day summary in R34) is a sentence that
      // takes the free width and may take two lines.
      return SHORT_LABEL_KEYS.has(column.key)
        ? { ...base, type: 'status' }
        : { ...base, type: 'name', grow: true, wrap: true };
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
  const signed = signedColumnsOf(id);
  const steps = stepsOf(id, columns);
  return columns.flatMap((column, index) =>
    shownAt(steps, column.key, step)
      ? [
          listColumnOf<ReportRowView>(
            column,
            index,
            (rowKind) => emptyNameMessageId(id, rowKind),
            signed.has(column.key),
            // The buyers' list writes each sale line as the client's own print does (R5).
            id === 'R5',
          ),
        ]
      : [],
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
  signed: ReadonlySet<string> = NO_KEYS,
): { labelField: string; values: Record<string, string> } {
  const values: Record<string, string> = {};
  columns.forEach((column, index) => {
    const cell = totals.cells[index];
    if (cell === undefined || cell.type === 'empty') return;
    values[column.key] = cellText(cell, context);
    if (cell.type === 'balance') values[`${column.key}:unit`] = sideText(cell, context);
    if (signed.has(column.key)) {
      values[`${column.key}:unit`] = resultWord(cell, context);
      const tone = resultTone(cell);
      if (tone !== undefined) values[`${column.key}:tone`] = tone;
    }
  });
  const first = columns[0]?.key ?? '';
  // A report with nothing to total (the day summary R34, the vouchers list R15 with no kind
  // picked) has no totals row: a row that says "Total" over empty places would be wrong.
  if (Object.keys(values).length === 0) return { labelField: first, values };
  values[first] = values[first] ?? context.text(totals.labelId);
  return { labelField: first, values };
}
