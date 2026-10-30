// The one grid for typing entries (بکری lots and sale lines, vouchers, جنرل lines, opening
// balances): AG Grid, set up once here (entry-grid skill, Code Standards F27, F39). A screen
// gives it columns and a row store. This file is the only one that imports AgGridReact for entry
// screens.
//
// - What the user types lives in the store, never only in the grid. A language switch makes a
//   new grid (`enableRtl` is fixed when a grid is created, CLAUDE.md Gotchas) from the store,
//   and the focused cell is focused again.
// - Calculated cells show text the screen got from the backend (`calc`). The grid never works
//   out money (CLAUDE.md rule 1). What the backend sent (`calc`, `workings`, `cellErrors`) is read
//   through a ref and the cells are refreshed, so a new answer never rebuilds the columns or
//   interrupts a cell being typed in.
// - Keys: gridKeys.ts. Esc: while a cell is being typed in, the grid answers the `edit` level of
//   the Esc order (escape.ts).
// - Picker cells (an account, an item) open the shared picker as a popup (PickerCellEditor).
//   Choice cells (ادھاری / نقدی) change with Space or a click. A calculated cell can carry the ⓘ
//   with the backend's working (PRD FR-7.43); Ctrl+I opens the same working from the cell.

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
import { AgGridReact, type CustomCellRendererProps } from 'ag-grid-react';
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { useGrouping, type Grouping } from '../format';
import { useLatest } from '../hooks/useLatest';
import { directionOf, displayName, useLanguage } from '../i18n';
import { useEscape } from '../keyboard';
import { CalcInfo, CalcInfoAt, type Working } from '../ui/CalcInfo';
import { displayText } from '../ui/NumberTextInput';
import './agGridSetup';
import { COLUMN_WIDTHS, FIT_MIN_WIDTHS, FIT_WEIGHTS, isNumberLike } from './columnTypes';
import { isEditable } from './enterOrder';
import type { EntryStore } from './entryStore';
import { createGridKeyHandler } from './gridKeys';
import { PickerCellEditor } from './PickerCellEditor';
import {
  isReadOnlyType,
  type CellRef,
  type EntryColumn,
  type EntryRow,
  type GridInterceptors,
} from './types';

type PerRow<T> = Readonly<Record<string, Readonly<Record<string, T>>>>;
type KeyHandler = ReturnType<typeof createGridKeyHandler>;

interface EntryGridProps {
  columns: readonly EntryColumn[];
  store: EntryStore;
  /** The Fluent message ID of the grid's name, read by screen readers. */
  labelId: string;
  /** Calculated text per row and field, from the backend preview. Shown in `calc` cells. */
  calc?: PerRow<string>;
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
  cellErrors?: PerRow<string>;
  /** What only warns, per row and field: the amber look and the words as the tooltip. */
  cellWarnings?: PerRow<string>;
  /**
   * Cells the user changed in a posting open for edit (plan 16), per row and field, with the words
   * "changed, it was ..." as the tooltip. The cell gets a corner mark, so colour is never the only
   * signal; the screen lists the changes in words too (NFR-USE-12).
   */
  cellChanged?: PerRow<string>;
  /** Can Ctrl+Delete remove this row? Default: yes. A saved lot of an edit cannot be removed. */
  canRemoveRow?: (row: EntryRow) => boolean;
  /**
   * 'fill': the parent gives the grid its height and only visible rows are drawn. 'auto': the
   * grid is as tall as its rows and draws them all (short grids, the development page, tests).
   */
  heightMode?: 'fill' | 'auto';
  /**
   * An 'auto' grid keeps the room of four rows even with fewer, so a screen of lines (the journal)
   * does not look empty and does not jump as lines are added (plan 38).
   */
  minRows?: 4;
  /**
   * The columns share the free width of the grid down to a least width each, so the grid never
   * scrolls sideways at the smallest window (NFR-USE-15). Default: fixed widths per type.
   */
  fit?: boolean;
  /** The backend's working of a calculated amount, per row and field. Gives the cell its ⓘ. */
  workings?: PerRow<Working>;
  /** What the screen adds to Enter and ↑ (moving between grids and panels). */
  interceptors?: GridInterceptors;
  /** The account the screen's party card would show now, while a picker cell is open. */
  onCardAccount?: (accountId: number | null) => void;
}

