// The one grid for reading a list (masters, registers): AG Grid, set up once here, read-only
// (plan 06, PRD FR-7.17, NFR-SPD-18, Code Standards F18, F34). Only the rows in view are drawn,
// so thousands of rows stay smooth. Keys: ↑ ↓ move between rows, Page Up and Page Down move a
// screenful, or turn the page of a list with page buttons (`paging`), Enter (or a double click)
// opens the row, and Tab leaves the grid. Nothing is typed in it,
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
  IsFullWidthRowParams,
  SortChangedEvent,
  SuppressKeyboardEventParams,
  ValueGetterParams,
} from 'ag-grid-community';
import { AgGridReact } from 'ag-grid-react';
import type { MonthSubtotal } from '../../bindings/MonthSubtotal';
import type { SortInput } from '../../bindings/SortInput';
import { useCallback, useEffect, useMemo, useRef } from 'react';
import type { Language } from '../../i18n';
import { formatPaisa, formatPaisaValue, groupDigits, useGrouping, type Paisa } from '../format';
import { isFromStep, useLayoutStep, type LayoutStep } from '../hooks/useLayoutStep';
import { directionOf, useLanguage } from '../i18n';
import './agGridSetup';
import { FillCell, FillNumberCell, NumberRuns, NumberRunsCell, hasNumberRun } from './NumberRuns';
import {
  LIST_COLUMN_WIDTHS,
  LIST_GROW_MIN_WIDTH,
  isListNumberLike,
  type ListColumnType,
} from './columnTypes';

/** How many frames the grid waits for a new page's first row before it stops asking for focus. */
const FOCUS_RETRIES = 30;

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
   * A short word set after the figure, which stays in the middle of the column, whatever the word is
   * (a balance's side, بنام or جمع; NFR-USE-19, plan 39 R1). The total row asks for the
   * word of this column under the key `<field>:unit` of the totals values.
   */
  readonly unit?: ((row: T, context: ListContext) => string) | undefined;
  /**
   * A figure that reads as a loss is set in the due colour (R33: a trip that lost money). Colour is
   * never the only signal (NFR-USE-12): the minus sign is there, and a column with a word in `unit`
   * says "Loss". The total row asks for the tone under the key `<field>:tone`.
   */
  readonly tone?: ((row: T, context: ListContext) => 'due' | undefined) | undefined;
  /**
   * The name the backend sorts this column by (a field of its rows, plan 38 R4). A column with one
   * has a clickable title: the first click sorts ascending, the second descending, the third goes
   * back to the list's own order. The backend sorts the whole filter, never only the page. A name
   * column gives a function of the language (`name_ur` or `name_en`), so it sorts in the order of
   * the script on screen.
   */
  readonly sortKey?: string | ((language: Language) => string) | undefined;
}

/** The name the backend sorts a column by, in the language on screen. */
function sortKeyOf<T>(column: ListColumn<T>, language: Language): string | undefined {
  const key = column.sortKey;
  return typeof key === 'function' ? key(language) : key;
}

/** The order of a list chosen on a column title, and how to change it. */
export interface ListSort {
  value: SortInput | null;
  onChange: (next: SortInput | null) => void;
}

/**
 * A list in date order that shows a heading row above each month: the month, how many rows the whole
 * filter has in it, and the backend's subtotals (plan 38 R4). The subtotals are over the whole filter,
 * never added up from the rows on the page. `monthOf` gives the month of a row as `YYYY-MM`.
 */
export interface ListMonths<T> {
  months: readonly MonthSubtotal[];
  monthOf: (row: T) => string;
  /**
   * The column a subtotal of the backend is shown under, when its name is not the column's (the
   * register's `open` figure goes under the column of paid or due).
   */
  columnOf?: Readonly<Record<string, string>> | undefined;
}

/** The heading row of a month, among the rows of the grid. Never a record: no key opens it. */
interface MonthHeading {
  readonly monthHeading: MonthSubtotal;
}

function isMonthHeading(row: unknown): row is MonthHeading {
  return typeof row === 'object' && row !== null && 'monthHeading' in row;
}

