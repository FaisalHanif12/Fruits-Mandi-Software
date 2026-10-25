// The print data of an analytics view as the printed table the shared report page draws (PRD
// FR-18.4, FR-19.13): the same header, filters, rows, totals row and "Page x of y" as a report. The
// backend wrote every figure in the print language; nothing is worked out here.

import type { AnalyticsPrintData } from '../../../bindings/AnalyticsPrintData';
import type { AnalyticsPrintHeading } from '../../../bindings/AnalyticsPrintHeading';
import type { PrintHeading, PrintTable } from '../../reports';

function headingOf(heading: AnalyticsPrintHeading | null): PrintHeading | null {
  if (heading === null) return null;
  switch (heading.kind) {
    case 'day':
    case 'year':
    case 'item':
      return { kind: 'text', text: heading.text };
    case 'month':
      return { kind: 'month', year: heading.year, month: heading.month };
    case 'other_items':
      return { kind: 'other_items' };
    case 'partner':
      return { kind: 'partner', text: heading.text };
  }
}

export function printTableOf(data: AnalyticsPrintData): PrintTable {
  return {
    language: data.language,
    paper: data.paper,
    title_id: data.title_id,
    filters: data.filters,
    columns: data.columns.map((column) => ({
      key: column.key,
      title_id: column.title_id,
      kind: column.kind,
    })),
    headings: data.columns.map((column) => headingOf(column.heading)),
    rows: data.rows,
    row_count: data.row_count,
    totals_label_id: data.totals_label_id,
    totals: data.totals,
  };
}