/** What the backend sent, read by the cells when they draw. Kept apart from the column setup. */
interface Live {
  calc: PerRow<string> | undefined;
  workings: PerRow<Working> | undefined;
  cellErrors: PerRow<string> | undefined;
  cellWarnings: PerRow<string> | undefined;
  cellChanged: PerRow<string> | undefined;
}

/** The word of a choice column for a value, or undefined when it has no word. */
function choiceLabel(
  column: EntryColumn,
  value: string,
  word: (id: string) => string,
): string | undefined {
  const choice = column.choices?.find((candidate) => candidate.value === value);
  return choice === undefined ? undefined : word(choice.labelId);
}

interface ColumnEnv {
  grouping: Grouping;
  language: ReturnType<typeof useLanguage>;
  word: (id: string) => string;
  store: EntryStore;
  /** The newest answer of the backend, read when a cell draws. */
  live: () => Live;
  fit: boolean;
  getKeyHandler: () => KeyHandler;
  onCardAccount: ((accountId: number | null) => void) | undefined;
  cycle: (cell: CellRef) => void;
  /** The cell a mouse press has just put the focus on, read by the click that follows it. */
  mouseFocus: MouseFocus;
}

/**
 * Whether a click only selected its cell. A press on a cell that is not focused focuses it, and
 * the grid says so (`cellFocused`) before the click arrives; a click on a cell that already has
 * the focus has no such move before it.
 */
interface MouseFocus {
  /** The grid moved the focus to this cell (from another) and tells what pressed. */
  noteFocus: (cell: { rowIndex: number; colId: string }, pressedByMouse: boolean) => void;
  /** True once if the click on this cell only selected it (a press that moved the focus here). */
  onlySelected: (cell: { rowIndex: number; colId: string }) => boolean;
}

/** Keeps the cell a mouse press has just focused, for the click that follows it. */
function useMouseFocus(): MouseFocus {
  const state = useRef<{
    pressed: { rowIndex: number; colId: string } | null;
    last: { rowIndex: number; colId: string } | null;
  }>({ pressed: null, last: null });
  const noteFocus = useCallback<MouseFocus['noteFocus']>((cell, pressedByMouse) => {
    const { last } = state.current;
    const moved = !(last?.rowIndex === cell.rowIndex && last.colId === cell.colId);
    state.current.last = cell;
    state.current.pressed = pressedByMouse && moved ? cell : null;
  }, []);
  const onlySelected = useCallback<MouseFocus['onlySelected']>((cell) => {
    const { pressed } = state.current;
    state.current.pressed = null;
    return pressed?.rowIndex === cell.rowIndex && pressed.colId === cell.colId;
  }, []);
  return useMemo(() => ({ noteFocus, onlySelected }), [noteFocus, onlySelected]);
}

