// The print of پچھلی بکری as one table (plan 72 P8): the heading row of each section, its rows, and the
// totals, all taken from what the backend sent. The adapter works out nothing; these tests hold the
// places of the cells, the messages of the headings with their date and count, and the order of the
// sections (today first, then the window newest first).

import { describe, expect, it } from 'vitest';
import {
  PAST_DAY_EMPTY_HEADING_ID,
  PAST_DAY_HEADING_ID,
  PAST_TODAY_HEADING_ID,
  pastPrintIsEmpty,
  pastPrintTable,
} from '../../src/features/bikri/print/past-print-table';
import type { BikriListPrintInput } from '../../src/bindings/BikriListPrintInput';
import { PAST_PRINT_COLUMNS, pastPrintDataFor } from '../helpers/past-print';

const INPUT: BikriListPrintInput = {
  from: '03-10-2026',
  to: '09-10-2026',
  with_today: true,
  seller_id: null,
  item_id: null,
  status: null,
  show_finished: false,
  search: null,
  all: false,
  language: 'en',
  paper: 'a4',
};

const at = (key: string) => PAST_PRINT_COLUMNS.findIndex((column) => column.key === key);
const arg = (name: string, value: string) => ({ name, value: { type: 'text', text: value } });

describe('the table of the print of پچھلی بکری', () => {
  const table = pastPrintTable(pastPrintDataFor(INPUT));
  const headings = table.rows.filter((row) => row.kind === 'group');

  it('has today first, then the days of the window newest first, each with a heading row', () => {
    expect(headings).toHaveLength(8);
    const dates = headings.map((row) => {
      const cell = row.cells[at('sellers')];
      if (cell?.type !== 'message') return null;
      const date = cell.args.find((candidate) => candidate.name === 'date')?.value;
      return date?.type === 'text' ? date.text : null;
    });
    expect(dates).toEqual([
      '10-10-2026',
      '09-10-2026',
      '08-10-2026',
      '07-10-2026',
      '06-10-2026',
      '05-10-2026',
      '04-10-2026',
      '03-10-2026',
    ]);
  });

  it('puts the words of a heading under the sellers, and the خام بکری under its own column, as the backend said them', () => {
    const [today, nine, eight] = headings;
    expect(today?.cells[at('sellers')]).toEqual({
      type: 'message',
      id: PAST_TODAY_HEADING_ID,
      args: [arg('date', '10-10-2026'), arg('count', '1')],
    });
    expect(nine?.cells[at('sellers')]).toEqual({
      type: 'message',
      id: PAST_DAY_HEADING_ID,
      args: [arg('date', '09-10-2026'), arg('count', '2')],
    });
    expect(nine?.cells[at('gross')]).toEqual({ type: 'money', text: '58,800' });
    // The other columns of a heading row are empty.
    expect(nine?.cells[at('number')]).toEqual({ type: 'empty' });
    expect(nine?.cells[at('status')]).toEqual({ type: 'empty' });
    // A day with no بکری is its heading alone: the words of an empty day, with the date, and no figure.
    expect(eight?.cells[at('sellers')]).toEqual({
      type: 'message',
      id: PAST_DAY_EMPTY_HEADING_ID,
      args: [arg('date', '08-10-2026')],
    });
    expect(eight?.cells[at('gross')]).toEqual({ type: 'empty' });
  });

  it('keeps the rows of each day under its heading', () => {
    const kinds = table.rows.map((row) => row.kind);
    // today: heading + 1 row; 09-10: heading + 2 rows; six empty days: a heading each.
    expect(kinds).toEqual([
      'group',
      'normal',
      'group',
      'normal',
      'normal',
      ...Array.from({ length: 6 }, () => 'group'),
    ]);
  });

  it('carries the title, the facts, the columns and the totals of the backend as they are', () => {
    const data = pastPrintDataFor(INPUT);
    expect(table).toMatchObject({
      language: 'en',
      paper: 'a4',
      title_id: data.title_id,
      filters: data.filters,
      columns: data.columns,
      totals_label_id: data.totals_label_id,
      totals: data.totals,
      row_count: data.row_count,
    });
  });

  it('has no section for today when the print was asked without it', () => {
    const without = pastPrintTable(pastPrintDataFor({ ...INPUT, with_today: false }));
    const first = without.rows[0]?.cells[at('sellers')];
    expect(first).toMatchObject({ type: 'message', id: PAST_DAY_HEADING_ID });
    expect(without.rows.filter((row) => row.kind === 'group')).toHaveLength(7);
  });

  it('a data without the named columns still puts the heading in the first column', () => {
    const data = pastPrintDataFor(INPUT);
    const bare = pastPrintTable({
      ...data,
      columns: [{ key: 'x', title_id: 'bikri-list-col-number', kind: 'text' }],
      totals: [{ type: 'empty' }],
    });
    const heading = bare.rows[0];
    expect(heading?.kind).toBe('group');
    expect(heading?.cells).toHaveLength(1);
    expect(heading?.cells[0]).toMatchObject({ type: 'message', id: PAST_TODAY_HEADING_ID });
  });
});

describe('an empty print', () => {
  it('is empty when no section holds a بکری', () => {
    expect(pastPrintIsEmpty(pastPrintDataFor(INPUT, { todayRows: [], windowRows: {} }))).toBe(true);
    expect(pastPrintIsEmpty(pastPrintDataFor(INPUT))).toBe(false);
    expect(
      pastPrintIsEmpty(
        pastPrintDataFor(INPUT, { todayRows: [], windowRows: { '2026-10-04': [{ number: '9' }] } }),
      ),
    ).toBe(false);
    // Without today and with nothing in the window.
    expect(
      pastPrintIsEmpty(pastPrintDataFor({ ...INPUT, with_today: false }, { windowRows: {} })),
    ).toBe(true);
  });
});
