// The columns of the party ledgers screen (PRD FR-12.43): level 1, the list of parties, and level 2,
// the statement of one party. Every figure and every side is the backend's; a column writes it with
// `money` and the word of its side, and adds up nothing (CLAUDE.md rule 1). Money columns, their
// headings and their totals are centred under the heading (F17, NFR-USE-19). The widths are the
// shared ones of the column types (F18), so the list never scrolls sideways (NFR-USE-15).

import type { BalanceSide } from '../../bindings/BalanceSide';
import type { LedgerSide } from '../../bindings/LedgerSide';
import type { PartyLedgerDay } from '../../bindings/PartyLedgerDay';
import type { PartyLedgerRow } from '../../bindings/PartyLedgerRow';
import type { PartyLedgerStatement } from '../../bindings/PartyLedgerStatement';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn, ListContext } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { sideMessageId } from '../../shared/ui';

/** The word of a side, or nothing for a balance of zero. */
function sideWord(side: BalanceSide, context: ListContext): string {
  const id = sideMessageId(side);
  return id === null ? '' : context.text(id);
}

/**
 * A name with what it is marked with, in words and not by colour alone (NFR-USE-12): a person in the
 * dead amount is marked ڈوبی (PRD FR-10.10), another inactive account says so.
 */
export function markedName(
  row: Pick<PartyLedgerRow, 'name_ur' | 'name_en' | 'tag' | 'is_dead' | 'is_active'>,
  context: ListContext,
): string {
  const name = displayName(row, context.language);
  if (row.is_dead) return `${name} (${context.text('dead-tag')})`;
  return row.is_active ? name : `${name} (${context.text('masters-status-inactive')})`;
}

/**
 * Level 1: code, name, and the balance under the side it is on. A debit balance (بنام) is in the first
 * money column, a credit balance (جمع) in the second, so the total row under them is the plain
 * total of each (F19). A balance of zero is shown as 0 under بنام.
 */
export const LIST_COLUMNS: readonly ListColumn<PartyLedgerRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'party-ledgers-col-code',
    value: (row) => row.code,
  },
  {
    field: 'name',
    type: 'name',
    headerId: 'party-ledgers-col-name',
    grow: true,
    value: (row, context) => markedName(row, context),
  },
  {
    field: 'debit',
    type: 'money',
    headerId: 'party-ledgers-col-debit',
    value: (row, { money }) => (row.side === 'cr' ? '' : money(row.balance_paisa)),
  },
  {
    field: 'credit',
    type: 'money',
    headerId: 'party-ledgers-col-credit',
    value: (row, { money }) => (row.side === 'cr' ? money(row.balance_paisa) : ''),
  },
];

/** The footer of level 1: the backend's sums over every party of the filter, never of the page. */
export function listTotals(
  page: { total_dr_paisa: PartyLedgerRowPaisa; total_cr_paisa: PartyLedgerRowPaisa } | undefined,
  context: ListContext,
): { labelField: string; values: Record<string, string> } {
  return {
    labelField: 'name',
    values:
      page === undefined
        ? {}
        : {
            name: context.text('glossary-meezan'),
            debit: context.money(page.total_dr_paisa),
            credit: context.money(page.total_cr_paisa),
          },
  };
}

type PartyLedgerRowPaisa = PartyLedgerRow['balance_paisa'];

/**
 * Level 2: the date, the balance before it, what the day charged and what was paid, and the balance
 * after it, each balance with its side word. Every column shares the free width, so the rows and the
 * total row run the whole width of the grid. The headings say what a buyer's and what a seller's
 * "charged" and "paid" are (the backend's `kind`).
 */
export function statementColumns(kind: LedgerSide): readonly ListColumn<PartyLedgerDay>[] {
  return [
    {
      field: 'date',
      type: 'date',
      headerId: 'party-ledgers-col-date',
      grow: true,
      value: (row) => isoToDisplayDate(row.date),
    },
    {
      field: 'previous',
      type: 'balance',
      headerId: 'party-ledgers-col-previous',
      // At the compact step (1024 px, with both menus open) five columns do not fit and the list
      // must never scroll sideways (NFR-USE-15, F22). The balance before a day is the balance after
      // the day before it, so this is the column that waits for the normal step.
      from: 'normal',
      grow: true,
      value: (row, { money }) => money(row.previous_paisa),
      unit: (row, context) => sideWord(row.previous_side, context),
    },
    {
      field: 'charged',
      type: 'money',
      headerId: `party-ledgers-col-${kind}-charged`,
      grow: true,
      value: (row, { money }) => money(row.charged_paisa),
    },
    {
      field: 'paid',
      type: 'money',
      headerId: `party-ledgers-col-${kind}-paid`,
      grow: true,
      value: (row, { money }) => money(row.paid_paisa),
    },
    {
      field: 'balance',
      type: 'balance',
      headerId: 'party-ledgers-col-balance',
      grow: true,
      value: (row, { money }) => money(row.balance_paisa),
      unit: (row, context) => sideWord(row.balance_side, context),
    },
  ];
}

/**
 * The footer of level 2: the balance before the first date asked for, what every date charged and
 * what was paid, and the balance after the last one. All four are the backend's, over the whole
 * filter.
 */
export function statementTotals(
  page: PartyLedgerStatement | undefined,
  context: ListContext,
): { labelField: string; values: Record<string, string> } {
  return {
    labelField: 'date',
    values:
      page === undefined
        ? {}
        : {
            date: context.text('glossary-meezan'),
            previous: context.money(page.opening_paisa),
            'previous:unit': sideWord(page.opening_side, context),
            charged: context.money(page.charged_total_paisa),
            paid: context.money(page.paid_total_paisa),
            balance: context.money(page.closing_paisa),
            'balance:unit': sideWord(page.closing_side, context),
          },
  };
}