function toColDef(column: EntryColumn, env: ColumnEnv): ColDef<EntryRow> {
  const { grouping, language, word, store, live, fit } = env;
  const numberLike = isNumberLike(column.type);
  const grouped = column.type === 'money' || column.type === 'qty' || column.type === 'rate';
  const readOnly = isReadOnlyType(column.type);
  const calcOf = (row: EntryRow): string => live().calc?.[row.rowId]?.[column.field] ?? '';
  const errorOf = (row: EntryRow | undefined): string | undefined =>
    row === undefined ? undefined : live().cellErrors?.[row.rowId]?.[column.field];
  const warningOf = (row: EntryRow | undefined): string | undefined =>
    row === undefined ? undefined : live().cellWarnings?.[row.rowId]?.[column.field];
  const changedOf = (row: EntryRow | undefined): string | undefined =>
    row === undefined ? undefined : live().cellChanged?.[row.rowId]?.[column.field];
  // The grey hint of an empty cell: written in the column, or sent by the backend (the last rate).
  const hintOf = (row: EntryRow): string | undefined =>
    column.hintFromCalc === true ? calcOf(row) || undefined : column.placeholder?.(row);
  const sizes = fit
    ? {
        flex: FIT_WEIGHTS[column.type],
        minWidth: column.minWidth ?? FIT_MIN_WIDTHS[column.type],
        // A heading that does not fit goes on a second line, and the header grows to hold it.
        wrapHeaderText: true,
        autoHeaderHeight: true,
      }
    : column.grow === true
      ? { width: COLUMN_WIDTHS[column.type], flex: 1, minWidth: COLUMN_WIDTHS[column.type] }
      : { width: COLUMN_WIDTHS[column.type] };

  const base: ColDef<EntryRow> = {
    colId: column.field,
    headerName: word(column.headerId),
    ...sizes,
    resizable: false,
    sortable: false,
    suppressMovable: true,
    editable: (params) => params.data !== undefined && isEditable(column, params.data),
    cellClass: numberLike ? ['cell-num'] : [],
    // The heading of a number column sits on the same edge as its figures (NFR-USE-19).
    headerClass: numberLike ? ['header-num'] : [],
    tooltipValueGetter: (params) => {
      const row = params.data;
      if (row === undefined) return undefined;
      const error = errorOf(row) ?? warningOf(row) ?? changedOf(row);
      if (error !== undefined) return error;
      if ((row.typed[column.field] ?? '') === '' && column.hintTooltipField !== undefined) {
        return live().calc?.[row.rowId]?.[column.hintTooltipField];
      }
      // Words the screen shows can be longer than their cell: the tooltip holds them in full.
      if (column.type === 'label') return calcOf(row) || undefined;
      return undefined;
    },
    cellClassRules: {
      'cell-error': (params) => errorOf(params.data) !== undefined,
      'cell-warn': (params) =>
        errorOf(params.data) === undefined && warningOf(params.data) !== undefined,
      'cell-changed': (params) => changedOf(params.data) !== undefined,
      'cell-calc': () => readOnly,
      'cell-hint': (params) =>
        params.data !== undefined &&
        (params.value === '' || params.value === undefined) &&
        hintOf(params.data) !== undefined,
      'cell-hint-number': () => column.hintIsNumber === true,
    },
  };

  if (column.type === 'picker') {
    const spec = column.picker;
    return {
      ...base,
      valueGetter: (params: ValueGetterParams<EntryRow>) => {
        const record = params.data?.picked?.[column.field];
        return record === undefined ? '' : displayName(record, language);
      },
      // The editor writes the record to the store itself; the grid has nothing to set.
      valueSetter: () => false,
      cellEditor: PickerCellEditor,
      cellEditorPopup: true,
      cellEditorPopupPosition: 'under',
      // A click on a picker cell that already has the focus opens the picker with the full list,
      // like a click on the field of a form (plan 32, R7). The click that only selects the cell
      // (a press on a cell that had no focus) opens nothing, so moving about the grid with the
      // mouse stays quiet. A key never opens the list either: arriving on the cell by Enter, Tab or
      // an arrow opens nothing, and Alt+↓ is the key for the list (PRD FR-7.16, gridKeys.ts).
      onCellClicked: (event) => {
        if (event.data === undefined || !isEditable(column, event.data)) return;
        const { rowIndex } = event;
        if (rowIndex === null) return;
        if (env.mouseFocus.onlySelected({ rowIndex, colId: column.field })) return;
        const editing = event.api
          .getEditingCells()
          .some((cell) => cell.rowIndex === rowIndex && cell.colId === column.field);
        if (editing) return;
        store.getState().setEditHint('open-list');
        event.api.startEditingCell({ rowIndex, colKey: column.field });
      },
      cellEditorParams:
        spec === undefined
          ? undefined
          : {
              spec,
              store,
              field: column.field,
              getKeyHandler: env.getKeyHandler,
              onCardAccount: env.onCardAccount,
            },
      tooltipValueGetter: (params) => {
        const error = errorOf(params.data) ?? warningOf(params.data) ?? changedOf(params.data);
        if (error !== undefined) return error;
        const record = params.data?.picked?.[column.field];
        return record === undefined ? undefined : `${record.code} ${displayName(record, language)}`;
      },
    };
  }

  if (column.type === 'choice') {
    const shown = (row: EntryRow): string => {
      const typed = row.typed[column.field] ?? '';
      return typed === '' ? calcOf(row) : typed;
    };
    return {
      ...base,
      editable: false,
      valueGetter: (params: ValueGetterParams<EntryRow>) =>
        params.data === undefined ? '' : shown(params.data),
      valueFormatter: (params: ValueFormatterParams<EntryRow>) =>
        choiceLabel(column, typeof params.value === 'string' ? params.value : '', word) ?? '',
      cellClass: ['cell-choice'],
      onCellClicked: (event) => {
        if (event.data === undefined || !isEditable(column, event.data)) return;
        env.cycle({ rowId: event.data.rowId, field: column.field });
      },
    };
  }

  // A calculated cell, or an editable cell that shows a calculated value while it is empty.
  const explainable = column.explain === true && (readOnly || column.showsCalc === true);
  return {
    ...base,
    valueGetter: (params: ValueGetterParams<EntryRow>) => {
      if (params.data === undefined) return '';
      if (readOnly) return calcOf(params.data);
      return params.data.typed[column.field] ?? '';
    },
    // The text goes to the store. Returning false tells the grid the value did not change in
    // its own data, so the row is redrawn from the store (the one source of truth).
    valueSetter: (params: ValueSetterParams<EntryRow>) => {
      store.getState().updateTyped(params.data.rowId, column.field, String(params.newValue ?? ''));
      return false;
    },
    valueFormatter: (params: ValueFormatterParams<EntryRow>) => {
      const value = typeof params.value === 'string' ? params.value : '';
      if (value === '' && params.data !== undefined) {
        if (column.showsCalc === true) return calcOf(params.data);
        return hintOf(params.data) ?? '';
      }
      return grouped ? displayText(value, grouping) : value;
    },
    ...(explainable
      ? {
          cellRenderer: (params: CustomCellRendererProps<EntryRow>) => {
            const row = params.data;
            const working =
              row === undefined ? undefined : live().workings?.[row.rowId]?.[column.field];
            const text = params.valueFormatted ?? String(params.value ?? '');
            return (
              <span className="flex items-center justify-between gap-1">
                {working === undefined ? <span /> : <CalcInfo working={working} inCell />}
                <span>{text}</span>
              </span>
            );
          },
        }
      : {}),
  };
}

