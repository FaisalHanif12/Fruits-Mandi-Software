// The columns of the اگراہی sheet (R5, PRD R5, plan 76 point 2): the buyer's code, his name with his tag
// (a person in the dead amount carries that word), what he bought that day in short ("(Banana 20 . 600)"),
// what he owed at the start of the day, today's bill, what he paid on the spot, and what he owes now, each
// balance with its side word. There is no وصولی column: a وصولی of the day shows in the next day's previous
// due. Every figure is the backend's; the list adds up nothing. Less used columns come with the wider layout
// steps so the list never scrolls sideways (NFR-USE-15, F22). Money is centred (NFR-USE-19).

import type { Balance } from '../../bindings/Balance';
import type { CollectionSheetRow } from '../../bindings/CollectionSheetRow';
import type { ListColumn } from '../../shared/grid';
import { collectionLineText } from './cells';
import { nameWithTag } from './cells';

/** The side word of a balance: بنام or جمع. A zero balance has none. */
function sideWord(balance: Balance, text: (id: string) => string): string {
  if (balance.side === 'zero') return '';
  return text(balance.side === 'dr' ? 'report-side-dr' : 'report-side-cr');
}

export const COLLECTION_COLUMNS: readonly ListColumn<CollectionSheetRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.party.code,
  },
  {
    field: 'party',
    type: 'name',
    headerId: 'collection-col-party',
    // The backend sorts by name in the Urdu order, or by the English name (`SortInput.language`).
    sortKey: 'party',
    grow: true,
    value: (row, { language, text }) =>
      row.is_dead
        ? `${nameWithTag(row.party, row.party.tag, language)} (${text('dead-tag')})`
        : nameWithTag(row.party, row.party.tag, language),
  },
  {
    field: 'detail',
    type: 'text',
    headerId: 'collection-col-detail',
    from: 'normal',
    grow: true,
    value: (row, context) => row.lines.map((line) => collectionLineText(line, context)).join(' '),
  },
  {
    field: 'previous',
    type: 'money',
    headerId: 'collection-col-previous',
    sortKey: 'previous_due',
    from: 'normal',
    value: (row, { money }) => money(row.previous_due.amount_paisa),
    unit: (row, { text }) => sideWord(row.previous_due, text),
  },
  {
    field: 'bill',
    type: 'money',
    headerId: 'collection-col-bill',
    sortKey: 'bill',
    value: (row, { money }) => money(row.bill_paisa),
  },
  {
    field: 'paid',
    type: 'money',
    headerId: 'collection-col-paid',
    sortKey: 'paid',
    from: 'normal',
    value: (row, { money }) => money(row.paid_paisa),
  },
  {
    field: 'total_due',
    type: 'money',
    headerId: 'collection-col-total-due',
    sortKey: 'total_due',
    value: (row, { money }) => money(row.total_due.amount_paisa),
    unit: (row, { text }) => sideWord(row.total_due, text),
  },
];
