// The one grid for reading a list (masters, registers): AG Grid, set up once here, read-only
// (plan 06, PRD FR-7.17, NFR-SPD-18, Code Standards F18, F34). Only the rows in view are drawn,
// so thousands of rows stay smooth. Keys: ↑ ↓ move between rows, Page Up and Page Down move a
// page, Enter (or a double click) opens the row, and Tab leaves the grid. Nothing is typed in it,
// so a language switch only rebuilds it (`enableRtl` is fixed when a grid is created) and the
// row that had focus is focused again. Sorting is off: the backend sends the rows in the right
// order, with Urdu letters in Urdu order (NFR-URD-19), which a browser sort cannot do.

import { useLocalization } from '@fluent/react';
import type {
  CellClassParams,
  CellKeyDownEvent,
  ColDef,
  GridApi,
  GridReadyEvent,
  ICellRendererParams,
  ValueGetterParams,
} from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
import { useEffect, useMemo, useRef } from 'react';
import type { Language } from '../../i18n';
import { formatPaisa, useGrouping, type Paisa } from '../format';
import { isFromStep, useLayoutStep, type LayoutStep } from '../hooks/useLayoutStep';
import { directionOf, useLanguage } from '../i18n';
import './agGridSetup';
import { NumberRuns, NumberRunsCell, hasNumberRun } from './NumberRuns';
import {
  LIST_COLUMN_WIDTHS,
  LIST_GROW_MIN_WIDTH,
  isListNumberLike,
  type ListColumnType,
} from './columnTypes';

/** What a column needs to turn a row into text: the language, Fluent, and money formatting. */
export interface ListContext {
  language: Language;
  /** A Fluent message ID as words in the app language (the label of a choice, for example), with its variables. */
  text: (id: string, vars?: Record<string, string>) => string;
  /**
   * An amount from the backend as whole rupees with the app's grouping (never worked out here). With
   * `plain` the separators are left out, for a rate inside a line of words ("2600", PRD 19.2 R5).
   */
  money: (amount: Paisa, plain?: boolean) => string;
}

export interface ListColumn<T> {
  /** The ID of the column. */
  readonly field: string;
  readonly type: ListColumnType;
  /** The Fluent message ID of the header. */
  readonly headerId: string;
  /**
   * The words of the header when they are not one message: the heading of a column that stands for
   * a day, a month or an item (analytics, plan 21). Names go through `displayName` (F38).
   */
  readonly headerText?: ((context: ListContext) => string) | undefined;
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
  /**
   * The column takes the free width of the grid, and gives it up again when the grid is narrow
   * (down to `LIST_GROW_MIN_WIDTH`). Several columns can grow; they share the free width.
   */
  readonly grow?: boolean | undefined;
  /**
   * A long text (the goods behind a ledger line) wraps onto more lines and the row grows with it,
   * instead of being cut off. Used by report columns; short columns never wrap.
   */
  readonly wrap?: boolean | undefined;
  /**
   * A short word set after the figure in a slot of fixed width, so figures keep one right edge
   * whatever the word is (a balance's side, بنام or جمع; NFR-USE-19). The total row asks for the
   * word of this column under the key `<field>:unit` of the totals values.
   */
  readonly unit?: ((row: T, context: ListContext) => string) | undefined;
  /**
   * A figure that reads as a loss is set in the due colour (R33: a trip that lost money). Colour is
   * never the only signal (NFR-USE-12): the minus sign is there, and a column with a word in `unit`
   * says "Loss". The total row asks for the tone under the key `<field>:tone`.
   */
  readonly tone?: ((row: T, context: ListContext) => 'due' | undefined) | undefined;
}

interface ListGridProps<T> {
  columns: readonly ListColumn<T>[];
  rows: readonly T[];
  /** A stable ID of the row, never its position. */
  getRowId: (row: T) => string;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
  /**
   * Enter or a double click on a row. `column` is the column of the cell that had focus or was
   * clicked, for a list whose cells open different records (the lines behind an analytics number).
   */
  onOpen: (row: T, column?: string) => void;
  /**
   * Enter or a double click on the pinned totals row, with the column of the cell. Without it the
   * totals row opens nothing.
   */
  onOpenTotals?: ((column: string) => void) | undefined;
  /** A row that is switched off reads dimmer. */
  isInactive?: ((row: T) => boolean) | undefined;
  /** A row that heads a group (a head in a list of heads) reads stronger. */
  isGroup?: ((row: T) => boolean) | undefined;
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
  /**
   * A total row pinned under the last row, with each figure under its column (Code Standards F19).
   * It gives the column that holds the word and the text of the columns that have a total. The
   * figures are the backend's, passed through `context.money`. Shown only while at least one of
   * its columns is on screen, so a layout step without them keeps the totals where the screen put
   * them.
   */
  totals?:
    ((context: ListContext) => { labelField: string; values: Record<string, string> }) | undefined;
}

