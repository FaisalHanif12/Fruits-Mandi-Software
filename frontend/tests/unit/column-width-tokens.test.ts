// Plan 39 R5 (Code Standards F18): one width for each kind of column, in every grid, table and print.
// The grids read their widths from shared/grid/columnTypes.ts; the plain tables read the same numbers
// as `--col-*-w` tokens in styles/tokens.css (Tailwind classes `w-col-money` and so on). This keeps the
// two lists equal, so a money column is as wide in today's sales as in a ledger or a cash book.

import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  COLUMN_WIDTHS,
  LIST_COLUMN_WIDTHS,
  SHARED_COLUMN_WIDTHS,
} from '../../src/shared/grid/columnTypes';

const TOKENS = fs.readFileSync(
  path.resolve(import.meta.dirname, '../../src/styles/tokens.css'),
  'utf8',
);

function token(name: string): number {
  const found = new RegExp(`--${name}:\\s*(\\d+)px;`).exec(TOKENS);
  if (found?.[1] === undefined) throw new Error(`No token --${name} in tokens.css`);
  return Number(found[1]);
}

describe('the width of each kind of column', () => {
  it.each(Object.entries(SHARED_COLUMN_WIDTHS))(
    'a %s column is the same width in the grids and in the tables',
    (kind, width) => {
      expect(token(`col-${kind}-w`)).toBe(width);
    },
  );

  it('has a Tailwind width class for every token, reading the token itself', () => {
    for (const kind of Object.keys(SHARED_COLUMN_WIDTHS)) {
      expect(TOKENS).toContain(`--spacing-col-${kind}: var(--col-${kind}-w);`);
    }
  });

  it('is one width for a money column, whichever grid draws it', () => {
    expect(COLUMN_WIDTHS.money).toBe(SHARED_COLUMN_WIDTHS.money);
    expect(COLUMN_WIDTHS.calc).toBe(SHARED_COLUMN_WIDTHS.money);
    expect(LIST_COLUMN_WIDTHS.money).toBe(SHARED_COLUMN_WIDTHS.money);
    expect(LIST_COLUMN_WIDTHS.number).toBe(SHARED_COLUMN_WIDTHS.money);
  });

  it('is one width for a quantity, a date and a code, whichever grid draws it', () => {
    expect(COLUMN_WIDTHS.qty).toBe(SHARED_COLUMN_WIDTHS.qty);
    expect(LIST_COLUMN_WIDTHS.count).toBe(SHARED_COLUMN_WIDTHS.qty);
    expect(COLUMN_WIDTHS.date).toBe(SHARED_COLUMN_WIDTHS.date);
    expect(LIST_COLUMN_WIDTHS.date).toBe(SHARED_COLUMN_WIDTHS.date);
    expect(LIST_COLUMN_WIDTHS.code).toBe(SHARED_COLUMN_WIDTHS.code);
    expect(LIST_COLUMN_WIDTHS.datetime).toBe(SHARED_COLUMN_WIDTHS.datetime);
    expect(LIST_COLUMN_WIDTHS.status).toBe(SHARED_COLUMN_WIDTHS.status);
  });

  it('keeps a least width for the first column of names of a plain table, as a token (R5)', () => {
    expect(token('col-name-min-w')).toBeGreaterThanOrEqual(160);
    expect(TOKENS).toContain('--spacing-col-name: var(--col-name-min-w);');
  });

  it('has the width of the figures column of a label-and-value card as a token (R8)', () => {
    expect(token('value-col-w')).toBeGreaterThanOrEqual(120);
  });
});
