// The dates of one party's statement, one row for each date with an entry, newest first, with a day
// that opens UNDER its own row (PRD FR-12.47, plan 74 W3, W6). One tab stop, like any tree
// (`treeKeyHandler`, Code Standards F27): ↑ ↓ Home End move between the dates, Enter, or the arrow that
// points at the children, opens the day under its row, the arrow that points at the parent shuts it, in
// both directions (FR-7.13). A click on a row opens it and a second click shuts it. Several days can be
// open; the screen shuts them when the window moves. Page Up and Page Down are the screen's: they move
// the window of 30 days. What an open day shows is given by the screen (`renderDay`): the R35 slip of
// that day, with its own print button; keys inside it are its own and never the list's.
// The footer row is the window's, from the backend: the balance at its start, what it charged, what
// was paid, the balance at its end. This list adds up nothing (CLAUDE.md rule 1). Number columns, their
// headings and their totals are centred (F17, NFR-USE-19). Up to 100 rows are drawn (a window holds at
// most 30); the rest of a long range is on the next page.

import { Localized } from '@fluent/react';
import { ChevronRight } from 'lucide-react';
import { useEffect, useId, useRef, type ReactNode } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import type { PartyLedgerDay } from '../../../bindings/PartyLedgerDay';
import type { PartyLedgerStatement } from '../../../bindings/PartyLedgerStatement';
import { isoToDisplayDate } from '../../../shared/format';
import { isFromStep, useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { treeKeyHandler } from '../../../shared/keyboard';
import { Balance, Num } from '../../../shared/ui';

/** Rows moved by Page Up and Page Down in a plain tree; here the keys move the window instead. */
const TREE_PAGE_ROWS = 10;

interface StatementDaysProps {
  /** Buyer or seller: it says what the columns "charged" and "paid" mean. */
  kind: LedgerSide;
  rows: readonly PartyLedgerDay[];
  /** The backend's figures of the window or range: its start, its sums and its end. */
  totals: PartyLedgerStatement;
  /** The dates that are open (`YYYY-MM-DD`). */
  open: ReadonlySet<string>;
  onToggle: (date: string, open: boolean) => void;
  /** The date the keys are on, or null: the first one. */
  active: string | null;
  onActive: (date: string) => void;
  /** What an open day shows under its row. */
  renderDay: (date: string) => ReactNode;
  /** Shown in place of the rows when there are none; the heading and the footer stay. */
  empty: ReactNode;
  /** A new window or page is on its way: the rows on screen are the last ones. */
  busy: boolean;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
}

export function StatementDays({
  kind,
  rows,
  totals,
  open,
  onToggle,
  active,
  onActive,
  renderDay,
  empty,
  busy,
  labelId,
}: StatementDaysProps) {
  const listId = useId();
  const boxRef = useRef<HTMLDivElement | null>(null);
  const step = useLayoutStep();
  // At the compact step (1024 px, with both menus open) five columns do not fit and the list must never
  // scroll sideways (NFR-USE-15, F22): the balance before a day is the balance after the day before it,
  // so this is the column that waits for the normal step.
  const withPrevious = isFromStep(step, 'normal');
  const template = `repeat(${String(withPrevious ? 5 : 4)}, minmax(0, 1fr))`;
  const activeIndex = Math.max(
    rows.findIndex((row) => row.date === active),
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
    if (current !== undefined) onToggle(current.date, !open.has(current.date));
  };
  const onTreeKey = treeKeyHandler({
    count: rows.length,
    active: activeIndex,
    pageSize: TREE_PAGE_ROWS,
    onMove: (index) => {
      const row = rows[index];
      if (row !== undefined) onActive(row.date);
    },
    onHorizontal: (action) => {
      if (current === undefined) return false;
      const isOpen = open.has(current.date);
      // A flat list: the arrow towards the children opens the day, the one towards the parent shuts it.
      if (action === 'opens' && !isOpen) onToggle(current.date, true);
      else if (action === 'closes' && isOpen) onToggle(current.date, false);
      else return false;
      return true;
    },
    onEnter: toggleCurrent,
  });

  const heading = (id: string) => (
    <Localized id={id}>
      <span role="columnheader" className="min-w-0 text-center type-label text-ink-muted" />
    </Localized>
  );
  return (
    <div
      ref={boxRef}
      role="treegrid"
      tabIndex={0}
      aria-labelledby={`${listId}-label`}
      aria-busy={busy}
      aria-activedescendant={current === undefined ? undefined : `${listId}-${current.date}`}
      onKeyDown={(event) => {
        // A key inside an open day (its print button, its goods) is that day's own.
        if (event.target !== event.currentTarget) return;
        // Page Up and Page Down move the window of days: the screen's shortcut, not a page of rows.
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
          {heading('party-ledgers-col-date')}
          {withPrevious ? heading('party-ledgers-col-previous') : null}
          {heading(`party-ledgers-col-${kind}-charged`)}
          {heading(`party-ledgers-col-${kind}-paid`)}
          {heading('party-ledgers-col-balance')}
        </div>
      </div>
      {rows.length === 0 ? (
        <div className="px-3 py-6">{empty}</div>
      ) : (
        rows.map((row) => {
          const isOpen = open.has(row.date);
          const isActive = row.date === current?.date;
          return (
            <div key={row.date}>
              <div
                id={`${listId}-${row.date}`}
                role="row"
                aria-expanded={isOpen}
                aria-selected={isActive}
                onClick={() => {
                  onActive(row.date);
                  onToggle(row.date, !isOpen);
                  boxRef.current?.focus();
                }}
                className={`flex min-h-target cursor-pointer items-center gap-3 border-b border-border px-3 py-2 type-body text-ink ${isActive ? 'bg-row-current' : ''}`}
              >
                <ChevronRight
                  aria-hidden="true"
                  className={`size-icon-md shrink-0 transition-calm ${isOpen ? 'rotate-90' : 'rtl:rotate-180'}`}
                />
                <div
                  className="grid min-w-0 flex-1 items-center gap-x-2"
                  style={{ gridTemplateColumns: template }}
                >
                  <span role="gridcell" className="min-w-0 text-center">
                    <Num>{isoToDisplayDate(row.date)}</Num>
                  </span>
                  {withPrevious ? (
                    <span role="gridcell" className="min-w-0 text-center">
                      <Balance
                        balance={{ amount_paisa: row.previous_paisa, side: row.previous_side }}
                      />
                    </span>
                  ) : null}
                  <span role="gridcell" className="min-w-0 text-center">
                    <Num paisa={row.charged_paisa} />
                  </span>
                  <span role="gridcell" className="min-w-0 text-center">
                    <Num paisa={row.paid_paisa} />
                  </span>
                  <span role="gridcell" className="min-w-0 text-center">
                    <Balance
                      balance={{ amount_paisa: row.balance_paisa, side: row.balance_side }}
                    />
                  </span>
                </div>
              </div>
              {isOpen ? (
                <div role="row">
                  <div
                    role="gridcell"
                    aria-colspan={withPrevious ? 5 : 4}
                    data-day-detail=""
                    className="border-b border-border bg-surface-sunken p-3"
                  >
                    {renderDay(row.date)}
                  </div>
                </div>
              ) : null}
            </div>
          );
        })
      )}
      <div
        role="row"
        className="flex items-center gap-3 bg-surface-sunken px-3 py-2 type-body font-semibold text-ink"
      >
        <span aria-hidden="true" className="size-icon-md shrink-0" />
        <div
          className="grid min-w-0 flex-1 items-center gap-x-2"
          style={{ gridTemplateColumns: template }}
        >
          <Localized id="glossary-meezan">
            <span role="gridcell" className="min-w-0 text-center" />
          </Localized>
          {withPrevious ? (
            <span role="gridcell" className="min-w-0 text-center">
              <Balance
                balance={{ amount_paisa: totals.opening_paisa, side: totals.opening_side }}
              />
            </span>
          ) : null}
          <span role="gridcell" className="min-w-0 text-center">
            <Num paisa={totals.charged_total_paisa} />
          </span>
          <span role="gridcell" className="min-w-0 text-center">
            <Num paisa={totals.paid_total_paisa} />
          </span>
          <span role="gridcell" className="min-w-0 text-center">
            <Balance balance={{ amount_paisa: totals.closing_paisa, side: totals.closing_side }} />
          </span>
        </div>
      </div>
    </div>
  );
}
