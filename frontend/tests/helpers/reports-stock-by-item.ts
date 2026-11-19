// The stock report R10 laid out by item (PRD 12.8, 19.2): one row for each item with its quantities
// over all its lots, the biggest balance first. The backend has no canned file of this layout, so the
// test builds the page from the real column definitions of R10 (taken from the backend's fruit file),
// keeping the eight columns the backend sends for the layout. The numbers are the worked day's
// (E2E-42): Banana 50 came, 45 were sold, 2 were taken as fruit, 3 are left; Apple 20 came, 10 were
// sold, 10 are left. The backend's own test checks how the layout is worked out.

import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportColumn } from '../../src/bindings/ReportColumn';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportRow } from '../../src/bindings/ReportRow';
import { R10_FRUIT } from './reports-fruit-real';

/** The keys of the item layout, in the order the backend sends them. */
export const R10_ITEM_KEYS = [
  'item',
  'received',
  'sold',
  'written_off',
  'fruit_taken',
  'balance',
  'our_left',
  'other_left',
] as const;

function columnOf(key: string): ReportColumn {
  const column = R10_FRUIT.columns.find((candidate) => candidate.key === key);
  if (column === undefined) throw new Error(`R10 has no column ${key}`);
  return column;
}

const number = (value: number): ReportCell => ({ type: 'number', value });
const names = (ur: string, en: string): ReportCell => ({ type: 'names', names: { ur, en } });

function row(cells: ReportCell[]): ReportRow {
  // A row of this layout opens nothing (it has no source record).
  return { cells, source: null, kind: 'normal' };
}

/** Apple (10 left) comes before Banana (3 left): the biggest balance first. */
export const R10_BY_ITEM: ReportPage = {
  ...R10_FRUIT,
  columns: R10_ITEM_KEYS.map(columnOf),
  rows: [
    row([
      names('سیب', 'Apple'),
      number(20),
      number(10),
      number(0),
      number(0),
      number(10),
      number(10),
      number(0),
    ]),
    row([
      names('کیلا', 'Banana'),
      number(50),
      number(45),
      number(0),
      number(2),
      number(3),
      number(3),
      number(0),
    ]),
  ],
  total_count: 2,
  totals: [
    { type: 'empty' },
    number(70),
    number(55),
    number(0),
    number(2),
    number(13),
    number(13),
    number(0),
  ],
  filters: { ...R10_FRUIT.filters, stock_view: 'item' },
};
