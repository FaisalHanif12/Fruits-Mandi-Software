// A zero amount in a money column of a read-only list is read in the quiet ink (plan 75, item 8): the
// grid gives the cell the class `cell-zero`. Only a money column, only a body cell whose text is "0", never
// the total row; a column of quantities or words is not touched. It is colour only: the 0 stays in its
// cell, so nothing moves and nothing is hidden (NFR-USE-12, NFR-USE-19).

import { render, screen } from '@testing-library/react';
import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { describe, expect, it } from 'vitest';
import { bundles } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';

interface Row {
  id: string;
  name: string;
  amount: string;
  qty: string;
}

const ROWS: Row[] = [
  { id: '1', name: 'Banana', amount: '0', qty: '0' },
  { id: '2', name: 'Apple', amount: '1,520', qty: '10' },
];

const COLUMNS: ListColumn<Row>[] = [
  { field: 'name', type: 'text', headerId: 'common-back', value: (row) => row.name },
  { field: 'amount', type: 'money', headerId: 'common-retry', value: (row) => row.amount },
  { field: 'qty', type: 'count', headerId: 'common-back', value: (row) => row.qty },
];

function cell(rowSelector: string, colId: string): Element {
  const found = document.querySelector(`${rowSelector} [col-id="${colId}"]`);
  if (found === null) throw new Error(`no cell ${rowSelector} ${colId}`);
  return found;
}

describe('a zero amount in a read-only list', () => {
  it('is quiet in a money column, and only there, and never in the total row', async () => {
    render(
      <LocalizationProvider l10n={new ReactLocalization([bundles.en])}>
        <ListGrid<Row>
          columns={COLUMNS}
          rows={ROWS}
          getRowId={(row) => row.id}
          labelId="common-back"
          heightMode="auto"
          onOpen={() => undefined}
          totals={() => ({
            labelField: 'name',
            values: { name: 'Total', amount: '0', qty: '0' },
          })}
        />
      </LocalizationProvider>,
    );
    await screen.findByText('Banana');
    // The 0 of the money column of the first row is quiet, the figure under it is not.
    expect(cell('.ag-row[row-id="1"]', 'amount')).toHaveClass('cell-zero');
    expect(cell('.ag-row[row-id="2"]', 'amount')).not.toHaveClass('cell-zero');
    // A column of quantities is not a column of amounts: its 0 keeps the ink of the row.
    expect(cell('.ag-row[row-id="1"]', 'qty')).not.toHaveClass('cell-zero');
    expect(cell('.ag-row[row-id="1"]', 'name')).not.toHaveClass('cell-zero');
    // The total row says its 0 as strongly as any total.
    const pinned = document.querySelector('.ag-row-pinned [col-id="amount"]');
    expect(pinned).not.toBeNull();
    expect(pinned).not.toHaveClass('cell-zero');
    // The 0 is still there, in the same cell.
    expect(cell('.ag-row[row-id="1"]', 'amount')).toHaveTextContent('0');
  });
});
