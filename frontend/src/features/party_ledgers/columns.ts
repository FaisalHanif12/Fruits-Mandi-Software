// The columns of the party ledgers screen (PRD FR-12.43): level 1, the list of parties. (Level 2, the
// statement of one party, is `components/StatementEntries.tsx`: a row for each entry, plan 78.) Every figure and every side is the backend's; a column writes it with
// `money` and the word of its side, and adds up nothing (CLAUDE.md rule 1). Money columns, their
// headings and their totals are centred under the heading (F17, NFR-USE-19). The widths are the
// shared ones of the column types (F18), so the list never scrolls sideways (NFR-USE-15).

import type { PartyLedgerRow } from '../../bindings/PartyLedgerRow';
import type { ListColumn, ListContext } from '../../shared/grid';
import { displayName } from '../../shared/i18n';

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
