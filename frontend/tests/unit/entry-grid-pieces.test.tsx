// The parts the بکری screen added to the shared entry grid (plan 12): a choice cell, a grey rate
// hint, amber warnings, the screen's hold on Enter and ↑, and a focus ring that goes when focus
// leaves the grid. Urdu and English.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  EntryGrid,
  createEntryStore,
  type CellRef,
  type EntryColumn,
  type GridInterceptors,
} from '../../src/shared/grid';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import { message, renderIn, visible } from '../helpers/render';

const COLUMNS: readonly EntryColumn[] = [
  { field: 'qty', type: 'qty', headerId: 'glossary-nag' },
  { field: 'rate', type: 'rate', headerId: 'glossary-rate-farq', hintFromCalc: true },
  {
    field: 'pay',
    type: 'choice',
    headerId: 'glossary-udhari',
    enterStop: false,
    choices: [
      { value: 'credit', labelId: 'glossary-udhari' },
      { value: 'cash', labelId: 'glossary-naqdi' },
    ],
  },
  { field: 'total', type: 'calc', headerId: 'glossary-meezan' },
];

interface RigOptions {
  rows?: Record<string, string>[];
  calc?: (rowIds: readonly string[]) => Record<string, Record<string, string>>;
  warnings?: (rowIds: readonly string[]) => Record<string, Record<string, string>>;
  errors?: (rowIds: readonly string[]) => Record<string, Record<string, string>>;
  interceptors?: GridInterceptors;
}

async function renderRig(language: Language, options: RigOptions = {}) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const store = createEntryStore(options.rows ?? [{}, {}]);
  const rowIds = store.getState().rows.map((row) => row.rowId);
  function Harness() {
    useEscapeDispatcher();
    return (
      <>
        <EntryGrid
          columns={COLUMNS}
          store={store}
          labelId="glossary-bikri"
          heightMode="auto"
          {...(options.calc === undefined ? {} : { calc: options.calc(rowIds) })}
          {...(options.warnings === undefined ? {} : { cellWarnings: options.warnings(rowIds) })}
          {...(options.errors === undefined ? {} : { cellErrors: options.errors(rowIds) })}
          {...(options.interceptors === undefined ? {} : { interceptors: options.interceptors })}
        />
        <input data-testid="outside" />
      </>
    );
  }
  renderIn(language, <Harness />);
  const user = userEvent.setup();
  const cell = (rowIndex: number, field: string): HTMLElement => {
    const found = document.querySelector<HTMLElement>(
      `.ag-row[row-index="${String(rowIndex)}"] .ag-cell[col-id="${field}"]`,
    );
    if (found === null) throw new Error(`no cell ${String(rowIndex)}:${field}`);
    return found;
  };
  await waitFor(() => {
    cell(0, 'qty');
  });
  return { store, user, cell, rowIds };
}

describe.each(LANGUAGES)('a choice cell (%s)', (language) => {
  it('shows the word of its value; a click moves to the next word, Space moves on again, round and round', async () => {
    const { store, user, cell } = await renderRig(language, {
      rows: [{ pay: 'credit' }],
    });
    expect(visible(cell(0, 'pay').textContent).trim()).toBe(message(language, 'glossary-udhari'));
    await user.click(cell(0, 'pay'));
    await waitFor(() => {
      expect(visible(cell(0, 'pay').textContent).trim()).toBe(message(language, 'glossary-naqdi'));
    });
    expect(store.getState().rows[0]?.typed['pay']).toBe('cash');
    await user.keyboard(' ');
    await waitFor(() => {
      expect(store.getState().rows[0]?.typed['pay']).toBe('credit');
    });
  });

  it('with the keys only: arrow onto the cell, then Space changes it', async () => {
    const { store, user, cell } = await renderRig(language, { rows: [{ pay: 'credit' }] });
    await user.click(cell(0, 'rate'));
    // The cell after the rate is on the left in Urdu and on the right in English.
    await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
    expect(document.activeElement?.closest('.ag-cell')?.getAttribute('col-id')).toBe('pay');
    await user.keyboard(' ');
    await waitFor(() => {
      expect(store.getState().rows[0]?.typed['pay']).toBe('cash');
    });
  });

  it('is not an Enter stop: Enter on the quantity and the rate skips it', async () => {
    const { user, cell } = await renderRig(language, { rows: [{ pay: 'credit' }, {}] });
    await user.click(cell(0, 'rate'));
    await user.keyboard('{Enter}');
    const active = document.activeElement?.closest<HTMLElement>('.ag-cell');
    expect(active?.getAttribute('col-id')).toBe('qty');
    expect(active?.closest('.ag-row')?.getAttribute('row-index')).toBe('1');
  });
});

