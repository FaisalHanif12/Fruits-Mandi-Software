// The print of پچھلی بکری as the table the shared report print draws (plan 72 P8, PRD 12.7, FR-19.13).
// The backend sends today's section and each day of the window with the rows of the P9 columns and the
// count and خام بکری of the day, all as printed text. This file only puts them in one table: a heading
// row for each section (its words under the sellers column, its خام بکری under the خام بکری column), then
// its rows. A day with no بکری is its heading alone, which says so. The words of a heading are Fluent
// messages with the date and the count as arguments, the same ones the Excel sheet uses. Nothing is added
// up or formatted here (CLAUDE.md rule 1).

import type { PrintReportColumn } from '../../../bindings/PrintReportColumn';
import type { PrintReportRow } from '../../../bindings/PrintReportRow';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { PrintTable } from '../../reports';
import type { BikriListPrintData } from '../../../bindings/BikriListPrintData';
import type { BikriListPrintDay } from '../../../bindings/BikriListPrintDay';

/** The message IDs of the headings (print.ftl). */
export const PAST_TODAY_HEADING_ID = 'bikri-list-today-heading';
export const PAST_DAY_HEADING_ID = 'bikri-list-day-heading';
export const PAST_DAY_EMPTY_HEADING_ID = 'bikri-list-day-heading-empty';

const EMPTY: ReportPrintCell = { type: 'empty' };

/** The place of a column by its key, or null. */
function placeOf(columns: readonly PrintReportColumn[], keys: readonly string[]): number | null {
  const at = columns.findIndex((column) => keys.includes(column.key));
  return at === -1 ? null : at;
}

/** A row of empty cells with the given cells put at their columns. */
function rowWith(
  columns: readonly PrintReportColumn[],
  put: readonly [number | null, ReportPrintCell][],
  kind: PrintReportRow['kind'],
): PrintReportRow {
  const cells = columns.map((): ReportPrintCell => EMPTY);
  for (const [place, cell] of put) {
    if (place !== null && place < cells.length) cells[place] = cell;
  }
  return { cells, kind };
}

const text = (name: string, value: string) => ({
  name,
  value: { type: 'text' as const, text: value },
});

/**
 * The heading row of a section. The words go under the sellers column (the widest column of words),
 * or the first column when the data has none; the خام بکری goes under its own column. A day with no
 * بکری has the words of an empty day and no figure.
 */
function headingRow(
  columns: readonly PrintReportColumn[],
  section: BikriListPrintDay,
  isToday: boolean,
): PrintReportRow {
  const words = placeOf(columns, ['sellers', 'seller']) ?? 0;
  const empty = !isToday && section.rows.length === 0;
  const message: ReportPrintCell = {
    type: 'message',
    id: isToday ? PAST_TODAY_HEADING_ID : empty ? PAST_DAY_EMPTY_HEADING_ID : PAST_DAY_HEADING_ID,
    args: empty
      ? [text('date', section.date)]
      : [text('date', section.date), text('count', section.count)],
  };
  return rowWith(
    columns,
    [
      [words, message],
      [empty ? null : placeOf(columns, ['gross']), { type: 'money', text: section.gross }],
    ],
    'group',
  );
}

/** The heading of a section and its rows (none for an empty day: its heading says so). */
function sectionRows(
  columns: readonly PrintReportColumn[],
  section: BikriListPrintDay,
  isToday: boolean,
): PrintReportRow[] {
  return [headingRow(columns, section, isToday), ...section.rows];
}

/**
 * The whole print as one table: the heading and the rows of today's section (when there is one), then of
 * each day of the window, newest first, and the totals of the filter under the columns.
 */
export function pastPrintTable(data: BikriListPrintData): PrintTable {
  const { columns } = data;
  const rows: PrintReportRow[] = [
    ...(data.today === null ? [] : sectionRows(columns, data.today, true)),
    ...data.days.flatMap((day) => sectionRows(columns, day, false)),
  ];
  return {
    language: data.language,
    paper: data.paper,
    title_id: data.title_id,
    filters: data.filters,
    columns,
    rows,
    row_count: data.row_count,
    totals_label_id: data.totals_label_id,
    totals: data.totals,
  };
}

/** True when there is no بکری to print in any section. */
export function pastPrintIsEmpty(data: BikriListPrintData): boolean {
  return (data.today?.rows.length ?? 0) === 0 && data.days.every((day) => day.rows.length === 0);
}
