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
import { listColumnOf, monthText, type ReportRowView } from '../reports';
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
  }
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
  H: ['rank', 'name', 'total_comm', 'share', 'qty', 'compare', 'change'],
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
  H: ['rank', 'name'],
};

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
  if (headed.length > 0) {
    const fixed = columns.flatMap((column, index) =>
      column.heading === null ? [{ column, index }] : [],
    );
    const fixedWidth = fixed.reduce(
      (sum, { column }) => sum + columnWidth(column.kind, column.key),
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
    width: columnWidth(column.kind, column.key),
  }));
  const priority = PRIORITY[view];
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

/** The grid's columns for the columns of the plan, in the backend's order. */
export function analyticsColumns(
  columns: readonly AnalyticsColumn[],
  plan: ColumnPlan,
): ListColumn<AnalyticsRowView>[] {
  return columns.flatMap((column, index) => {
    if (!plan.visible.has(index)) return [];
    const base = listColumnOf<AnalyticsRowView>(column, index);
    const heading = column.heading;
    // The rank of H shares its column with the headings of the three lists, which are words.
    const named: ListColumn<AnalyticsRowView> =
      column.key === 'rank' ? { ...base, type: 'short', wrap: true } : base;
    return [
      heading === null
        ? named
        : {
            ...named,
            headerText: (context: ListContext) => headingText(heading, context),
          },
    ];
  });
}

/** The columns of the lines behind a number, most useful first (the grid is not windowed). */
const LINES_PRIORITY: readonly string[] = [
  'date',
  'buyer',
  'item',
  'qty',
  'total_comm',
  'doc',
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
