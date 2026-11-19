// The two additions to the shared list grid for the analytics screens (plan 21): `onOpen` tells which
// column the Enter or the double click was on, and the pinned totals row opens only what the screen
// offers (`onOpenTotals`); without it the totals row opens nothing and never hands the screen a row
// that is not a record.

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { describe, expect, it, vi } from 'vitest';
import { bundles } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';

interface Row {
  id: string;
  name: string;
  amount: string;
}

const ROWS: Row[] = [
  { id: '1', name: 'Banana', amount: '3,928' },
  { id: '2', name: 'Apple', amount: '1,520' },
];

const COLUMNS: ListColumn<Row>[] = [
  { field: 'name', type: 'text', headerId: 'common-back', value: (row) => row.name },
  { field: 'amount', type: 'money', headerId: 'common-retry', value: (row) => row.amount },
];

function show(props: {
  onOpen: (row: Row, column?: string) => void;
  onOpenTotals?: (c: string) => void;
}) {
  return render(
    <LocalizationProvider l10n={new ReactLocalization([bundles.en])}>
      <ListGrid<Row>
        columns={COLUMNS}
        rows={ROWS}
        getRowId={(row) => row.id}
        labelId="common-back"
        heightMode="auto"
        totals={() => ({ labelField: 'name', values: { name: 'Total', amount: '5,448' } })}
        {...props}
      />
    </LocalizationProvider>,
  );
}

function cellOf(rowSelector: string, colId: string): Element {
  const found = document.querySelector(`${rowSelector} [col-id="${colId}"]`);
  if (found === null) throw new Error(`no cell ${rowSelector} ${colId}`);
  return found;
}

describe('the list grid hands the screen the column of the cell', () => {
  it('Enter on a cell gives the row and the column that has focus', async () => {
    const onOpen = vi.fn();
    show({ onOpen });
    await screen.findByText('Banana');
    const cell = cellOf('.ag-row[row-id="1"]', 'amount');
    fireEvent.mouseDown(cell);
    fireEvent.click(cell);
    fireEvent.keyDown(cell, { key: 'Enter', code: 'Enter' });
    await waitFor(() => {
      expect(onOpen).toHaveBeenCalledWith(ROWS[0], 'amount');
    });
  });

  it('a double click gives the row and the column of the cell clicked', async () => {
    const onOpen = vi.fn();
    show({ onOpen });
    await screen.findByText('Apple');
    fireEvent.doubleClick(cellOf('.ag-row[row-id="2"]', 'name'));
    await waitFor(() => {
      expect(onOpen).toHaveBeenCalledWith(ROWS[1], 'name');
    });
  });
});

describe('the totals row', () => {
  it('opens only what the screen offers, with the column of the cell', async () => {
    const onOpen = vi.fn();
    const onOpenTotals = vi.fn();
    show({ onOpen, onOpenTotals });
    await screen.findByText('Total');
    fireEvent.doubleClick(cellOf('.ag-row-pinned', 'amount'));
    await waitFor(() => {
      expect(onOpenTotals).toHaveBeenCalledWith('amount');
    });
    expect(onOpen).not.toHaveBeenCalled();
  });

  it('never hands the screen a row when it offers nothing for the totals', async () => {
    const onOpen = vi.fn();
    show({ onOpen });
    await screen.findByText('Total');
    fireEvent.doubleClick(cellOf('.ag-row-pinned', 'amount'));
    // Give the grid its turn; nothing must have been opened.
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(onOpen).not.toHaveBeenCalled();
  });
});