/** The rows with a heading before the first row of each month the backend sent a subtotal for. */
function withMonthHeadings<T>(
  rows: readonly T[],
  subtotals: readonly MonthSubtotal[] | undefined,
  monthOf: ((row: T) => string) | undefined,
): (T | MonthHeading)[] {
  if (subtotals === undefined || monthOf === undefined || subtotals.length === 0) return [...rows];
  const made: (T | MonthHeading)[] = [];
  let previous: string | null = null;
  for (const row of rows) {
    const month = monthOf(row);
    if (month !== previous) {
      const found = subtotals.find((candidate) => candidate.month === month);
      if (found !== undefined) made.push({ monthHeading: found });
      previous = month;
    }
    made.push(row);
  }
  return made;
}

/** A list with page buttons: Page Up and Page Down turn the page (PRD FR-7.17, plan 38 R1). */
export interface ListPaging {
  /** A page turn asked for with a key. The screen moves the page. */
  onPage: (direction: 'previous' | 'next') => void;
  /**
   * Counts the page turns made with the keys. When it grows and `ready` is true, focus goes to the
   * first row of the page that is on screen then.
   */
  focusFirst: number;
  /** The page asked for has arrived. */
  ready: boolean;
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
  /** For a list with page buttons: Page Up and Page Down turn the page. */
  paging?: ListPaging | undefined;
  /** The order of the list: a click on the title of a column with a `sortKey` changes it. */
  sort?: ListSort | undefined;
  /** A heading row above each month, for a list in date order (plan 38 R4). */
  months?: ListMonths<T> | undefined;
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
  sort: { value: SortInput | null } | undefined,
  /** The rows of this list grow with a long column: the other cells keep their words in the middle. */
  growing: boolean,
): ColDef<T> {
  const unitOf = column.unit;
  const sortKey = sortKeyOf(column, context.language);
  const sortable = sort !== undefined && sortKey !== undefined;
  return {
    colId: column.field,
    headerName,
    width: LIST_COLUMN_WIDTHS[column.type],
    ...(column.grow === true ? { flex: 1, minWidth: LIST_GROW_MIN_WIDTH } : {}),
    resizable: false,
    // The backend sorts the whole filter; the grid only shows the arrow and tells the screen. The
    // comparator never moves a row, so the order of the answer stays (a browser sort cannot do the
    // Urdu order either, NFR-URD-19).
    sortable,
    ...(sortable
      ? {
          comparator: () => 0,
          sortingOrder: ['asc', 'desc', null] as ('asc' | 'desc' | null)[],
          sort: sort.value?.column === sortKey ? sort.value.direction : null,
        }
      : {}),
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
      // A name: words at the start edge, even when it is made of digits (a trip is its number and day).
      ...(column.type === 'name' ? ['cell-name'] : []),
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
    // The heading of a number column is centred over its figures (NFR-USE-19, plan 39 R1).
    headerClass: isListNumberLike(column.type) ? ['header-num'] : [],
    ...(column.script === undefined
      ? {}
      : {
          cellRenderer: (params: ICellRendererParams<T>) => {
            const words = (
              <span lang={column.script?.(context.language)} className="cell-lang">
                {typeof params.value === 'string' ? <NumberRuns text={params.value} /> : ''}
              </span>
            );
            return growing ? <span className="cell-fill">{words}</span> : words;
          },
        }),
    // A text cell with a number inside keeps it in order and in Inter in Urdu (F25). Only a cell that
    // holds a number gets the renderer; every other cell is drawn by the grid as before. In a list
    // whose rows grow with a long column, every other cell is put in the middle of its row too.
    ...(column.script === undefined && unitOf === undefined && !isListNumberLike(column.type)
      ? growing
        ? { cellRenderer: FillCell }
        : {
            cellRendererSelector: (params: ICellRendererParams<T>) =>
              hasNumberRun(params.value) ? { component: NumberRunsCell } : undefined,
          }
      : {}),
    ...(growing &&
    column.script === undefined &&
    unitOf === undefined &&
    isListNumberLike(column.type)
      ? { cellRenderer: FillNumberCell }
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
            const figure = (
              <span className="cell-with-unit">
                <span>{typeof params.value === 'string' ? params.value : ''}</span>
                <span className="cell-unit" dir="auto">
                  {unit}
                </span>
              </span>
            );
            return growing ? <span className="cell-fill">{figure}</span> : figure;
          },
        }),
  };
}

