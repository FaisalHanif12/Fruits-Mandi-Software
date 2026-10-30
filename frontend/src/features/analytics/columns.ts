// The columns of an analytics view as the list grid wants them (PRD 18.2, NFR-USE-15, Code Standards
// F18, F22). The backend sends the columns with a kind, like a report: the kind picks the grid's
// column type. A column that stands for a day, a month, a year or an item has its heading written
// here from the backend's heading (the month by its name, an item by its name in the app language).
// A table never scrolls sideways, so the columns shown are the ones that fit (fit.ts), most useful
// first; in views D and E, with one column for each period or item, a window of them is shown.

import type { AnalyticsColumn } from '../../bindings/AnalyticsColumn';
import type { AnalyticsHeading } from '../../bindings/AnalyticsHeading';
import type { AnalyticsRow } from '../../bindings/AnalyticsRow';
import type { AnalyticsView } from '../../bindings/AnalyticsView';
import type { ReportColumn } from '../../bindings/ReportColumn';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn, ListContext } from '../../shared/grid';
import { LIST_COLUMN_WIDTHS } from '../../shared/grid/columnTypes';
import type { LayoutStep } from '../../shared/hooks/useLayoutStep';
import { displayName } from '../../shared/i18n';
import { listColumnOf, monthText, type ReportRowView, type SignMode } from '../reports';
import { columnWidth, fitColumns, STEP_ROOM, windowOf, type ColumnWindow } from './fit';

/** One row of the grid: its place in the whole view is its key. */
export interface AnalyticsRowView {
  key: string;
  row: AnalyticsRow;
}

/** The words of the heading of a column that stands for a day, a month, a year or an item. */
export function headingText(heading: AnalyticsHeading, context: ListContext): string {
  switch (heading.kind) {
    case 'day':
      return isoToDisplayDate(heading.iso);
    case 'month':
      return monthText(context.text, heading.year, heading.month);
    case 'year':
      return String(heading.year);
    case 'item':
      return displayName(
        { name_ur: heading.names.ur, name_en: heading.names.en },
        context.language,
      );
    case 'other_items':
      return context.text('analytics-col-other-items');
    case 'partner':
      return displayName(
        { name_ur: heading.names.ur, name_en: heading.names.en },
        context.language,
      );
  }
}

/**
 * The heading of a headed column as the grid shows it. A month, a day or an item stands for the
 * column by itself; a partner's column needs whose it is and then what it holds ("Partner A: Share"),
 * because a trip has the same two columns for each partner (J).
 */
function headerOf(
  column: AnalyticsColumn,
  heading: AnalyticsHeading,
  context: ListContext,
): string {
  const name = headingText(heading, context);
  return heading.kind === 'partner'
    ? context.text('analytics-col-partner-heading', { name, title: context.text(column.title_id) })
    : name;
}

/**
 * True when the figure of this column can be a loss: a trip's result and each partner's part of it
 * (J), and the trips' result of a day, month or year (G). It is red, with its sign and a word
 * ("Loss", "Profit"), never colour alone (NFR-USE-12, PRD E50). Other figures keep a plain sign.
 */
function isSigned(view: AnalyticsView, column: AnalyticsColumn): boolean {
  if (column.kind !== 'money') return false;
  if (view === 'J') return column.key === 'result' || column.heading?.kind === 'partner';
  return view === 'G' && column.key === 'trips_result';
}

/**
 * How a column shows its result. View I (gardens) writes "Profit" or "Loss" only on the row of a
 * closed garden, for the garden's result and each partner's part (PRD FR-12.38); an open garden keeps
 * its sign only. The totals row has no word, because it adds open and closed gardens.
 */
function signModeOf(view: AnalyticsView, column: AnalyticsColumn): SignMode {
  if (isSigned(view, column)) return true;
  const result = column.key === 'result' || column.heading?.kind === 'partner';
  return view === 'I' && column.kind === 'money' && result ? 'closed' : false;
}

/** The keys of the columns of a page that can be a loss, for the totals row. */
export function signedKeysOf(
  view: AnalyticsView,
  columns: readonly AnalyticsColumn[],
): ReadonlySet<string> {
  return new Set(columns.filter((column) => isSigned(view, column)).map((column) => column.key));
}

/**
 * The columns of each view, most useful first (PRD 18.2): the columns shown at a step are the first
 * of this list that fit. A column the list does not name comes last. Views D and E are windowed.
 */
