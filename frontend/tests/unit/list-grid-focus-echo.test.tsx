// The list grid and the focus it hands back to the screen. Every paged list keeps the row the grid reports and
// gives it back as `focusRowId`, so a restore works after a redraw. When the rows have not changed, that value is
// only the grid's own report: a restore of it a frame later would pull the focus back to a row the user has
// already left with ↓ (found on the party day sheet R35, plan 99). A new set of rows still restores it.

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { describe, expect, it } from 'vitest';
import { bundles } from '../../src/i18n';
import { ListGrid, type ListColumn } from '../../src/shared/grid';

interface Row {
  id: string;
  name: string;
}

const ROWS: Row[] = [
  { id: '1', name: 'Banana' },
  { id: '2', name: 'Apple' },
  { id: '3', name: 'Mango' },
];

const COLUMNS: ListColumn<Row>[] = [
  { field: 'name', type: 'text', headerId: 'common-back', value: (row) => row.name },
];

const frames = () =>
  act(async () => {
    for (let index = 0; index < 3; index += 1) {
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
  });

function grid(focusRowId: string | null, rows: Row[] = ROWS) {
  return (
    <LocalizationProvider l10n={new ReactLocalization([bundles.en])}>
      <ListGrid<Row>
        columns={COLUMNS}
        rows={rows}
        getRowId={(row) => row.id}
        labelId="common-back"
        heightMode="auto"
        onOpen={() => undefined}
        onFocusRow={() => undefined}
        focusRowId={focusRowId}
      />
    </LocalizationProvider>
  );
}

const cell = (rowId: string) => {
  const found = document.querySelector<HTMLElement>(`.ag-row[row-id="${rowId}"] [col-id="name"]`);
  if (found === null) throw new Error(`no cell of row ${rowId}`);
  return found;
};

const focusedRow = () => document.activeElement?.closest('.ag-row')?.getAttribute('row-id');

/** Puts the keys on a row, the way a user does: a click, then the arrow. */
async function down(from: string, to: string) {
  const start = cell(from);
  fireEvent.keyDown(start, { key: 'ArrowDown', code: 'ArrowDown' });
  await waitFor(() => {
    expect(focusedRow()).toBe(to);
  });
}

describe('the focus a screen hands back to the list grid', () => {
  it("does not pull the focus back to a row the user has left, when it is only the grid's own report", async () => {
    const view = render(grid(null));
    await screen.findByText('Banana');
    fireEvent.mouseDown(cell('1'));
    fireEvent.click(cell('1'));
    cell('1').focus();
    await down('1', '2');
    await down('2', '3');
    // The screen's state is a step behind: it hands back row 2 while the user is on row 3.
    view.rerender(grid('2'));
    await frames();
    expect(focusedRow()).toBe('3');
  });

  it('still restores the row after the list is drawn again with new rows', async () => {
    const view = render(grid(null));
    await screen.findByText('Banana');
    fireEvent.mouseDown(cell('1'));
    fireEvent.click(cell('1'));
    cell('1').focus();
    await down('1', '2');
    // A new answer of the same list: the row the screen asks for gets the focus again.
    view.rerender(grid('1', [...ROWS]));
    await waitFor(() => {
      expect(focusedRow()).toBe('1');
    });
  });
});
