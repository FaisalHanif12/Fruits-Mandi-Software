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
// most 30); the rest of a long range is on the next page. The table itself is the shared `DayTable`
// (plan 76): this file only says which columns a party's statement has.

import { Localized } from '@fluent/react';
import { useMemo, type ReactNode } from 'react';
import type { LedgerSide } from '../../../bindings/LedgerSide';
import type { PartyLedgerDay } from '../../../bindings/PartyLedgerDay';
import type { PartyLedgerStatement } from '../../../bindings/PartyLedgerStatement';
import { isoToDisplayDate } from '../../../shared/format';
import { DayTable, type DayColumn } from '../../../shared/grid';
import { Balance, Num } from '../../../shared/ui';

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

export function StatementDays({ kind, totals, ...table }: StatementDaysProps) {
  // At the compact step (1024 px, with both menus open) five columns do not fit and the list must never
  // scroll sideways (NFR-USE-15, F22): the balance before a day is the balance after the day before it,
  // so this is the column that waits for the normal step.
  const columns = useMemo<readonly DayColumn<PartyLedgerDay>[]>(
    () => [
      {
        key: 'date',
        headerId: 'party-ledgers-col-date',
        cell: (row) => <Num>{isoToDisplayDate(row.date)}</Num>,
        footer: (
          <Localized id="glossary-meezan">
            <span />
          </Localized>
        ),
      },
      {
        key: 'previous',
        headerId: 'party-ledgers-col-previous',
        fromStep: 'normal',
        cell: (row) => (
          <Balance balance={{ amount_paisa: row.previous_paisa, side: row.previous_side }} />
        ),
        footer: (
          <Balance balance={{ amount_paisa: totals.opening_paisa, side: totals.opening_side }} />
        ),
      },
      {
        key: 'charged',
        headerId: `party-ledgers-col-${kind}-charged`,
        cell: (row) => <Num paisa={row.charged_paisa} />,
        footer: <Num paisa={totals.charged_total_paisa} />,
      },
      {
        key: 'paid',
        headerId: `party-ledgers-col-${kind}-paid`,
        cell: (row) => <Num paisa={row.paid_paisa} />,
        footer: <Num paisa={totals.paid_total_paisa} />,
      },
      {
        key: 'balance',
        headerId: 'party-ledgers-col-balance',
        cell: (row) => (
          <Balance balance={{ amount_paisa: row.balance_paisa, side: row.balance_side }} />
        ),
        footer: (
          <Balance balance={{ amount_paisa: totals.closing_paisa, side: totals.closing_side }} />
        ),
      },
    ],
    [kind, totals],
  );
  return <DayTable columns={columns} {...table} />;
}
