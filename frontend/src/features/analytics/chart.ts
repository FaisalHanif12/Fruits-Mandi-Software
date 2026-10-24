// The chart above a view's table (PRD FR-18.3): drawn from the rows the screen already has; the
// backend sends no chart data. The heights are numbers for the drawing (`chartValueOfPaisa`), and
// every figure written on the chart is the text of the cell the backend sent (`cellText`), so the
// chart adds up nothing and can never disagree with the table under it (CLAUDE.md rule 1).
//
// What each view draws: A the total commission of each item, B of each seller, C the purchase value
// of each buyer, D one stack for each period with a part for each item, E one stack for each seller
// with a part for each item, F each partner's profit share and his net change side by side, G the
// money columns of each period side by side (many thin bars for the days of a month), H the top
// items, J the result of each trip, purchase, item or day. A chart shows the first rows (the backend
// sends the biggest first) and says how many it left out.

import type { AnalyticsColumn } from '../../bindings/AnalyticsColumn';
import type { AnalyticsPage } from '../../bindings/AnalyticsPage';
import type { AnalyticsRow } from '../../bindings/AnalyticsRow';
import type { AnalyticsView } from '../../bindings/AnalyticsView';
import type { ReportCell } from '../../bindings/ReportCell';
import { chartValueOfPaisa } from '../../shared/format';
import type { ListContext } from '../../shared/grid';
import type { BarChartData, BarSeries } from '../../shared/charts';
import { cellText, nameWithTag } from '../reports';
import { headingText } from './columns';

/** A chart shows this many rows, and this many series (the palette has six colours). */
export const MAX_CHART_ROWS = 10;
export const MAX_CHART_SERIES = 6;

export interface ChartPlan {
  data: BarChartData;
  /** The message of what the chart shows, for a screen reader and the line above it. */
  labelId: string;
  /** How many rows (or series) the chart shows, and how many there are: it says so when it cut. */
  shown: number;
  total: number;
}

/** The height of a bar for a cell: money and balances as paisa, a count or a share as it is. */
function heightOf(cell: ReportCell | undefined): number {
  if (cell === undefined) return 0;
  switch (cell.type) {
    case 'money':
    case 'balance':
      return chartValueOfPaisa(cell.paisa);
    case 'number':
      return cell.value;
    case 'percent':
      return cell.hundredths;
    default:
      return 0;
  }
}

function seriesOf(
  name: string,
  cells: readonly (ReportCell | undefined)[],
  context: ListContext,
): BarSeries {
  return {
    name,
    values: cells.map(heightOf),
    labels: cells.map((cell) => (cell === undefined ? '' : cellText(cell, context))),
  };
}

/**
 * The name under a bar. A party is written by his name with his tag, not with his code: the code is
 * long and a name is what is read in a chart (the table under it has both).
 */
function categoryText(cell: ReportCell | undefined, context: ListContext): string {
  if (cell === undefined) return '';
  return cell.type === 'party'
    ? nameWithTag(cell.party, cell.party.tag, context.language)
    : cellText(cell, context);
}

function indexOfKey(columns: readonly AnalyticsColumn[], key: string): number {
  return columns.findIndex((column) => column.key === key);
}

/** The rows that are records (a heading of H is not one), in the backend's order. */
function recordRows(rows: readonly AnalyticsRow[]): AnalyticsRow[] {
  return rows.filter((row) => row.kind === 'normal');
}

/** The rows of the first list of H (the top items): from its heading to the next heading. */
function firstList(rows: readonly AnalyticsRow[]): AnalyticsRow[] {
  const out: AnalyticsRow[] = [];
  let started = false;
  for (const row of rows) {
    if (row.kind === 'group') {
      if (started) break;
      started = true;
    } else if (started) {
      out.push(row);
    }
  }
  return out;
}

function simple(
  page: AnalyticsPage,
  rows: readonly AnalyticsRow[],
  nameKey: string,
  valueKey: string,
  seriesId: string,
  labelId: string,
  context: ListContext,
): ChartPlan | null {
  const name = indexOfKey(page.columns, nameKey);
  const value = indexOfKey(page.columns, valueKey);
  if (name < 0 || value < 0 || rows.length === 0) return null;
  const shown = rows.slice(0, MAX_CHART_ROWS);
  return {
    data: {
      categories: shown.map((row) => categoryText(row.cells[name], context)),
      series: [
        seriesOf(
          context.text(seriesId),
          shown.map((row) => row.cells[value]),
          context,
        ),
      ],
      stacked: false,
    },
    labelId,
    shown: shown.length,
    total: rows.length,
  };
}

