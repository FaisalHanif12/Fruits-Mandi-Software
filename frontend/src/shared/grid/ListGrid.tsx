// The one grid for reading a list (masters, registers): AG Grid, set up once here, read-only
// (plan 06, PRD FR-7.17, NFR-SPD-18, Code Standards F18, F34). Only the rows in view are drawn,
// so thousands of rows stay smooth. Keys: ↑ ↓ move between rows, Page Up and Page Down move a
// page, Enter (or a double click) opens the row, and Tab leaves the grid. Nothing is typed in it,
// so a language switch only rebuilds it (`enableRtl` is fixed when a grid is created) and the
// row that had focus is focused again. Sorting is off: the backend sends the rows in the right
// order, with Urdu letters in Urdu order (NFR-URD-19), which a browser sort cannot do.

import { useLocalization } from '@fluent/react';
import type {
  CellKeyDownEvent,
  ColDef,
  GridApi,
  GridReadyEvent,
  ICellRendererParams,
  ValueGetterParams,
} from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
import { useMemo, useRef } from 'react';
import type { Language } from '../../i18n';
import { formatPaisa, useGrouping, type Paisa } from '../format';
import { isFromStep, useLayoutStep, type LayoutStep } from '../hooks/useLayoutStep';
import { directionOf, useLanguage } from '../i18n';
import './agGridSetup';
import { LIST_COLUMN_WIDTHS, isListNumberLike, type ListColumnType } from './columnTypes';

/** What a column needs to turn a row into text: the language, Fluent, and money formatting. */
export interface ListContext {
  language: Language;
  /** A Fluent message ID as words in the app language (the label of a choice, for example), with its variables. */
  text: (id: string, vars?: Record<string, string>) => string;
  /** An amount from the backend as whole rupees with the app's grouping (never worked out here). */
  money: (amount: Paisa) => string;
}

export interface ListColumn<T> {
  /** The ID of the column. */
  readonly field: string;
  readonly type: ListColumnType;
  /** The Fluent message ID of the header. */
  readonly headerId: string;
  /** The text of the cell, already for display. Names go through `displayName` (F38). */
  readonly value: (row: T, context: ListContext) => string;
  /**
   * The script of the text, when it is not the language of the app: the other script's name
   * (an Urdu name in the English app) reads in its own font and size, with its own `lang`
   * (NFR-USE-20, NFR-URD-10).
   */
  readonly script?: ((language: Language) => Language) | undefined;
  /**
   * The narrowest layout step this column is shown at. Less used columns come with the wider
   * steps, so a list never scrolls sideways (NFR-USE-15, Code Standards F22). Default: compact.
   */
  readonly from?: LayoutStep | undefined;
  /** The column takes the free width of the grid (at least its own width). One per list. */
  readonly grow?: boolean | undefined;
}

interface ListGridProps<T> {
  columns: readonly ListColumn<T>[];
  rows: readonly T[];
  /** A stable ID of the row, never its position. */
  getRowId: (row: T) => string;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
  /** Enter or a double click on a row. */
  onOpen: (row: T) => void;
  /** A row that is switched off reads dimmer. */
  isInactive?: ((row: T) => boolean) | undefined;
  /** The row to focus when the grid is first drawn, for example the one just edited. */
  focusRowId?: string | null | undefined;
  /** Asked when the grid is drawn: the row that had focus before a rebuild (see ListGrid). */
  readFocusRow?: (() => string | null) | undefined;
  /** Tells the screen which row has focus, so a rebuilt grid can focus it again. */
  onFocusRow?: ((rowId: string) => void) | undefined;
  /**
   * 'fill': the parent gives the grid its height and only visible rows are drawn. 'auto': the
   * grid is as tall as its rows and draws them all (short lists, tests).
   */
  heightMode?: 'fill' | 'auto';
}

