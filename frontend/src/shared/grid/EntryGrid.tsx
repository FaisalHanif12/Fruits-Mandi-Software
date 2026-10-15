// The one grid for typing entries (بکری lots and sale lines, vouchers, جنرل lines, opening
// balances): AG Grid, set up once here (entry-grid skill, Code Standards F27, F39). A screen
// gives it columns and a row store. This file is the only one that imports AgGridReact for entry
// screens.
//
// - What the user types lives in the store, never only in the grid. A language switch makes a
//   new grid (`enableRtl` is fixed when a grid is created, CLAUDE.md Gotchas) from the store,
//   and the focused cell is focused again.
// - Calculated cells show text the screen got from the backend (`calc`). The grid never works
//   out money (CLAUDE.md rule 1).
// - Keys: gridKeys.ts. Esc: while a cell is being typed in, the grid answers the `edit` level of
//   the Esc order (escape.ts).

import { useLocalization } from '@fluent/react';
import type {
  CellFocusedEvent,
  ColDef,
  GridApi,
  GridReadyEvent,
  ValueFormatterParams,
  ValueGetterParams,
  ValueSetterParams,
} from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
import { useEffect, useLayoutEffect, useMemo, useRef } from 'react';
import { useGrouping, type Grouping } from '../format';
import { directionOf, useLanguage } from '../i18n';
import { useEscape } from '../keyboard';
import { displayText } from '../ui/NumberTextInput';
import './agGridSetup';
import { COLUMN_WIDTHS, isNumberLike } from './columnTypes';
import { isEditable } from './enterOrder';
import type { EntryStore } from './entryStore';
import { createGridKeyHandler } from './gridKeys';
import { isReadOnlyType, type EntryColumn, type EntryRow } from './types';

interface EntryGridProps {
  columns: readonly EntryColumn[];
  store: EntryStore;
  /** The Fluent message ID of the grid's name, read by screen readers. */
  labelId: string;
  /** Calculated text per row and field, from the backend preview. Shown in `calc` cells. */
  calc?: Readonly<Record<string, Readonly<Record<string, string>>>>;
  /** Ctrl+D copies the cell from the line above (sale-line grids only). */
  copyAbove?: boolean;
  /** Ctrl+I on a calculated cell: show how it was worked out. */
  onExplain?: (cell: { rowId: string; field: string }) => void;
  /** No key adds or removes a row: one row per account in the opening balances grid. */
  fixedRows?: boolean;
  /**
   * What the backend refused, per row and field, as words in the app language. The cell gets the
   * error look and the words as its tooltip; the screen lists them too, so colour is never the
   * only signal (NFR-USE-12).
   */
  cellErrors?: Readonly<Record<string, Readonly<Record<string, string>>>>;
  /**
   * 'fill': the parent gives the grid its height and only visible rows are drawn. 'auto': the
   * grid is as tall as its rows and draws them all (short grids, the development page, tests).
   */
  heightMode?: 'fill' | 'auto';
}

function toColDef(
  column: EntryColumn,
  headerName: string,
  grouping: Grouping,
  onCommit: (rowId: string, field: string, text: string) => void,
  calc: EntryGridProps['calc'],
  cellErrors: EntryGridProps['cellErrors'],
): ColDef<EntryRow> {
  const numberLike = isNumberLike(column.type);
  const grouped = column.type === 'money' || column.type === 'qty' || column.type === 'rate';
  return {
    colId: column.field,
    headerName,
    width: COLUMN_WIDTHS[column.type],
    resizable: false,
    sortable: false,
    suppressMovable: true,
    editable: (params) => params.data !== undefined && isEditable(column, params.data),
    valueGetter: (params: ValueGetterParams<EntryRow>) => {
      if (params.data === undefined) return '';
      if (isReadOnlyType(column.type)) return calc?.[params.data.rowId]?.[column.field] ?? '';
      return params.data.typed[column.field] ?? '';
    },
    // The text goes to the store. Returning false tells the grid the value did not change in
    // its own data, so the row is redrawn from the store (the one source of truth).
    valueSetter: (params: ValueSetterParams<EntryRow>) => {
      onCommit(params.data.rowId, column.field, String(params.newValue ?? ''));
      return false;
    },
    valueFormatter: (params: ValueFormatterParams<EntryRow>) => {
      const value = typeof params.value === 'string' ? params.value : '';
      if (value === '' && params.data !== undefined) {
        return column.placeholder?.(params.data) ?? '';
      }
      return grouped ? displayText(value, grouping) : value;
    },
    cellClass: numberLike ? ['cell-num'] : [],
    // The heading of a number column sits on the same edge as its figures (NFR-USE-19).
    headerClass: numberLike ? ['header-num'] : [],
    tooltipValueGetter: (params) =>
      params.data === undefined ? undefined : cellErrors?.[params.data.rowId]?.[column.field],
    cellClassRules: {
      'cell-error': (params) =>
        params.data !== undefined && cellErrors?.[params.data.rowId]?.[column.field] !== undefined,
      'cell-calc': () => isReadOnlyType(column.type),
      'cell-hint': (params) =>
        params.data !== undefined &&
        (params.value === '' || params.value === undefined) &&
        column.placeholder?.(params.data) !== undefined,
    },
  };
}

