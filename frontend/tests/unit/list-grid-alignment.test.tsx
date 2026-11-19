// Plan 39 (NFR-USE-19 v1.6, Code Standards F17 to F19): what the list grid does so that every column
// stands under its heading. jsdom has no layout, so this checks the marks the stylesheet and the
// alignment check (tests/e2e/alignment/measure.mjs) read: a number column has `cell-num` on its cells
// and `header-num` on its heading (centred in grid.css), a text column has neither (it starts at the
// start edge), the word of the totals row stands in the first text column and never in a column of
// figures (R6), a list whose rows grow with a long column puts the other cells in the middle of the
// row (R7), and grid.css keeps the figures, their heading and their total centred in both languages.

import { waitFor } from '@testing-library/react';
import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';
import { useLanguageStore } from '../../src/shared/i18n';
import { renderIn, visible } from '../helpers/render';

interface Line {
  id: string;
  date: string;
  name: string;
  amount: string;
  note: string;
}

const COLUMNS: readonly ListColumn<Line>[] = [
  { field: 'date', type: 'date', headerId: 'report-col-date', value: (row) => row.date },
  { field: 'name', type: 'name', headerId: 'report-col-detail', value: (row) => row.name },
  { field: 'amount', type: 'money', headerId: 'report-col-balance', value: (row) => row.amount },
  {
    field: 'note',
    type: 'text',
    headerId: 'report-col-detail',
    value: (row) => row.note,
    wrap: true,
  },
];

const ROWS: Line[] = [
  { id: 'a', date: '10-10-2026', name: 'Akram', amount: '12,880', note: 'Banana 20 at 600' },
  { id: 'b', date: '11-10-2026', name: 'Arshad', amount: '250', note: 'Apple 5' },
];

function draw(language: Language, columns: readonly ListColumn<Line>[]) {
  useLanguageStore.setState({ language });
  return renderIn(
    language,
    <ListGrid<Line>
      columns={columns}
      rows={ROWS}
      getRowId={(row) => row.id}
      labelId="report-grid-label"
      onOpen={() => undefined}
      heightMode="auto"
      totals={() => ({
        // The screen names the date column, a column of figures, as the place of the word.
        labelField: 'date',
        values: { date: 'Total', amount: '13,130' },
      })}
    />,
  );
}

const cell = (rowId: string, colId: string) =>
  document.querySelector(`.ag-row[row-id="${rowId}"] [col-id="${colId}"]`);
const heading = (colId: string) => document.querySelector(`.ag-header-cell[col-id="${colId}"]`);

describe.each(LANGUAGES)('the list grid lines its columns up (%s)', (language) => {
  it('marks a number column on its cells and on its heading, and a text column on neither (R1, R2)', async () => {
    draw(language, COLUMNS.slice(0, 3));
    await waitFor(() => {
      expect(cell('a', 'amount')).not.toBeNull();
    });
    for (const field of ['date', 'amount']) {
      expect(cell('a', field), field).toHaveClass('cell-num');
      expect(heading(field), field).toHaveClass('header-num');
    }
    expect(cell('a', 'name')).not.toHaveClass('cell-num');
    expect(heading('name')).not.toHaveClass('header-num');
  });

  it('puts the word of the totals row in the first text column, not in a column of figures (R6)', async () => {
    draw(language, COLUMNS.slice(0, 3));
    await waitFor(() => {
      expect(document.querySelector('.ag-row-pinned [col-id="amount"]')).not.toBeNull();
    });
    const pinned = (colId: string) =>
      visible(document.querySelector(`.ag-row-pinned [col-id="${colId}"]`)?.textContent);
    expect(pinned('date')).toBe('');
    expect(pinned('name')).toBe('Total');
    expect(pinned('amount')).toBe('13,130');
  });

  it('puts the word where the screen asked when that column is text already (R6)', async () => {
    draw(language, COLUMNS.slice(1, 3));
    await waitFor(() => {
      expect(document.querySelector('.ag-row-pinned [col-id="amount"]')).not.toBeNull();
    });
    // The date column is not shown here, so the word has no figure column to leave.
    expect(visible(document.querySelector('.ag-row-pinned [col-id="name"]')?.textContent)).toBe('');
  });

  it('puts every other cell in the middle of a row that grows with a long column (R7)', async () => {
    draw(language, COLUMNS);
    await waitFor(() => {
      expect(cell('a', 'amount')).not.toBeNull();
    });
    expect(document.querySelector('.list-grid')).toHaveClass('list-grid-wrap');
    for (const field of ['date', 'name', 'amount', 'note']) {
      expect(cell('a', field)?.querySelector('.cell-fill'), field).not.toBeNull();
    }
    // The words keep their spaces: a whole text is one element inside the centred one.
    expect(visible(cell('a', 'note')?.textContent)).toBe('Banana 20 at 600');
  });

  it('leaves a list without a growing column as it was', async () => {
    draw(language, COLUMNS.slice(0, 3));
    await waitFor(() => {
      expect(cell('a', 'amount')).not.toBeNull();
    });
    expect(document.querySelector('.list-grid')).not.toHaveClass('list-grid-wrap');
    expect(document.querySelector('.cell-fill')).toBeNull();
  });
});

describe('the stylesheet of the grids (plan 39)', () => {
  const css = fs.readFileSync(
    path.resolve(import.meta.dirname, '../../src/shared/grid/grid.css'),
    'utf8',
  );
  const rule = (selector: string): string => {
    const start = css.indexOf(`${selector} {`);
    if (start === -1) throw new Error(`No rule ${selector}`);
    return css.slice(start, css.indexOf('}', start));
  };

  it('centres the figures of a number cell, written left to right, in both languages (R1)', () => {
    const cellRule = rule('.entry-grid .cell-num,\n.entry-grid .cell-num input');
    expect(cellRule).toContain('direction: ltr;');
    expect(cellRule).toContain('text-align: center;');
    expect(cellRule).toContain('font-variant-numeric: tabular-nums;');
    expect(cellRule).not.toContain('text-align: end;');
  });

  it('centres the heading of a number column and of a small column (R1, R3)', () => {
    const head = rule(
      '.entry-grid .header-num .ag-header-cell-label,\n.entry-grid .header-center .ag-header-cell-label',
    );
    expect(head).toContain('justify-content: center;');
    expect(css).toMatch(
      /\.entry-grid \.header-num \.ag-header-cell-text,\s*\.entry-grid \.header-center \.ag-header-cell-text \{\s*text-align: center;/,
    );
  });

  it('centres a small column (a choice, a tick, an icon) (R3)', () => {
    expect(rule('.entry-grid .cell-center')).toContain('text-align: center;');
  });

  it('keeps a figure with its word in the middle of the column, the word after it (R1)', () => {
    const unit = rule('.entry-grid .cell-with-unit');
    expect(unit).toContain('grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);');
    expect(rule('.entry-grid .cell-with-unit > :first-child')).toContain('grid-column: 2;');
    expect(rule('.entry-grid .cell-unit')).toContain('grid-column: 3;');
  });

  it('has no right-edge rule for figures left (the rule of plan 39 v1.1 replaced it)', () => {
    expect(css).not.toMatch(/\.cell-num[^{]*\{[^}]*justify-content: flex-end/);
    expect(css).not.toContain('header-unit');
  });
});