function toColDef<T>(
  column: ListColumn<T>,
  headerName: string,
  context: ListContext,
  total: Record<string, string> | undefined,
): ColDef<T> {
  const unitOf = column.unit;
  return {
    colId: column.field,
    headerName,
    width: LIST_COLUMN_WIDTHS[column.type],
    ...(column.grow === true ? { flex: 1, minWidth: LIST_GROW_MIN_WIDTH } : {}),
    resizable: false,
    sortable: false,
    suppressMovable: true,
    suppressHeaderMenuButton: true,
    // A long header (for example "Credit allowed") wraps instead of being cut off.
    wrapHeaderText: true,
    autoHeaderHeight: true,
    valueGetter: (params: ValueGetterParams<T>) =>
      params.node?.rowPinned === 'bottom'
        ? (total?.[column.field] ?? '')
        : params.data === undefined
          ? ''
          : column.value(params.data, context),
    cellClass: [
      ...(isListNumberLike(column.type) ? ['cell-num'] : []),
      ...(column.wrap === true ? ['cell-wrap'] : []),
    ],
    ...(column.tone === undefined
      ? {}
      : {
          cellClassRules: {
            'cell-tone-due': (params: CellClassParams<T>) =>
              params.node.rowPinned === 'bottom'
                ? total?.[`${column.field}:tone`] === 'due'
                : params.data !== undefined && column.tone?.(params.data, context) === 'due',
          },
        }),
    ...(column.wrap === true ? { wrapText: true, autoHeight: true } : {}),
    // The heading of a number column sits on the same edge as its figures (NFR-USE-19).
    headerClass: [
      ...(isListNumberLike(column.type) ? ['header-num'] : []),
      // The figure is set back from the edge by the slot of its word, so its heading is too.
      ...(column.unit === undefined ? [] : ['header-unit']),
    ],
    ...(column.script === undefined
      ? {}
      : {
          cellRenderer: (params: ICellRendererParams<T>) => (
            <span lang={column.script?.(context.language)} className="cell-lang">
              {typeof params.value === 'string' ? <NumberRuns text={params.value} /> : ''}
            </span>
          ),
        }),
    // A text cell with a number inside keeps it in order and in Inter in Urdu (F25). Only a cell that
    // holds a number gets the renderer; every other cell is drawn by the grid as before.
    ...(column.script === undefined && unitOf === undefined && !isListNumberLike(column.type)
      ? {
          cellRendererSelector: (params: ICellRendererParams<T>) =>
            hasNumberRun(params.value) ? { component: NumberRunsCell } : undefined,
        }
      : {}),
    ...(unitOf === undefined
      ? {}
      : {
          cellRenderer: (params: ICellRendererParams<T>) => {
            const unit =
              params.node.rowPinned === 'bottom'
                ? (total?.[`${column.field}:unit`] ?? '')
                : params.data === undefined
                  ? ''
                  : unitOf(params.data, context);
            return (
              <span className="cell-with-unit">
                <span>{typeof params.value === 'string' ? params.value : ''}</span>
                <span className="cell-unit" dir="auto">
                  {unit}
                </span>
              </span>
            );
          },
        }),
  };
}

function GridInner<T>({
  columns,
  rows,
  getRowId,
  labelId,
  onOpen,
  onOpenTotals,
  isInactive,
  isGroup,
  focusRowId,
  readFocusRow,
  onFocusRow,
  heightMode = 'fill',
  totals,
}: ListGridProps<T>) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const direction = directionOf(language);
  const apiRef = useRef<GridApi<T> | null>(null);

  const grouping = useGrouping();
  const step = useLayoutStep();
  const shown = useMemo(
    () => columns.filter((column) => isFromStep(step, column.from ?? 'compact')),
    [columns, step],
  );
  const context = useMemo<ListContext>(
    () => ({
      language,
      text: (id, vars) => l10n.getString(id, vars),
      money: (amount, plain = false) => formatPaisa(amount, { grouping, plain }),
    }),
    [language, l10n, grouping],
  );
  // The total row exists while one of its columns is on screen.
  const total = useMemo(() => {
    const made = totals?.(context);
    if (made === undefined) return undefined;
    const visible = shown.some((column) => made.values[column.field] !== undefined);
    return visible
      ? { ...made.values, [made.labelField]: made.values[made.labelField] ?? '' }
      : undefined;
  }, [totals, context, shown]);
  const columnDefs = useMemo(
    () =>
      shown.map((column) =>
        toColDef(
          column,
          column.headerText?.(context) ?? l10n.getString(column.headerId),
          context,
          total,
        ),
      ),
    [shown, l10n, context, total],
  );
  // The total row and the columns can change together (the "More columns" tick of the account
  // list). The grid then draws the new total row with the value getters it had before, so the
  // word in its label cell stayed empty. Drawing the cells again after the change fixes that.
  useEffect(() => {
    apiRef.current?.refreshCells({ force: true });
  }, [columnDefs]);

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
    const column = event.column.getColId();
    // The totals row is pinned and has no record behind it: it opens only what the screen offers.
    if (event.node.isRowPinned()) {
      onOpenTotals?.(column);
      return;
    }
    onOpen(event.data, column);
  };

  return (
    <div
      className={`entry-grid ag-theme-quartz ${heightMode === 'fill' ? 'list-grid-fill' : ''}`}
      role="group"
      aria-label={l10n.getString(labelId)}
      dir={direction}
    >
      <AgGridReact<T>
        theme="legacy"
        enableRtl={direction === 'rtl'}
        rowData={rows as T[]}
        pinnedBottomRowData={total === undefined ? [] : [{} as T]}
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
          // The totals row is a pinned row with no record behind it: no class rule reads it.
          'row-inactive': (params) =>
            params.data !== undefined &&
            !params.node.isRowPinned() &&
            isInactive?.(params.data) === true,
          'row-group': (params) =>
            params.data !== undefined &&
            !params.node.isRowPinned() &&
            isGroup?.(params.data) === true,
        }}
        onGridReady={onGridReady}
        onCellKeyDown={onCellKeyDown}
        onCellDoubleClicked={(event) => {
          if (event.data === undefined) return;
          const column = event.column.getColId();
          if (event.node.isRowPinned()) {
            onOpenTotals?.(column);
            return;
          }
          onOpen(event.data, column);
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
