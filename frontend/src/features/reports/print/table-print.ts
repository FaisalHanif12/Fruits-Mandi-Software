// The print data of the new table prints (plan 76: the اگراہی sheet R5, the buyer dues page, the commission
// page and a partner's month statement) as the table the shared print layout draws (`ReportPrintDocument`).
// The backend sends them as `TablePrintData`: the header and footer in `frame`, the columns, every row and the
// totals already written in the print language. This only changes the shape: the title and the filters
// of the header are the frame's, a column with a name for a heading becomes a text heading. It adds up
// nothing (CLAUDE.md rule 1).

import type { ReportId } from '../../../bindings/ReportId';
import type { TablePrintData } from '../../../bindings/TablePrintData';
import type { PrintHeading, PrintTable } from './ReportPrintDocument';

/**
 * The table to draw for `data`. `report` names the report when the layout has words of its own for it
 * (R5 writes each sale line as "(item qty . rate)").
 */
export function printTableOf(data: TablePrintData, report?: ReportId): PrintTable {
  const headings: (PrintHeading | null)[] = data.column_labels.map((label) =>
    label === null ? null : { kind: 'text', text: label },
  );
  return {
    language: data.frame.language,
    paper: data.paper,
    title_id: data.frame.header.title_id,
    filters: data.frame.header.facts,
    columns: data.columns,
    ...(headings.some((heading) => heading !== null) ? { headings } : {}),
    rows: data.rows,
    row_count: data.row_count,
    totals_label_id: data.totals_label_id,
    totals: data.totals,
    words: data.words,
    ...(data.extra_totals.length > 0 ? { extra_totals: data.extra_totals } : {}),
    ...(report === undefined ? {} : { report }),
  };
}
