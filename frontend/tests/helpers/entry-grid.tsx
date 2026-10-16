// Test helper: the shared entry grid with sale-line style columns, rendered the way the app does
// (Fluent, direction, the Esc dispatcher the shell mounts), and small finders for its cells.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useMemo } from 'react';
import {
  EntryGrid,
  createEntryStore,
  type EntryColumn,
  type EntryStore,
} from '../../src/shared/grid';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import type { Language } from '../../src/i18n';
import { renderIn } from './render';

export const DIFFERENT = 'y';

/**
 * Buyer, qty and rate are the Enter stops of a sale line (PRD FR-7.14). "Different rate" and the
 * note are editable but not stops. Buyer rate is a stop, open only when different is y. The date
 * is editable, not a stop. The last column is calculated.
 */
export const COLUMNS: readonly EntryColumn[] = [
  { field: 'buyer', type: 'account', headerId: 'glossary-gahak' },
  { field: 'qty', type: 'qty', headerId: 'glossary-nag' },
  { field: 'rate', type: 'rate', headerId: 'glossary-rate-farq' },
  { field: 'different', type: 'text', headerId: 'glossary-udhari', enterStop: false },
  { field: 'date', type: 'date', headerId: 'glossary-adaigi', enterStop: false },
  {
    field: 'buyerRate',
    type: 'money',
    headerId: 'glossary-naqdi',
    editable: (row) => row.typed['different'] === DIFFERENT,
  },
  { field: 'note', type: 'text', headerId: 'glossary-kharcha', enterStop: false },
  { field: 'line', type: 'calc', headerId: 'glossary-meezan' },
];

/** A read-only words column, added to the grid when a test asks for it. */
const LABEL_COLUMN: EntryColumn = { field: 'name', type: 'label', headerId: 'glossary-bikri' };

function Harness({
  store,
  copyAbove,
  onExplain,
  fixedRows,
  cellErrors,
  withLabel,
}: {
  withLabel?: boolean;
  store: EntryStore;
  copyAbove?: boolean;
  onExplain?: (cell: { rowId: string; field: string }) => void;
  fixedRows?: boolean;
  cellErrors?: Record<string, Record<string, string>>;
}) {
  useEscapeDispatcher();
  const rows = store((state) => state.rows);
  const calc = useMemo(
    () =>
      Object.fromEntries(
        rows.map((row) => [
          row.rowId,
          { line: [row.typed['qty'], row.typed['rate']].join('|'), name: row.typed['buyer'] ?? '' },
        ]),
      ),
    [rows],
  );
  return (
    <EntryGrid
      columns={withLabel === true ? [...COLUMNS, LABEL_COLUMN] : COLUMNS}
      store={store}
      labelId="glossary-bikri"
      calc={calc}
      copyAbove={copyAbove ?? false}
      {...(onExplain === undefined ? {} : { onExplain })}
      {...(fixedRows === undefined ? {} : { fixedRows })}
      {...(cellErrors === undefined ? {} : { cellErrors })}
      heightMode="auto"
    />
  );
}

export interface GridRig {
  store: EntryStore;
  user: ReturnType<typeof userEvent.setup>;
  cell: (rowIndex: number, field: string) => HTMLElement;
  /** Clicks a cell so it has focus (a single click never starts typing). */
  focusCell: (rowIndex: number, field: string) => Promise<void>;
  /** The cell that has focus now, as "row:field". */
  focused: () => string | null;
  typed: (rowIndex: number, field: string) => string | undefined;
  isEditing: () => boolean;
}

export async function renderGrid(
  language: Language,
  options: {
    rows?: Record<string, string>[];
    copyAbove?: boolean;
    onExplain?: (cell: { rowId: string; field: string }) => void;
    fixedRows?: boolean;
    /** Adds a read-only words column called `name`, which shows the typed `buyer`. */
    withLabel?: boolean;
    /** Built from the rows that exist: row index, then field, then the words. */
    cellErrors?: (rowIds: readonly string[]) => Record<string, Record<string, string>>;
  } = {},
): Promise<GridRig> {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const store = createEntryStore(options.rows ?? []);
  const cellErrors = options.cellErrors?.(store.getState().rows.map((row) => row.rowId));
  renderIn(
    language,
    <Harness
      store={store}
      {...(options.fixedRows === undefined ? {} : { fixedRows: options.fixedRows })}
      {...(options.withLabel === undefined ? {} : { withLabel: options.withLabel })}
      {...(cellErrors === undefined ? {} : { cellErrors })}
      {...(options.copyAbove === undefined ? {} : { copyAbove: options.copyAbove })}
      {...(options.onExplain === undefined ? {} : { onExplain: options.onExplain })}
    />,
  );
  const user = userEvent.setup();
  const cell = (rowIndex: number, field: string): HTMLElement => {
    const found = document.querySelector<HTMLElement>(
      `.ag-row[row-index="${String(rowIndex)}"] .ag-cell[col-id="${field}"]`,
    );
    if (found === null) throw new Error(`no cell ${String(rowIndex)}:${field}`);
    return found;
  };
  await waitFor(() => {
    cell(0, 'buyer');
  });
  return {
    store,
    user,
    cell,
    focusCell: async (rowIndex, field) => {
      await user.click(cell(rowIndex, field));
    },
    focused: () => {
      const active = document.activeElement;
      const cellElement = active?.closest<HTMLElement>('.ag-cell');
      const row = cellElement?.closest<HTMLElement>('.ag-row');
      if (cellElement == null || row == null) return null;
      return `${row.getAttribute('row-index') ?? '?'}:${cellElement.getAttribute('col-id') ?? '?'}`;
    },
    typed: (rowIndex, field) => store.getState().rows[rowIndex]?.typed[field],
    isEditing: () => document.querySelector('.ag-cell-inline-editing') !== null,
  };
}

export { screen };
