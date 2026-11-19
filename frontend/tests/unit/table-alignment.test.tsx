// Plan 39 (PRD FR-19.13, NFR-USE-19 v1.6, Code Standards F17 to F19): the shared table keeps its
// columns lined up by itself, in both languages. jsdom has no layout, so this checks what the
// alignment check of tests/e2e/alignment/measure.mjs relies on: the kind of each column is on its
// cells, its heading and its total, a number column is written left to right and centred, a text
// column starts at the start edge, a small column is centred, the rows are middle-aligned and a
// sized column has the width token of its kind. The measured proof (2 px) runs in a real browser on
// every screen and print: `node tests/e2e/alignment/run.mjs`.

import { describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { Table, type TableColumn } from '../../src/shared/ui';
import { message, renderIn } from '../helpers/render';

interface Row {
  id: string;
  name: string;
  qty: string;
  tick: string;
}

const ROWS: readonly Row[] = [
  { id: 'a', name: 'Akram', qty: '17', tick: 'yes' },
  { id: 'b', name: 'Arshad', qty: '250', tick: 'no' },
];

const COLUMNS: readonly TableColumn<Row>[] = [
  { id: 'name', header: 'Name', cell: (row) => row.name, total: 'Total' },
  {
    id: 'qty',
    header: 'Quantity',
    kind: 'number',
    size: 'qty',
    cell: (row) => row.qty,
    total: '267',
  },
  { id: 'tick', header: 'Tick', kind: 'centre', cell: (row) => row.tick },
  { id: 'open', header: 'Open', kind: 'action', cell: () => 'open' },
];

describe.each(LANGUAGES)('the shared table (%s)', (language) => {
  function draw() {
    renderIn(
      language,
      <Table
        label={message(language, 'report-grid-label')}
        columns={COLUMNS}
        rows={ROWS}
        rowKey={(row) => row.id}
      />,
    );
    const table = document.querySelector('table');
    if (table === null) throw new Error('No table drawn');
    return table;
  }

  it('names the kind of every column on its heading, its cells and its total', () => {
    const table = draw();
    const kinds = ['text', 'num', 'centre', 'centre'];
    const heads = [...table.querySelectorAll('thead th')];
    expect(heads.map((cell) => cell.getAttribute('data-align'))).toEqual(kinds);
    for (const row of [...table.querySelectorAll('tbody tr'), table.querySelector('tfoot tr')]) {
      expect(
        [...(row?.querySelectorAll('td') ?? [])].map((c) => c.getAttribute('data-align')),
      ).toEqual(kinds);
    }
  });

  it('centres a number column, heading and total with it, written left to right (R1)', () => {
    const table = draw();
    const column = [
      table.querySelector('thead th:nth-child(2)'),
      ...table.querySelectorAll('tbody td:nth-child(2)'),
      table.querySelector('tfoot td:nth-child(2)'),
    ];
    for (const cell of column) {
      expect(cell).toHaveAttribute('dir', 'ltr');
      expect(cell).toHaveClass('text-center');
      expect(cell).not.toHaveClass('text-end');
    }
  });

  it('starts a text column at the start edge with its heading (R2)', () => {
    const table = draw();
    for (const cell of [
      table.querySelector('thead th:nth-child(1)'),
      ...table.querySelectorAll('tbody td:nth-child(1)'),
      table.querySelector('tfoot td:nth-child(1)'),
    ]) {
      expect(cell).toHaveClass('text-start');
      expect(cell).not.toHaveAttribute('dir');
    }
  });

  it('centres a small column and a column of buttons (R3)', () => {
    const table = draw();
    for (const position of [3, 4]) {
      for (const cell of [
        table.querySelector(`thead th:nth-child(${String(position)})`),
        ...table.querySelectorAll(`tbody td:nth-child(${String(position)})`),
      ]) {
        expect(cell).toHaveClass('text-center');
      }
    }
  });

  it('keeps every cell in the middle of its row, with the same padding (R7)', () => {
    const table = draw();
    for (const cell of table.querySelectorAll('th, td')) {
      expect(cell).toHaveClass('align-middle');
      expect(cell).toHaveClass('px-3');
      expect(cell).toHaveClass('py-2');
    }
    expect(table.querySelector('tbody tr')).not.toHaveClass('align-top');
  });

  it('gives a sized column the width token of its kind, at its heading, cells and total (R5)', () => {
    const table = draw();
    for (const cell of [
      table.querySelector('thead th:nth-child(2)'),
      ...table.querySelectorAll('tbody td:nth-child(2)'),
      table.querySelector('tfoot td:nth-child(2)'),
    ]) {
      expect(cell).toHaveClass('w-col-qty');
    }
    // A column with no size takes the room that is left.
    expect(table.querySelector('thead th:nth-child(1)')?.className).not.toMatch(/(^|\s)w-col-/);
  });

  it('keeps room for a name in the first column of words, and in no other column (R5)', () => {
    const table = draw();
    for (const cell of [
      table.querySelector('thead th:nth-child(1)'),
      ...table.querySelectorAll('tbody td:nth-child(1)'),
    ]) {
      expect(cell).toHaveClass('min-w-col-name');
    }
    for (const position of [2, 3, 4]) {
      expect(
        table.querySelector(`thead th:nth-child(${String(position)})`),
        String(position),
      ).not.toHaveClass('min-w-col-name');
    }
  });

  it('reads the heading of a button column to a screen reader and draws nothing of it', () => {
    const table = draw();
    const head = table.querySelector('thead th:nth-child(4)');
    expect(head?.querySelector('.sr-only')?.textContent).toBe('Open');
  });
});