function toColDef<T>(column: ListColumn<T>, headerName: string, context: ListContext): ColDef<T> {
  return {
    colId: column.field,
    headerName,
    width: LIST_COLUMN_WIDTHS[column.type],
    ...(column.grow === true ? { flex: 1, minWidth: LIST_COLUMN_WIDTHS[column.type] } : {}),
    resizable: false,
    sortable: false,
    suppressMovable: true,
    suppressHeaderMenuButton: true,
    valueGetter: (params: ValueGetterParams<T>) =>
      params.data === undefined ? '' : column.value(params.data, context),
    cellClass: isListNumberLike(column.type) ? ['cell-num'] : [],
    ...(column.script === undefined
      ? {}
      : {
          cellRenderer: (params: ICellRendererParams<T>) => (
            <span lang={column.script?.(context.language)} className="cell-lang">
              {typeof params.value === 'string' ? params.value : ''}
            </span>
          ),
        }),
  };
}

function GridInner<T>({
  columns,
  rows,
  getRowId,
  labelId,
  onOpen,
  isInactive,
  focusRowId,
  readFocusRow,
  onFocusRow,
  heightMode = 'fill',
}: ListGridProps<T>) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const direction = directionOf(language);
  const apiRef = useRef<GridApi<T> | null>(null);

  const grouping = useGrouping();
  const step = useLayoutStep();
  const columnDefs = useMemo(() => {
    const context: ListContext = {
      language,
      text: (id, vars) => l10n.getString(id, vars),
      money: (amount) => formatPaisa(amount, { grouping }),
    };
    return columns
      .filter((column) => isFromStep(step, column.from ?? 'compact'))
      .map((column) => toColDef(column, l10n.getString(column.headerId), context));
  }, [columns, l10n, language, grouping, step]);

  const onGridReady = (event: GridReadyEvent<T>) => {
    apiRef.current = event.api;
    const first = columns[0]?.field;
    const wanted = readFocusRow?.() ?? focusRowId;
    if (wanted === null || wanted === undefined || first === undefined) return;
    let index = -1;
    event.api.forEachNode((node, position) => {
      if (node.data !== undefined && getRowId(node.data) === wanted) index = position;
    });
    if (index >= 0) {
      event.api.setFocusedCell(index, first);
      event.api.ensureIndexVisible(index);
    }
  };

  const onCellKeyDown = (event: CellKeyDownEvent<T>) => {
    const key = event.event;
    if (!(key instanceof KeyboardEvent) || key.key !== 'Enter' || event.data === undefined) return;
    key.preventDefault();
    onOpen(event.data);
  };

  return (
    <div
      className={`entry-grid list-grid ag-theme-quartz ${heightMode === 'fill' ? 'list-grid-fill' : ''}`}
      role="group"
      aria-label={l10n.getString(labelId)}
      dir={direction}
    >
      <AgGridReact<T>
        theme="legacy"
        enableRtl={direction === 'rtl'}
        rowData={rows as T[]}
        columnDefs={columnDefs}
        getRowId={(params) => getRowId(params.data)}
        domLayout={heightMode === 'auto' ? 'autoHeight' : 'normal'}
        suppressColumnVirtualisation
        suppressNoRowsOverlay
        animateRows={false}
        suppressMovableColumns
        // Tab leaves the grid like it leaves any field; the arrows move inside it.
        tabToNextCell={() => false}
        tabToNextHeader={() => false}
        rowClassRules={{
          'row-inactive': (params) =>
            params.data !== undefined && isInactive?.(params.data) === true,
        }}
        onGridReady={onGridReady}
        onCellKeyDown={onCellKeyDown}
        onRowDoubleClicked={(event) => {
          if (event.data !== undefined) onOpen(event.data);
        }}
        onCellFocused={(event) => {
          if (event.rowIndex === null || onFocusRow === undefined) return;
          const row = apiRef.current?.getDisplayedRowAtIndex(event.rowIndex)?.data;
          if (row !== undefined) onFocusRow(getRowId(row));
        }}
      />
    </div>
  );
}

/** The list grid. It is rebuilt when the language changes, and the focused row is focused again. */
export function ListGrid<T>(props: ListGridProps<T>) {
  const language = useLanguage();
  // The row that has focus, kept here so it survives the rebuild (a new grid starts empty).
  const focused = useRef<string | null>(props.focusRowId ?? null);
  const { onFocusRow } = props;
  return (
    <GridInner<T>
      key={language}
      {...props}
      readFocusRow={() => focused.current}
      onFocusRow={(rowId) => {
        focused.current = rowId;
        onFocusRow?.(rowId);
      }}
    />
  );
}