const PRIORITY: Readonly<Record<AnalyticsView, readonly string[]>> = {
  A: [
    'item',
    'total_comm',
    'compare',
    'change',
    'share',
    'qty',
    'gross',
    'seller_comm',
    'buyer_comm',
    'laga',
    'avg_rate',
  ],
  B: [
    'seller',
    'total_comm',
    'compare',
    'change',
    'net',
    'gross',
    'bikri_count',
    'qty',
    'seller_comm',
    'buyer_comm',
  ],
  C: [
    'buyer',
    'purchase_value',
    'dues_now',
    'compare',
    'change',
    'buyer_comm',
    'laga',
    'qty',
    'avg_days',
  ],
  D: [],
  E: [],
  // Partners (F): his profit and what he gained or lost in all, then where it came from: the shares of
  // the commission and the expenses, then his gardens, trips and fruit side by side (PRD 18.2 F), which
  // open the lines behind them (`drill`). What is left over (`other`) comes last.
  F: [
    'partner',
    'profit_share',
    'net_change',
    'income_share',
    'expense_share',
    'gardens',
    'trips',
    'fruit',
    'other',
  ],
  // Trend (G): the period and the profit first, then what the profit is made of.
  G: ['period', 'profit', 'commission', 'gross', 'shared_income', 'shop_expenses', 'trips_result'],
  H: ['rank', 'name', 'total_comm', 'share', 'qty', 'compare', 'change'],
  // Gardens (I): the garden, its result and the sales, then what was paid and the commission. The
  // partner columns are inserted from the page itself (`priorityOf`).
  I: [
    'group',
    'status',
    'result',
    'sales_our_shop',
    'sales_other_shops',
    'commission',
    'paid_shop',
    'paid_pocket',
    'other_in',
  ],
  // Trips (J): the name, the result and where it came from. The partner columns are inserted
  // from the page itself (`priorityOf`), because a trip has one pair for each partner.
  J: [
    'group',
    'result',
    'sales_money',
    'commission',
    'bought_cost',
    'sold_qty',
    'left_qty',
    'cost_sold',
    'bought_qty',
    'written_off_qty',
    'written_off_cost',
    'left_cost',
  ],
  // Fruit taken (K): who it was for and the item, the value and the quantity, then where it came from.
  K: ['for', 'item', 'value', 'qty', 'own_shop', 'other_shops'],
};

/**
 * The columns that name the row. They stay when the user asks for the other columns, so a figure
 * is never shown without the item, seller or buyer it belongs to.
 */
const ANCHORS: Readonly<Record<AnalyticsView, readonly string[]>> = {
  A: ['item'],
  B: ['seller'],
  C: ['buyer'],
  D: [],
  E: [],
  F: ['partner'],
  G: ['period'],
  H: ['rank', 'name'],
  I: ['group'],
  J: ['group'],
  K: ['for', 'item'],
};

/** The views whose columns are one for each period or item: a window of them is shown (D and E). */
const WINDOWED: ReadonlySet<AnalyticsView> = new Set<AnalyticsView>(['D', 'E']);

/**
 * Where the partner columns of a view go in its priority list: his part comes right after the key in
 * `parts` and his percent after the key in `shares`, so the partners show before the least used
 * columns at the wide step. Views I (gardens) and J (trips) have a pair of columns for each partner.
 */
const PARTNER_PLACES: Partial<Record<AnalyticsView, { parts: string; shares: string }>> = {
  I: { parts: 'result', shares: 'sales_our_shop' },
  J: { parts: 'result', shares: 'sold_qty' },
};

/**
 * The columns of a view, most useful first, for this page. Views I and J have a pair of columns for
 * each partner (his percent and his part of the result), put in place by `PARTNER_PLACES`.
 */
function priorityOf(view: AnalyticsView, columns: readonly AnalyticsColumn[]): readonly string[] {
  const base = PRIORITY[view];
  const place = PARTNER_PLACES[view];
  if (place === undefined) return base;
  const partner = columns.filter((column) => column.heading?.kind === 'partner');
  const parts = partner.filter((column) => column.kind === 'money').map((column) => column.key);
  const shares = partner.filter((column) => column.kind !== 'money').map((column) => column.key);
  const afterParts = base.indexOf(place.parts) + 1;
  const afterShares = base.indexOf(place.shares) + 1;
  return [
    ...base.slice(0, afterParts),
    ...parts,
    ...base.slice(afterParts, afterShares),
    ...shares,
    ...base.slice(afterShares),
  ];
}

export interface ColumnPlan {
  /** The columns shown, by their place in the backend's list (a cell is read at that place). */
  visible: ReadonlySet<number>;
  /** The window of the headed columns in views D and E, or null when they all fit (or none). */
  window: ColumnWindow | null;
  /**
   * Set when this window cannot hold every column of the view: how many are shown now and how many
   * there are. The screen says so and offers the other columns (PRD FR-7.2, Code Standards F22), so
   * every column of PRD 18.2 can be read on screen at every size. Null when all of them fit, and
   * for views D and E, whose columns are all reachable with the window buttons.
   */
  trimmed: { shown: number; total: number } | null;
}

/**
 * Which columns of a page are shown at the layout step, and the window of headed ones. With `other`
 * on, the columns that do not fit by default come in and the least needed ones step aside (the
 * same idea as the "More columns" tick of the lists, shared/grid/moreColumns.ts).
 *
 * Example (view A at 1,366 px): by default the average rate waits; with `other` on it is shown and
 * the لاگا column gives up its place.
 */