/** The element of a cell, found by the row ID and the field, inside one grid. */
function cellElement(root: HTMLElement | null, cell: CellRef): HTMLElement | null {
  return (
    root?.querySelector<HTMLElement>(
      `.ag-row[row-id="${CSS.escape(cell.rowId)}"] .ag-cell[col-id="${CSS.escape(cell.field)}"]`,
    ) ?? null
  );
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
  cellWarnings,
  cellChanged,
  canRemoveRow,
  heightMode = 'fill',
  minRows,
  fit = false,
  workings,
  interceptors,
  onCardAccount,
}: EntryGridProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const direction = directionOf(language);
  const grouping = useGrouping();
  const rows = useRowsOf(store);
  const apiRef = useRef<GridApi<EntryRow> | null>(null);
  // The element of the grid, as state, so the working of a cell can be opened next to its cell.
  const [root, setRoot] = useState<HTMLDivElement | null>(null);
  const [ready, setReady] = useState(false);
  // The cell whose working is open (Ctrl+I), and the element the box opens next to.
  const [explaining, setExplaining] = useState<{ cell: CellRef; anchor: HTMLElement } | null>(null);

  const live = useLatest<Live>({ calc, workings, cellErrors, cellWarnings, cellChanged });

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

  const cycle = useMemo(
    () => (cell: CellRef) => {
      const column = columns.find((candidate) => candidate.field === cell.field);
      const choices = column?.choices;
      const row = store.getState().rows.find((candidate) => candidate.rowId === cell.rowId);
      if (choices === undefined || choices.length === 0 || row === undefined) return;
      const typed = row.typed[cell.field] ?? '';
      const shown = typed === '' ? (live().calc?.[cell.rowId]?.[cell.field] ?? '') : typed;
      const at = choices.findIndex((choice) => choice.value === shown);
      const next = choices[(at + 1) % choices.length];
      if (next !== undefined) store.getState().updateTyped(cell.rowId, cell.field, next.value);
    },
    [columns, store, live],
  );

  const explain = useMemo(
    () => (cell: CellRef) => {
      const anchor = cellElement(root, cell);
      if (anchor !== null && live().workings?.[cell.rowId]?.[cell.field] !== undefined) {
        setExplaining({ cell, anchor });
      }
      onExplain?.(cell);
    },
    [onExplain, root, live],
  );

  const keyHandler = useMemo(
    () =>
      createGridKeyHandler({
        store,
        columns,
        direction,
        copyAbove: copyAbove ?? false,
        fixedRows: fixedRows ?? false,
        ...(canRemoveRow === undefined ? {} : { canRemoveRow }),
        onExplain: explain,
        cycleChoice: cycle,
        ...(interceptors === undefined ? {} : { interceptors }),
      }),
    [store, columns, direction, copyAbove, explain, fixedRows, canRemoveRow, cycle, interceptors],
  );
  const getKeyHandler = useLatest(keyHandler);

  const mouseFocusTracker = useMouseFocus();
  const columnDefs = useMemo(
    () =>
      columns.map((column) =>
        toColDef(column, {
          mouseFocus: mouseFocusTracker,
          grouping,
          language,
          word: (id) => l10n.getString(id),
          store,
          live,
          fit,
          getKeyHandler,
          onCardAccount,
          cycle,
        }),
      ),
    [
      columns,
      l10n,
      grouping,
      language,
      store,
      fit,
      onCardAccount,
      cycle,
      live,
      getKeyHandler,
      mouseFocusTracker,
    ],
  );

  const defaultColDef = useMemo<ColDef<EntryRow>>(
    () => ({ suppressKeyboardEvent: keyHandler }),
    [keyHandler],
  );

  // A new answer from the backend: the cells read it again. Editing is not interrupted.
  useLayoutEffect(() => {
    apiRef.current?.refreshCells({ force: true });
  }, [calc, workings, cellErrors, cellWarnings, cellChanged]);

  // After a key or the screen asked for a cell, focus it once the rows are drawn.
  const pendingFocus = store((state) => state.pendingFocus);
  useEffect(() => {
    const api = apiRef.current;
    if (pendingFocus === null || api === null) return;
    store.getState().setPendingFocus(null);
    const rowIndex =
      pendingFocus.rowIndex === 'last'
        ? rows.length - 1
        : Math.min(pendingFocus.rowIndex, rows.length - 1);
    api.setFocusedCell(rowIndex, pendingFocus.field);
    api.ensureIndexVisible(rowIndex);
  }, [pendingFocus, rows, store, ready]);

  const onGridReady = (event: GridReadyEvent<EntryRow>) => {
    apiRef.current = event.api;
    setReady(true);
    // A new grid (for example after a language switch) focuses the cell that had focus.
    const focus = store.getState().focus;
    const rowIndex =
      focus === null ? -1 : store.getState().rows.findIndex((row) => row.rowId === focus.rowId);
    if (focus !== null && rowIndex >= 0) event.api.setFocusedCell(rowIndex, focus.field);
  };

  const onCellFocused = (event: CellFocusedEvent<EntryRow>) => {
    if (event.rowIndex === null || event.column === null || typeof event.column === 'string')
      return;
    // The grid says `cellFocused` for a press on the focused cell too; the tracker tells a move
    // of the focus by a mouse press from a press that stayed where it was.
    const source = event.sourceEvent?.type;
    mouseFocusTracker.noteFocus(
      { rowIndex: event.rowIndex, colId: event.column.getColId() },
      source === 'pointerdown' || source === 'mousedown' || source === 'touchstart',
    );
    const row = store.getState().rows[event.rowIndex];
    if (row !== undefined)
      store.getState().setFocus({ rowId: row.rowId, field: event.column.getColId() });
  };

  const explainedWorking =
    explaining === null ? undefined : workings?.[explaining.cell.rowId]?.[explaining.cell.field];

  return (
    <div
      ref={setRoot}
      className={`entry-grid ag-theme-quartz ${fit ? 'entry-grid-fit' : ''} ${minRows === 4 ? 'entry-grid-min4' : ''}`}
      role="group"
      aria-label={l10n.getString(labelId)}
      dir={direction}
      // Focus went to another part of the screen (the other grid, a field): this grid no longer
      // shows a focused cell, so only one thing on the screen looks focused (NFR-USE-22). The cell
      // is remembered in the store. Focus in the popup of a cell being typed in is not leaving.
      onBlur={(event) => {
        const next = event.relatedTarget;
        if (!(next instanceof Element) || event.currentTarget.contains(next)) return;
        if (next.closest('.ag-popup') !== null) return;
        if ((apiRef.current?.getEditingCells().length ?? 0) > 0) return;
        apiRef.current?.clearFocusedCell();
      }}
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
        // The list of a picker cell must be able to hang below a short grid, so popups are drawn
        // on the page and not inside the grid, which clips what leaves it.
        popupParent={document.body}
        onGridReady={onGridReady}
        onCellFocused={onCellFocused}
      />
      {explaining !== null && explainedWorking !== undefined ? (
        <CalcInfoAt
          working={explainedWorking}
          anchor={explaining.anchor}
          onClose={() => {
            setExplaining(null);
          }}
        />
      ) : null}
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