/** One stack for each period (D) or seller (E), with a part for each item. */
function matrix(page: AnalyticsPage, view: 'D' | 'E', context: ListContext): ChartPlan | null {
  const headed = page.columns.flatMap((column, index) =>
    column.heading === null ? [] : [{ column, index }],
  );
  const rows = recordRows(page.rows);
  if (headed.length === 0 || rows.length === 0) return null;
  const labelId = view === 'D' ? 'analytics-chart-d' : 'analytics-chart-e';
  const firstColumn = 0;
  if (view === 'D') {
    // The periods are the categories; each item is a series (its row).
    const items = rows.slice(0, MAX_CHART_SERIES);
    return {
      data: {
        categories: headed.map(({ column }) =>
          column.heading === null ? '' : headingText(column.heading, context),
        ),
        series: items.map((row) => {
          const head = row.cells[firstColumn];
          return seriesOf(
            head === undefined ? '' : cellText(head, context),
            headed.map(({ index }) => row.cells[index]),
            context,
          );
        }),
        stacked: true,
      },
      labelId,
      shown: items.length,
      total: rows.length,
    };
  }
  // E: the sellers are the categories; each item is a series (its column).
  const sellers = rows.slice(0, MAX_CHART_ROWS);
  const items = headed.slice(0, MAX_CHART_SERIES);
  return {
    data: {
      categories: sellers.map((row) => categoryText(row.cells[firstColumn], context)),
      series: items.map(({ column, index }) =>
        seriesOf(
          column.heading === null ? '' : headingText(column.heading, context),
          sellers.map((row) => row.cells[index]),
          context,
        ),
      ),
      stacked: true,
    },
    labelId,
    shown: sellers.length,
    total: rows.length,
  };
}

/** True for a cell with no figure in it: the backend left it empty on purpose. */
function isNoFigure(cell: ReportCell | undefined): boolean {
  return cell === undefined || cell.type === 'empty';
}

/**
 * One group of bars for each row, with a bar for each of these columns, side by side (F, G, J). A
 * column whose cells are all empty has no bar (the garden and fruit columns of F have no figure
 * until D4R): a legend entry with no bar would only confuse. A loss is a bar below the line.
 */
function sideBySide(
  page: AnalyticsPage,
  rows: readonly AnalyticsRow[],
  nameKey: string,
  valueKeys: readonly string[],
  labelId: string,
  context: ListContext,
  dense = false,
): ChartPlan | null {
  const name = indexOfKey(page.columns, nameKey);
  if (name < 0 || rows.length === 0) return null;
  const shown = dense ? rows : rows.slice(0, MAX_CHART_ROWS);
  const series = valueKeys.flatMap((key) => {
    const at = indexOfKey(page.columns, key);
    const column = page.columns[at];
    if (column === undefined) return [];
    if (shown.every((row) => isNoFigure(row.cells[at]))) return [];
    return [
      seriesOf(
        context.text(column.title_id),
        shown.map((row) => row.cells[at]),
        context,
      ),
    ];
  });
  if (series.length === 0) return null;
  return {
    data: {
      categories: shown.map((row) => categoryText(row.cells[name], context)),
      series: series.slice(0, MAX_CHART_SERIES),
      stacked: false,
      dense,
    },
    labelId,
    shown: shown.length,
    total: rows.length,
  };
}

/** The first column of J holds the trip, the purchase, the item or the day. */
const J_NAME_KEY = 'group';

/** More days than this are drawn as many thin bars (the days of a month). */
const DENSE_AFTER = 12;

/** What a view draws above its table, or null when there is nothing to draw. */
export function chartFor(
  view: AnalyticsView,
  page: AnalyticsPage,
  context: ListContext,
): ChartPlan | null {
  switch (view) {
    case 'A':
      return simple(
        page,
        recordRows(page.rows),
        'item',
        'total_comm',
        'report-col-total-comm',
        'analytics-chart-a',
        context,
      );
    case 'B':
      return simple(
        page,
        recordRows(page.rows),
        'seller',
        'total_comm',
        'report-col-total-comm',
        'analytics-chart-b',
        context,
      );
    case 'C':
      return simple(
        page,
        recordRows(page.rows),
        'buyer',
        'purchase_value',
        'analytics-col-purchase-value',
        'analytics-chart-c',
        context,
      );
    case 'D':
    case 'E':
      return matrix(page, view, context);
    case 'F':
      return sideBySide(
        page,
        recordRows(page.rows),
        'partner',
        ['profit_share', 'net_change'],
        'analytics-chart-f',
        context,
      );
    case 'G': {
      const rows = recordRows(page.rows);
      return sideBySide(
        page,
        rows,
        'period',
        ['commission', 'shared_income', 'shop_expenses', 'profit', 'trips_result'],
        'analytics-chart-g',
        context,
        rows.length > DENSE_AFTER,
      );
    }
    case 'H':
      return simple(
        page,
        firstList(page.rows),
        'name',
        'total_comm',
        'report-col-total-comm',
        'analytics-chart-h',
        context,
      );
    case 'J':
      return sideBySide(
        page,
        recordRows(page.rows),
        J_NAME_KEY,
        ['sales_money', 'result'],
        'analytics-chart-j',
        context,
      );
  }
}