describe.each(LANGUAGES)('the grey hint, warnings and errors of a cell (%s)', (language) => {
  it('an empty rate shows the hint as grey text, and a typed rate replaces it', async () => {
    const { cell } = await renderRig(language, {
      rows: [{}, { rate: '580' }],
      calc: (ids) => ({
        [ids[0] ?? '']: { rate: '600' },
        [ids[1] ?? '']: { rate: '600' },
      }),
    });
    expect(cell(0, 'rate')).toHaveClass('cell-hint');
    expect(visible(cell(0, 'rate').textContent).trim()).toBe('600');
    expect(cell(1, 'rate')).not.toHaveClass('cell-hint');
    expect(visible(cell(1, 'rate').textContent).trim()).toBe('580');
  });

  it('a warning gives the amber look, and an error on the same cell wins', async () => {
    const { cell } = await renderRig(language, {
      warnings: (ids) => ({
        [ids[0] ?? '']: { qty: 'warn words' },
        [ids[1] ?? '']: { qty: 'warn words' },
      }),
      errors: (ids) => ({ [ids[1] ?? '']: { qty: 'error words' } }),
    });
    expect(cell(0, 'qty')).toHaveClass('cell-warn');
    expect(cell(0, 'qty')).not.toHaveClass('cell-error');
    expect(cell(1, 'qty')).toHaveClass('cell-error');
    expect(cell(1, 'qty')).not.toHaveClass('cell-warn');
  });
});

describe.each(LANGUAGES)('the screen’s hold on Enter and ↑ (%s)', (language) => {
  it('Enter on a cell goes to the screen first; when the screen takes it the grid does nothing', async () => {
    const taken: CellRef[] = [];
    const { user, cell, store } = await renderRig(language, {
      rows: [{}],
      interceptors: {
        onEnter: (at) => {
          taken.push(at);
          return at.field === 'rate';
        },
      },
    });
    await user.click(cell(0, 'rate'));
    await user.keyboard('{Enter}');
    expect(taken.map((at) => at.field)).toEqual(['rate']);
    // The screen took it: no new row, focus stays for the screen to move.
    expect(store.getState().rows).toHaveLength(1);
  });

  it('when the screen does not take Enter the grid goes on as usual', async () => {
    const onEnter = vi.fn(() => false);
    const { user, cell, store } = await renderRig(language, {
      rows: [{}],
      interceptors: { onEnter },
    });
    await user.click(cell(0, 'rate'));
    await user.keyboard('{Enter}');
    expect(onEnter).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(store.getState().rows).toHaveLength(2);
    });
  });

  it('↑ on the first row goes to the screen, and only there', async () => {
    const onAboveFirst = vi.fn(() => true);
    const { user, cell } = await renderRig(language, {
      rows: [{}, {}],
      interceptors: { onAboveFirst },
    });
    await user.click(cell(1, 'qty'));
    await user.keyboard('{ArrowUp}');
    expect(onAboveFirst).not.toHaveBeenCalled();
    await user.keyboard('{ArrowUp}');
    expect(onAboveFirst).toHaveBeenCalledTimes(1);
  });
});

describe.each(LANGUAGES)('the focus ring (%s)', (language) => {
  it('is gone when focus leaves the grid, and comes back when a cell is chosen', async () => {
    const { user, cell } = await renderRig(language);
    await user.click(cell(0, 'qty'));
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
    await user.click(screen.getByTestId('outside'));
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).toBeNull();
    });
    await user.click(cell(1, 'qty'));
    await waitFor(() => {
      expect(document.querySelector('.ag-cell-focus')).not.toBeNull();
    });
  });
});