function GridInner({
  columns,
  store,
  labelId,
  calc,
  copyAbove,
  onExplain,
  fixedRows,
  cellErrors,
  heightMode = 'fill',
}: EntryGridProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const direction = directionOf(language);
  const grouping = useGrouping();
  const rows = useRowsOf(store);
  const apiRef = useRef<GridApi<EntryRow> | null>(null);

  // Esc while typing in a cell cancels the typing and keeps the old value (PRD FR-7.18).
  useEscape(
    'edit',
    () => {
      apiRef.current?.stopEditing(true);
    },
    () => (apiRef.current?.getEditingCells().length ?? 0) > 0,
  );

  // The grid is rebuilt when the language changes. Text still being typed is saved first.
  useLayoutEffect(
    () => () => {
      apiRef.current?.stopEditing();
    },
    [],
  );

  const keyHandler = useMemo(
    () =>
      createGridKeyHandler({
        store,
        columns,
        direction,
        copyAbove: copyAbove ?? false,
        fixedRows: fixedRows ?? false,
        ...(onExplain === undefined ? {} : { onExplain }),
      }),
    [store, columns, direction, copyAbove, onExplain, fixedRows],
  );

  const columnDefs = useMemo(
    () =>
      columns.map((column) =>
        toColDef(
          column,
          l10n.getString(column.headerId),
          grouping,
          store.getState().updateTyped,
          calc,
          cellErrors,
        ),
      ),
    [columns, l10n, grouping, store, calc, cellErrors],
  );

  const defaultColDef = useMemo<ColDef<EntryRow>>(
    () => ({ suppressKeyboardEvent: keyHandler }),
    [keyHandler],
  );

  // After a row is added or removed, focus the cell the key asked for, once the rows are drawn.
  useEffect(() => {
    const pending = store.getState().pendingFocus;
    const api = apiRef.current;
    if (pending === null || api === null) return;
    store.getState().setPendingFocus(null);
    const rowIndex =
      pending.rowIndex === 'last' ? rows.length - 1 : Math.min(pending.rowIndex, rows.length - 1);
    api.setFocusedCell(rowIndex, pending.field);
    api.ensureIndexVisible(rowIndex);
  }, [rows, store]);

  const onGridReady = (event: GridReadyEvent<EntryRow>) => {
    apiRef.current = event.api;
    // A new grid (for example after a language switch) focuses the cell that had focus.
    const focus = store.getState().focus;
    const rowIndex =
      focus === null ? -1 : store.getState().rows.findIndex((row) => row.rowId === focus.rowId);
    if (focus !== null && rowIndex >= 0) event.api.setFocusedCell(rowIndex, focus.field);
  };

  const onCellFocused = (event: CellFocusedEvent<EntryRow>) => {
    if (event.rowIndex === null || event.column === null || typeof event.column === 'string')
      return;
    const row = store.getState().rows[event.rowIndex];
    if (row !== undefined)
      store.getState().setFocus({ rowId: row.rowId, field: event.column.getColId() });
  };

  return (
    <div
      className="entry-grid ag-theme-quartz"
      role="group"
      aria-label={l10n.getString(labelId)}
      dir={direction}
    >
      <AgGridReact<EntryRow>
        theme="legacy"
        enableRtl={direction === 'rtl'}
        rowData={rows}
        columnDefs={columnDefs}
        defaultColDef={defaultColDef}
        getRowId={(params) => params.data.rowId}
        domLayout={heightMode === 'auto' ? 'autoHeight' : 'normal'}
        suppressColumnVirtualisation
        animateRows={false}
        suppressMovableColumns
        tooltipShowDelay={300}
        stopEditingWhenCellsLoseFocus
        onGridReady={onGridReady}
        onCellFocused={onCellFocused}
      />
    </div>
  );
}

/** The rows of the store, kept up to date. */
function useRowsOf(store: EntryStore): EntryRow[] {
  return store((state) => state.rows);
}

/** The entry grid. It is rebuilt when the language changes, from the store. */
export function EntryGrid(props: EntryGridProps) {
  const language = useLanguage();
  return <GridInner key={language} {...props} />;
}