/** What a month heading needs to say: the month, its count and its subtotals under their names. */
interface MonthHeadingText {
  title: string;
  count: string;
  figures: readonly { key: string; label: string; value: string }[];
}

/** The heading row of a month across the whole row (plan 38 R4): stronger, on the sunken ground. */
function MonthHeadingCell(
  props: ICellRendererParams & { describe: (heading: MonthSubtotal) => MonthHeadingText },
) {
  const data: unknown = props.data;
  if (!isMonthHeading(data)) return null;
  const text = props.describe(data.monthHeading);
  return (
    <div className="month-heading">
      <span className="month-heading-title">
        <NumberRuns text={text.title} />
      </span>
      <span className="month-heading-count">
        <NumberRuns text={text.count} />
      </span>
      {text.figures.map((figure) => (
        <span key={figure.key} className="month-heading-figure">
          <span className="month-heading-label">{figure.label}</span>
          <bdi dir="ltr" className="number-run">
            {figure.value}
          </bdi>
        </span>
      ))}
    </div>
  );
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
  paging,
  sort,
  months,
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
  // The rows with the heading of each month among them, when the list is in date order.
  // The parts are used, not the `months` object the screen makes anew on every render.
  const monthSubtotals = months?.months;
  const monthOf = months?.monthOf;
  const monthColumnOf = months?.columnOf;
  const displayRows = useMemo(
    () => withMonthHeadings(rows, monthSubtotals, monthOf),
    [rows, monthSubtotals, monthOf],
  );
  /** The words of a month heading: the month, its count and the subtotals under their column names. */
  const describeMonth = useCallback(
    (heading: MonthSubtotal) => {
      const [year = '', month = ''] = heading.month.split('-');
      const figures = heading.figures.flatMap((figure) => {
        const field = monthColumnOf?.[figure.key] ?? figure.key;
        const column = shown.find((candidate) => candidate.field === field);
        if (column === undefined) return [];
        return [
          {
            key: figure.key,
            label: column.headerText?.(context) ?? l10n.getString(column.headerId),
            value: figure.is_money
              ? formatPaisaValue(figure.value, { grouping })
              : groupDigits(String(figure.value), grouping),
          },
        ];
      });
      return {
        title: `${l10n.getString(`report-month-${String(Number(month))}`)} ${year}`,
        count: l10n.getString('common-month-count', { count: String(heading.count) }),
        figures,
      };
    },
    [shown, context, l10n, grouping, monthColumnOf],
  );
  // The total row exists while one of its columns is on screen. The word "Total" stands in the first
  // text column, never in a column of figures (plan 39 R6): a screen that names a column of figures
  // as the label's place has the word moved to the first text column that has no total of its own.
  const total = useMemo(() => {
    const made = totals?.(context);
    if (made === undefined) return undefined;
    const visible = shown.some((column) => made.values[column.field] !== undefined);
    if (!visible) return undefined;
    const label = made.values[made.labelField] ?? '';
    const labelColumn = shown.find((column) => column.field === made.labelField);
    const home =
      labelColumn !== undefined && isListNumberLike(labelColumn.type)
        ? shown.find(
            (column) => !isListNumberLike(column.type) && made.values[column.field] === undefined,
          )
        : undefined;
    if (home === undefined) return { ...made.values, [made.labelField]: label };
    return { ...made.values, [made.labelField]: '', [home.field]: label };
  }, [totals, context, shown]);
  // The grid takes the current order, not the `sort` object the screen makes anew on every render:
  // new column definitions on every render would rebuild the headings under a click.
  const sortOn = sort !== undefined;
  const sortValue = sort?.value ?? null;
  const sortState = useMemo(() => (sortOn ? { value: sortValue } : undefined), [sortOn, sortValue]);
  const growing = shown.some((column) => column.wrap === true);
  const columnDefs = useMemo(
    () =>
      shown.map((column) =>
        toColDef(
          column,
          column.headerText?.(context) ?? l10n.getString(column.headerId),
          context,
          total,
          sortState,
          growing,
        ),
      ),
    [shown, l10n, context, total, sortState, growing],
  );
  // The total row and the columns can change together (the "More columns" tick of the account
  // list). The grid then draws the new total row with the value getters it had before, so the
  // word in its label cell stayed empty. Drawing the cells again after the change fixes that.
  useEffect(() => {
    apiRef.current?.refreshCells({ force: true });
  }, [columnDefs]);

  // A page turned with Page Up or Page Down puts focus on the first row of the new page, once it
  // is on screen. A grid drawn later (a language switch) starts with the turns it has already seen.
  const turnsSeen = useRef(paging?.focusFirst ?? 0);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const focusFrame = useRef(0);
  const firstField = columns[0]?.field;
  useEffect(() => {
    if (paging === undefined || firstField === undefined) return;
    if (paging.focusFirst === turnsSeen.current || !paging.ready) return;
    turnsSeen.current = paging.focusFirst;
    // The cell that had focus was drawn again with the old page, so the browser has lost its focus
    // and the next Page Down would reach nothing. The new row is drawn a moment later, so focus is
    // asked for again until it is inside the grid (a few frames at most).
    let frames = 0;
    // The first row of records: a month heading above it is not a place to type or open.
    const firstIndex = (): number => {
      const api = apiRef.current;
      if (api === null) return 0;
      for (let index = 0; index < api.getDisplayedRowCount(); index += 1) {
        if (!isMonthHeading(api.getDisplayedRowAtIndex(index)?.data)) return index;
      }
      return 0;
    };
    const focusFirstRow = () => {
      const index = firstIndex();
      apiRef.current?.setFocusedCell(index, firstField);
      apiRef.current?.ensureIndexVisible(index);
      // The grid marks the cell but leaves the browser focus where it lost it: it is put on the
      // marked cell, so the next key reaches the grid.
      rootRef.current?.querySelector<HTMLElement>('.ag-cell-focus')?.focus();
    };
    // Focus is settled when it is inside the grid and on the first row. The grid keeps the cells
    // of the rows it draws again, so the cell that was clicked on the old page can still hold the
    // browser focus while it now shows another row.
    const focusIsOnFirstRow = (): boolean => {
      const root = rootRef.current;
      if (!root?.contains(document.activeElement)) return false;
      const row = root.querySelector('.ag-cell-focus')?.closest('.ag-row');
      return row?.getAttribute('row-index') === String(firstIndex());
    };
    const settle = () => {
      if (focusIsOnFirstRow() || frames >= FOCUS_RETRIES) {
        return;
      }
      frames += 1;
      focusFirstRow();
      focusFrame.current = requestAnimationFrame(settle);
    };
    focusFirstRow();
    focusFrame.current = requestAnimationFrame(settle);
  }, [paging, firstField, rows]);
  useEffect(
    () => () => {
      cancelAnimationFrame(focusFrame.current);
    },
    [],
  );

  // The page turn of the screen is made anew on every render. It is read at the moment of the key,
  // so the options handed to the grid below stay the same object: a new `defaultColDef` on every
  // render made the grid rebuild its columns, and a click on a title or a row was lost (plan 38).
  const onPageRef = useRef(paging?.onPage);
  useEffect(() => {
    onPageRef.current = paging?.onPage;
  });
  const suppressKeyboardEvent = useCallback((params: SuppressKeyboardEventParams<T>) => {
    const key = params.event;
    const onPage = onPageRef.current;
    if (onPage === undefined || key.ctrlKey || key.altKey || key.metaKey || key.shiftKey) {
      return false;
    }
    if (key.key !== 'PageDown' && key.key !== 'PageUp') return false;
    // The grid would scroll a screenful; the list turns its page instead.
    key.preventDefault();
    if (key.type === 'keydown') onPage(key.key === 'PageDown' ? 'next' : 'previous');
    return true;
  }, []);
  const defaultColDef = useMemo(() => ({ suppressKeyboardEvent }), [suppressKeyboardEvent]);
  const isFullWidthRow = useCallback(
    (params: IsFullWidthRowParams<T>) => isMonthHeading(params.rowNode.data),
    [],
  );
  const fullWidthParams = useMemo(() => ({ describe: describeMonth }), [describeMonth]);

  const onGridReady = (event: GridReadyEvent<T>) => {
    apiRef.current = event.api;
    const first = columns[0]?.field;
    const wanted = readFocusRow?.() ?? focusRowId;
    if (wanted === null || wanted === undefined || first === undefined) return;
    let index = -1;
    event.api.forEachNode((node, position) => {
      if (node.data !== undefined && !isMonthHeading(node.data) && getRowId(node.data) === wanted) {
        index = position;
      }
    });
    if (index >= 0) {
      event.api.setFocusedCell(index, first);
      event.api.ensureIndexVisible(index);
    }
  };

  // A click on a column title: the grid has marked the column, and the screen asks the backend for
  // the new order. Only a click counts, not the order the screen itself puts back.
  const onSortChanged = (event: SortChangedEvent<T>) => {
    if (sort === undefined || event.source !== 'uiColumnSorted') return;
    const marked = event.api.getColumnState().find((state) => state.sort != null);
    const column = columns.find((candidate) => candidate.field === marked?.colId);
    const key = column === undefined ? undefined : sortKeyOf(column, language);
    if (marked?.sort == null || key === undefined) {
      sort.onChange(null);
      return;
    }
    // The script of the names on screen goes with the sort: Urdu names sort in Urdu order.
    sort.onChange({ column: key, direction: marked.sort, language });
  };

  const onCellKeyDown = (event: CellKeyDownEvent<T>) => {
    const key = event.event;
    if (!(key instanceof KeyboardEvent) || key.key !== 'Enter' || event.data === undefined) return;
    // A month heading opens nothing.
    if (isMonthHeading(event.data)) return;
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
      ref={rootRef}
      className={`entry-grid list-grid ag-theme-quartz ${heightMode === 'fill' ? 'list-grid-fill' : ''} ${growing ? 'list-grid-wrap' : ''}`}
      role="group"
      aria-label={l10n.getString(labelId)}
      dir={direction}
    >
      <AgGridReact<T>
        theme="legacy"
        enableRtl={direction === 'rtl'}
        // A heading row is not a record: it is drawn across the whole row by the renderer below.
        rowData={displayRows as T[]}
        {...(months === undefined
          ? {}
          : {
              isFullWidthRow,
              fullWidthCellRenderer: MonthHeadingCell,
              fullWidthCellRendererParams: fullWidthParams,
            })}
        pinnedBottomRowData={total === undefined ? [] : [{} as T]}
        columnDefs={columnDefs}
        defaultColDef={defaultColDef}
        getRowId={(params) =>
          isMonthHeading(params.data)
            ? `month:${params.data.monthHeading.month}`
            : getRowId(params.data)
        }
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
            !isMonthHeading(params.data) &&
            !params.node.isRowPinned() &&
            isInactive?.(params.data) === true,
          'row-group': (params) =>
            params.data !== undefined &&
            (isMonthHeading(params.data) ||
              (!params.node.isRowPinned() && isGroup?.(params.data) === true)),
        }}
        onGridReady={onGridReady}
        onSortChanged={onSortChanged}
        onCellKeyDown={onCellKeyDown}
        onCellDoubleClicked={(event) => {
          if (event.data === undefined || isMonthHeading(event.data)) return;
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
          if (row !== undefined && !isMonthHeading(row)) onFocusRow(getRowId(row));
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