export function planColumns(
  view: AnalyticsView,
  columns: readonly AnalyticsColumn[],
  step: LayoutStep,
  windowStart: number,
  other = false,
): ColumnPlan {
  const room = STEP_ROOM[step];
  const headed = columns.flatMap((column, index) => (column.heading === null ? [] : [index]));
  if (headed.length > 0 && WINDOWED.has(view)) {
    const fixed = columns.flatMap((column, index) =>
      column.heading === null ? [{ column, index }] : [],
    );
    const fixedWidth = fixed.reduce(
      (sum, { column }) =>
        sum + columnWidth(column.kind, column.key, signModeOf(view, column) !== false),
      0,
    );
    const window = windowOf(
      headed.length,
      fixedWidth,
      LIST_COLUMN_WIDTHS.figure,
      room,
      windowStart,
    );
    const shownHeaded = headed.slice(window.start, window.start + window.count);
    return {
      visible: new Set([...fixed.map(({ index }) => index), ...shownHeaded]),
      window: window.count < window.total ? window : null,
      trimmed: null,
    };
  }
  const items = columns.map((column) => ({
    key: column.key,
    width: columnWidth(column.kind, column.key, signModeOf(view, column) !== false),
  }));
  const priority = priorityOf(view, columns);
  const byDefault = fitColumns(items, priority, room);
  const waiting = columns.map((column) => column.key).filter((key) => !byDefault.has(key));
  let keep = byDefault;
  if (other && waiting.length > 0) {
    const anchors = ANCHORS[view];
    // The row's name first, then the columns that were waiting, then the rest in the usual order. A
    // column the priority list does not name is ranked last by `fitColumns` whatever happens here.
    keep = fitColumns(
      items,
      [
        ...anchors,
        ...priority.filter((key) => waiting.includes(key) && !anchors.includes(key)),
        ...priority,
      ],
      room,
    );
  }
  return {
    visible: new Set(columns.flatMap((column, index) => (keep.has(column.key) ? [index] : []))),
    window: null,
    trimmed: waiting.length === 0 ? null : { shown: keep.size, total: columns.length },
  };
}

/**
 * The buyers' view (C) works out the dues and the days to pay for the page only, so the backend
 * sorts it by these columns alone (plan 38 R4). Every column of another view can be sorted.
 */
const SORTABLE_IN_C: ReadonlySet<string> = new Set([
  'buyer',
  'qty',
  'purchase_value',
  'buyer_comm',
  'laga',
]);

function isSortable(view: AnalyticsView, key: string): boolean {
  return view !== 'C' || SORTABLE_IN_C.has(key);
}

/** The grid's columns for the columns of the plan, in the backend's order. */
export function analyticsColumns(
  view: AnalyticsView,
  columns: readonly AnalyticsColumn[],
  plan: ColumnPlan,
): ListColumn<AnalyticsRowView>[] {
  return columns.flatMap((column, index) => {
    if (!plan.visible.has(index)) return [];
    const base = listColumnOf<AnalyticsRowView>(
      column,
      index,
      undefined,
      signModeOf(view, column),
      false,
      isSortable(view, column.key),
    );
    const heading = column.heading;
    // The rank of H shares its column with the headings of the three lists, which are words.
    // The days to pay (C) has a long heading ("Days to pay (average)"): it takes the room of a figure,
    // so the heading has two lines at most (plan 39 R4).
    const named: ListColumn<AnalyticsRowView> =
      column.key === 'rank'
        ? { ...base, type: 'short', wrap: true }
        : column.key === 'avg_days'
          ? { ...base, type: 'figure' }
          : base;
    return [
      heading === null
        ? named
        : {
            ...named,
            headerText: (context: ListContext) => headerOf(column, heading, context),
          },
    ];
  });
}

/**
 * The columns of the lines behind a number, most useful first (the grid is not windowed). The fruit
 * lines (view K) are the columns of the fruit register: who it was for and the value come early, the
 * rate and where it came from after the document.
 */
const LINES_PRIORITY: readonly string[] = [
  'date',
  'buyer',
  'for',
  'item',
  'kind',
  'qty',
  'value',
  'total_comm',
  'amount',
  'doc',
  'rate',
  'source',
  'share',
  'base',
  'gross',
  'seller',
  'seller_comm',
  'buyer_comm',
  'laga',
  'buyer_value',
];

/** The columns of the lines, as many as fit the layout step. */
export function linesColumns(
  columns: readonly ReportColumn[],
  step: LayoutStep,
): ListColumn<ReportRowView>[] {
  const keep = fitColumns(
    columns.map((column) => ({ key: column.key, width: columnWidth(column.kind, column.key) })),
    LINES_PRIORITY,
    STEP_ROOM[step],
  );
  return columns.flatMap((column, index) =>
    keep.has(column.key) ? [listColumnOf<ReportRowView>(column, index)] : [],
  );
}
