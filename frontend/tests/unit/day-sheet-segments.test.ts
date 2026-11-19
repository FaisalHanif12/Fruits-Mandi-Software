// Where the short buyers' and sellers' tables of a day stand among the lines of the day total sheet
// (R34, PRD v1.12): after the sellers' lines of each day, before the cash lines. The function only
// orders pieces the backend made; it reads no money.

import { describe, expect, it } from 'vitest';
import type { PrintReportTable } from '../../src/bindings/PrintReportTable';
import type { ReportPrintCell } from '../../src/bindings/ReportPrintCell';
import { daySheetSegments } from '../../src/features/reports/print/daySheet';

function row(day: string, id: string): { cells: ReportPrintCell[]; kind: 'normal' } {
  return {
    cells: [
      { type: 'date', text: day },
      { type: 'label', id },
    ],
    kind: 'normal',
  };
}

function table(day: string, key: 'buyers' | 'sellers'): PrintReportTable {
  return {
    key,
    day,
    day_iso: day.split('-').reverse().join('-'),
    title_id: `report-r34-table-${key}`,
    columns: [],
    rows: [],
    totals_label_id: 'report-total',
    totals: [],
  };
}

const ids = (rows: { cells: ReportPrintCell[] }[]) =>
  rows.map((entry) => {
    const cell = entry.cells[1];
    return cell?.type === 'label' ? cell.id : '';
  });

describe('daySheetSegments', () => {
  it('is one piece of every row when there are no short tables', () => {
    const rows = [row('10-10-2026', 'a'), row('10-10-2026', 'b')];
    expect(daySheetSegments(rows, [])).toEqual([{ rows, startsDay: true }]);
  });

  it('puts the two tables of a day after its sellers’ lines and before its cash lines', () => {
    const rows = [
      row('10-10-2026', 'report-r34-bikri'),
      row('10-10-2026', 'report-r34-sellers-paid'),
      row('10-10-2026', 'report-r34-cash-opening'),
    ];
    const tables = [table('10-10-2026', 'buyers'), table('10-10-2026', 'sellers')];
    const segments = daySheetSegments(rows, tables);
    expect(segments).toHaveLength(3);
    const [first, middle, last] = segments;
    expect(first && 'rows' in first && ids(first.rows)).toEqual([
      'report-r34-bikri',
      'report-r34-sellers-paid',
    ]);
    expect(first && 'rows' in first && first.startsDay).toBe(true);
    expect(middle && 'tables' in middle && middle.tables.map((one) => one.key)).toEqual([
      'buyers',
      'sellers',
    ]);
    expect(last && 'rows' in last && ids(last.rows)).toEqual(['report-r34-cash-opening']);
    // The cash lines go on under the tables: they do not start the day again with a new head.
    expect(last && 'rows' in last && last.startsDay).toBe(false);
  });

  it('gives each day of a range its own tables, in the order of the days', () => {
    const rows = [
      row('10-10-2026', 'report-r34-sellers-paid'),
      row('10-10-2026', 'report-r34-cash-opening'),
      row('11-10-2026', 'report-r34-sellers-paid'),
      row('11-10-2026', 'report-r34-cash-opening'),
    ];
    const tables = [
      table('10-10-2026', 'buyers'),
      table('10-10-2026', 'sellers'),
      table('11-10-2026', 'buyers'),
      table('11-10-2026', 'sellers'),
    ];
    const order = daySheetSegments(rows, tables).map((segment) =>
      'tables' in segment
        ? segment.tables.map((one) => one.day).join(',')
        : `rows:${String(segment.startsDay)}`,
    );
    expect(order).toEqual([
      'rows:true',
      '10-10-2026,10-10-2026',
      'rows:false',
      'rows:true',
      '11-10-2026,11-10-2026',
      'rows:false',
    ]);
  });

  it('puts the tables at the end of a day that has no sellers’ line, and never loses one', () => {
    const rows = [row('10-10-2026', 'report-r34-bikri')];
    const tables = [table('10-10-2026', 'buyers'), table('12-10-2026', 'sellers')];
    const segments = daySheetSegments(rows, tables);
    expect(
      segments.map((segment) => ('tables' in segment ? segment.tables.length : 'rows')),
    ).toEqual(['rows', 1, 1]);
  });
});
