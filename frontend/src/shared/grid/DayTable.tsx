// One row for each date, with a day that opens UNDER its own row (PRD FR-12.47, FR-9.3, FR-17.12, plans
// 74 and 76): the list of a party's statement, of the days of the commission page and of a partner's
// statement. One tab stop, like any tree (`treeKeyHandler`, Code Standards F27): ↑ ↓ Home End move
// between the dates, Enter, or the arrow that points at the children, opens the day under its row, the
// arrow that points at the parent shuts it, in both directions (FR-7.13). A click on a row opens it and a
// second click shuts it. Several days can be open; the screen shuts them when the window moves. Page Up
// and Page Down are the screen's: they move the window. What an open day shows is given by the screen
// (`renderDay`); keys inside it are its own and never the list's.
//
// The columns are the screen's: each has a heading (a Fluent message ID), a cell for a row, and a cell
// for the footer and for the lead row (the opening). A column that does not fit the compact step waits
// for `fromStep` (NFR-USE-15, F22). The figures in the cells are the backend's, drawn by the screen's own
// cell functions; this list adds up nothing (CLAUDE.md rule 1). Number columns, their headings and their
// totals are centred (F17, NFR-USE-19).

import { Localized } from '@fluent/react';
import { ChevronRight } from 'lucide-react';
import { useEffect, useId, useRef, type ReactNode } from 'react';
import { isFromStep, useLayoutStep, type LayoutStep } from '../hooks/useLayoutStep';
import { treeKeyHandler } from '../keyboard';

/** Rows moved by Page Up and Page Down in a plain tree; in a window the keys move the window instead. */
const TREE_PAGE_ROWS = 10;

export interface DayColumn<Row> {
  key: string;
  /** The Fluent message ID of the heading. */
  headerId?: string;
  /** The heading as it is, when it is not one message (a partner's name). Wins over `headerId`. */
  header?: ReactNode;
  /** What the cell of a row shows. */
  cell: (row: Row) => ReactNode;
  /** What the footer shows under the column (the first column usually says "میزان"). */
  footer?: ReactNode;
  /** What the lead row, before the first date, shows in the column (the opening). */
  lead?: ReactNode;
  /** The column waits for this step; below it the column is not drawn. Default: always. */
  fromStep?: LayoutStep;
}

interface DayTableProps<Row extends { date: string }> {
  columns: readonly DayColumn<Row>[];
  rows: readonly Row[];
  /**
   * What names a row: its date (`YYYY-MM-DD`) by default, one row for each date. A list with more than one row
   * on a date (a party's statement, one row for each entry, plan 78 B3) gives the row its own key.
   */
  rowKey?: (row: Row) => string;
  /** The keys of the rows that are open (the dates, `YYYY-MM-DD`, unless `rowKey` says otherwise). */
  open: ReadonlySet<string>;
  onToggle: (key: string, open: boolean) => void;
  /** The key of the row the keys are on, or null: the first one. */
  active: string | null;
  onActive: (key: string) => void;
  /** What an open row shows under it. It gets the row's key. */
  renderDay: (key: string) => ReactNode;
  /** Shown in place of the rows when there are none; the heading and the footer stay. */
  empty: ReactNode;
  /** A new window or page is on its way: the rows on screen are the last ones. */
  busy: boolean;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
  /** Draw the lead row (the opening) under the headings. */
  withLead?: boolean;
  /** Draw the footer row (the totals). Default: true. */
  withFooter?: boolean;
}

const CELL = 'min-w-0 text-center';

const BY_DATE = (row: { date: string }): string => row.date;

