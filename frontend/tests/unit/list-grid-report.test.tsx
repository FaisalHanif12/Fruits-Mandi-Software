// The additions of the list grid for the report tables (plan 20): a figure with a short word in a
// slot of fixed width after it (the side of a balance), a long text that wraps, a row that heads a
// group, and a totals row that carries the word of its balance. The rest of the grid is unchanged
// (backup list, registers and the other lists have their own tests).

import { waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';
import { useLanguageStore } from '../../src/shared/i18n';
import { renderIn, visible } from '../helpers/render';

interface Line {
  id: string;
  text: string;
  amount: string;
  side: string;
  head: boolean;
}

const COLUMNS: readonly ListColumn<Line>[] = [
  {
    field: 'text',
    type: 'text',
    headerId: 'report-col-detail',
    value: (row) => row.text,
    wrap: true,
    grow: true,
  },
  {
    field: 'balance',
    type: 'balance',
    headerId: 'report-col-balance',
    value: (row) => row.amount,
    unit: (row) => row.side,
  },
];

const ROWS: Line[] = [
  { id: 'a', text: 'Heads', amount: '', side: '', head: true },
  { id: 'b', text: 'Cash', amount: '58,484', side: 'Debit', head: false },
  { id: 'c', text: 'Payables', amount: '440', side: 'Credit', head: false },
];

function draw(language: Language) {
  useLanguageStore.setState({ language });
  return renderIn(
    language,
    <ListGrid<Line>
      columns={COLUMNS}
      rows={ROWS}
      getRowId={(row) => row.id}
      labelId="report-grid-label"
      onOpen={() => undefined}
      heightMode="auto"
      isGroup={(row) => row.head}
      totals={() => ({
        labelField: 'text',
        values: { text: 'Total', balance: '58,044', 'balance:unit': 'Debit' },
      })}
    />,
  );
}

const cell = (rowId: string, colId: string) =>
  document.querySelector(`.ag-row[row-id="${rowId}"] [col-id="${colId}"]`);

describe.each(LANGUAGES)('the list grid for reports (%s)', (language: Language) => {
  it('puts the figure and its word in two parts of the cell, the word in its own slot', async () => {
    draw(language);
    await waitFor(() => {
      expect(cell('b', 'balance')).not.toBeNull();
    });
    const slot = cell('b', 'balance')?.querySelector('.cell-unit');
    expect(visible(slot?.textContent)).toBe('Debit');
    expect(visible(cell('b', 'balance')?.textContent)).toBe('58,484Debit');
    // The cell is a number cell: left to right, on the physical right edge.
    expect(cell('b', 'balance')?.classList.contains('cell-num')).toBe(true);
  });

  it('a row with no word has an empty slot, so the figures of the column keep one edge', async () => {
    draw(language);
    await waitFor(() => {
      expect(cell('a', 'balance')).not.toBeNull();
    });
    expect(cell('a', 'balance')?.querySelector('.cell-unit')).not.toBeNull();
    expect(visible(cell('a', 'balance')?.querySelector('.cell-unit')?.textContent)).toBe('');
  });

  it('marks the row that heads a group, and not the totals row', async () => {
    draw(language);
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="a"]')).not.toBeNull();
    });
    expect(document.querySelector('.ag-row[row-id="a"]')?.classList.contains('row-group')).toBe(
      true,
    );
    expect(document.querySelector('.ag-row[row-id="b"]')?.classList.contains('row-group')).toBe(
      false,
    );
    expect(document.querySelector('.ag-row-pinned')?.classList.contains('row-group')).toBe(false);
  });

  it('the totals row carries its balance and the word of the balance', async () => {
    draw(language);
    await waitFor(() => {
      expect(document.querySelector('.ag-row-pinned')).not.toBeNull();
    });
    const pinned = document.querySelector('.ag-row-pinned');
    expect(visible(pinned?.querySelector('[col-id="text"]')?.textContent)).toBe('Total');
    expect(visible(pinned?.querySelector('[col-id="balance"]')?.textContent)).toBe('58,044Debit');
  });

  it('a long text wraps and its row grows with it', async () => {
    draw(language);
    await waitFor(() => {
      expect(cell('b', 'text')).not.toBeNull();
    });
    expect(cell('b', 'text')?.classList.contains('cell-wrap')).toBe(true);
  });
});
