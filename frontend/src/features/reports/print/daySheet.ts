// Where the short buyers' and sellers' tables of the day total sheet (R34) stand among its lines
// (PRD R34, v1.12): after the sellers' lines of each day, before the cash lines. This only decides the
// ORDER of pieces the backend already made; it adds up nothing and reads no money (CLAUDE.md rule 1).
// A day is the run of rows with the same date cell. A table whose day has no sellers' line is put at
// the end of that day, and a table whose day has no row at all goes last, so nothing is ever lost.

import type { PrintReportTable } from '../../../bindings/PrintReportTable';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';

/** The line after which the two short tables of a day stand (the last of the sellers' lines). */
export const DAY_TABLES_AFTER_ID = 'report-r34-sellers-paid';

/** A piece of rows of the main table, or the short tables that stand between two pieces. */
export type DaySegment<R> =
  | {
      rows: R[];
      /** True for the first piece of a day: it has the head row of the columns. */
      startsDay: boolean;
    }
  | { tables: PrintReportTable[] };

interface RowLike {
  cells: readonly ReportPrintCell[];
}

/** The date cell of a row as printed, or null when the first cell is not a date. */
function dayOf(row: RowLike): string | null {
  const first = row.cells[0];
  return first?.type === 'date' ? first.text : null;
}

/** True when the row is the line the day's short tables follow. */
function isAnchor(row: RowLike): boolean {
  return row.cells.some((cell) => cell.type === 'label' && cell.id === DAY_TABLES_AFTER_ID);
}

/** The runs of rows with one date, in order. A row with no date joins the run before it. */
function daysOf<R extends RowLike>(rows: readonly R[]): { day: string | null; rows: R[] }[] {
  const days: { day: string | null; rows: R[] }[] = [];
  for (const row of rows) {
    const day = dayOf(row);
    const last = days.at(-1);
    if (last !== undefined && (day === null || day === last.day)) last.rows.push(row);
    else days.push({ day, rows: [row] });
  }
  return days;
}

/**
 * The pieces of a print in order. With no short tables it is one piece of all the rows, as every
 * report other than the day total sheet is printed.
 */
export function daySheetSegments<R extends RowLike>(
  rows: readonly R[],
  tables: readonly PrintReportTable[],
): DaySegment<R>[] {
  if (tables.length === 0) return [{ rows: [...rows], startsDay: true }];
  const segments: DaySegment<R>[] = [];
  const placed = new Set<PrintReportTable>();
  const tablesOf = (day: string | null): PrintReportTable[] => {
    const found = tables.filter((table) => table.day === day && !placed.has(table));
    for (const table of found) placed.add(table);
    return found;
  };
  for (const { day, rows: dayRows } of daysOf(rows)) {
    const at = dayRows.findIndex(isAnchor);
    const dayTables = tablesOf(day);
    if (at === -1 || dayTables.length === 0) {
      segments.push({ rows: dayRows, startsDay: true });
      if (dayTables.length > 0) segments.push({ tables: dayTables });
      continue;
    }
    segments.push({ rows: dayRows.slice(0, at + 1), startsDay: true });
    segments.push({ tables: dayTables });
    const rest = dayRows.slice(at + 1);
    if (rest.length > 0) segments.push({ rows: rest, startsDay: false });
  }
  const left = tables.filter((table) => !placed.has(table));
  if (left.length > 0) segments.push({ tables: left });
  return segments;
}