export function DayTable<Row extends { date: string }>({
  columns,
  rows,
  rowKey = BY_DATE,
  open,
  onToggle,
  active,
  onActive,
  renderDay,
  empty,
  busy,
  labelId,
  withLead = false,
  withFooter = true,
}: DayTableProps<Row>) {
  const listId = useId();
  const boxRef = useRef<HTMLDivElement | null>(null);
  const step = useLayoutStep();
  // At the compact step (1024 px, with both menus open) the list must never scroll sideways
  // (NFR-USE-15, F22): a column with a `fromStep` waits for its step.
  const shown = columns.filter(
    (column) => column.fromStep === undefined || isFromStep(step, column.fromStep),
  );
  const template = `repeat(${String(shown.length)}, minmax(0, 1fr))`;
  const activeIndex = Math.max(
    rows.findIndex((row) => rowKey(row) === active),
    0,
  );
  const current = rows[activeIndex];

  // Keep the date the keys are on in view while the arrows move through a long list.
  useEffect(() => {
    if (active === null) return;
    const row = document.getElementById(`${listId}-${active}`);
    if (row === null || row.getBoundingClientRect().height === 0) return;
    row.scrollIntoView({ block: 'nearest' });
  }, [active, listId]);

  const toggleCurrent = () => {
    if (current !== undefined) onToggle(rowKey(current), !open.has(rowKey(current)));
  };
  const onTreeKey = treeKeyHandler({
    count: rows.length,
    active: activeIndex,
    pageSize: TREE_PAGE_ROWS,
    onMove: (index) => {
      const row = rows[index];
      if (row !== undefined) onActive(rowKey(row));
    },
    onHorizontal: (action) => {
      if (current === undefined) return false;
      const isOpen = open.has(rowKey(current));
      // A flat list: the arrow towards the children opens the day, the one towards the parent shuts it.
      if (action === 'opens' && !isOpen) onToggle(rowKey(current), true);
      else if (action === 'closes' && isOpen) onToggle(rowKey(current), false);
      else return false;
      return true;
    },
    onEnter: toggleCurrent,
  });

  const cells = (render: (column: DayColumn<Row>) => ReactNode) => (
    <div
      className="grid min-w-0 flex-1 items-center gap-x-2"
      style={{ gridTemplateColumns: template }}
    >
      {shown.map((column) => (
        <span key={column.key} role="gridcell" className={CELL}>
          {render(column)}
        </span>
      ))}
    </div>
  );

  return (
    <div
      ref={boxRef}
      role="treegrid"
      tabIndex={0}
      aria-labelledby={`${listId}-label`}
      aria-busy={busy}
      aria-activedescendant={current === undefined ? undefined : `${listId}-${rowKey(current)}`}
      onKeyDown={(event) => {
        // A key inside an open day (its print button, its goods) is that day's own.
        if (event.target !== event.currentTarget) return;
        // Page Up and Page Down move the window: the screen's shortcut, not a page of rows.
        if (event.key === 'PageUp' || event.key === 'PageDown') return;
        onTreeKey(event);
      }}
      className={`flex flex-col overflow-hidden rounded-3 border border-border bg-surface ${busy ? 'opacity-70' : ''}`}
    >
      <Localized id={labelId}>
        <span id={`${listId}-label`} className="sr-only" />
      </Localized>
      <div
        role="row"
        className="flex items-center gap-3 border-b border-border bg-surface-sunken px-3 py-2"
      >
        <span aria-hidden="true" className="size-icon-md shrink-0" />
        <div
          className="grid min-w-0 flex-1 items-center gap-x-2"
          style={{ gridTemplateColumns: template }}
        >
          {shown.map((column) =>
            column.header !== undefined || column.headerId === undefined ? (
              <span
                key={column.key}
                role="columnheader"
                className="min-w-0 text-center type-label text-ink-muted"
              >
                {column.header}
              </span>
            ) : (
              <Localized key={column.key} id={column.headerId}>
                <span
                  role="columnheader"
                  className="min-w-0 text-center type-label text-ink-muted"
                />
              </Localized>
            ),
          )}
        </div>
      </div>
      {withLead ? (
        <div
          role="row"
          className="flex items-center gap-3 border-b border-border px-3 py-2 type-body text-ink-muted"
        >
          <span aria-hidden="true" className="size-icon-md shrink-0" />
          {cells((column) => column.lead)}
        </div>
      ) : null}
      {rows.length === 0 ? (
        <div className="px-3 py-6">{empty}</div>
      ) : (
        rows.map((row) => {
          const key = rowKey(row);
          const isOpen = open.has(key);
          const isActive = current !== undefined && key === rowKey(current);
          return (
            <div key={key}>
              <div
                id={`${listId}-${key}`}
                role="row"
                aria-expanded={isOpen}
                aria-selected={isActive}
                onClick={() => {
                  onActive(key);
                  onToggle(key, !isOpen);
                  boxRef.current?.focus();
                }}
                className={`flex min-h-target cursor-pointer items-center gap-3 border-b border-border px-3 py-2 type-body text-ink ${isActive ? 'bg-row-current' : ''}`}
              >
                <ChevronRight
                  aria-hidden="true"
                  className={`size-icon-md shrink-0 transition-calm ${isOpen ? 'rotate-90' : 'rtl:rotate-180'}`}
                />
                {cells((column) => column.cell(row))}
              </div>
              {isOpen ? (
                <div role="row">
                  <div
                    role="gridcell"
                    aria-colspan={shown.length}
                    data-day-detail=""
                    className="border-b border-border bg-surface-sunken p-3"
                  >
                    {renderDay(key)}
                  </div>
                </div>
              ) : null}
            </div>
          );
        })
      )}
      {withFooter ? (
        <div
          role="row"
          className="flex items-center gap-3 bg-surface-sunken px-3 py-2 type-body font-semibold text-ink"
        >
          <span aria-hidden="true" className="size-icon-md shrink-0" />
          {cells((column) => column.footer)}
        </div>
      ) : null}
    </div>
  );
}
