// The columns of ادھاری تفصیل (R5, PRD R5 v1.43, plan 80 Q-R5-2): the buyer with his tag (a person in the dead
// amount carries that word), what he bought today in short ("(Banana 20 . 600)"), today's bill, today's credit,
// what he owed at the start of the day and what he owes now, each balance with its side word. There is no cash
// column and no cash total (PRD R5 v1.57, plan 95 item 17): the whole sale print keeps the cash. The words are the print's own (`print-adhari-sheet-col-*`), so the screen and the paper say
// the same. Every figure is the backend's; the list adds up nothing. Money is centred (NFR-USE-19).

import type { AdhariSheetRow } from '../../bindings/AdhariSheetRow';
import type { Balance } from '../../bindings/Balance';
import type { ListColumn } from '../../shared/grid';
import { collectionLineText, nameWithTag } from './cells';

/** The side word of a balance: بنام or جمع. A zero balance has none. */
function sideWord(balance: Balance, text: (id: string) => string): string {
  if (balance.side === 'zero') return '';
  return text(balance.side === 'dr' ? 'report-side-dr' : 'report-side-cr');
}

/** The Fluent id of each column title, in the order the sheet shows them (the guide card reads this list). */
export const ADHARI_COLUMN_TITLE_IDS: readonly string[] = [
  'print-adhari-sheet-col-party',
  'print-adhari-sheet-col-detail',
  'print-adhari-sheet-col-bill',
  'print-adhari-sheet-col-today-credit',
  'print-adhari-sheet-col-previous',
  'print-adhari-sheet-col-total',
];

export const ADHARI_COLUMNS: readonly ListColumn<AdhariSheetRow>[] = [
  {
    field: 'party',
    type: 'name',
    headerId: 'print-adhari-sheet-col-party',
    grow: true,
    value: (row, { language, text }) =>
      row.is_dead
        ? `${nameWithTag(row.party, row.party.tag, language)} (${text('dead-tag')})`
        : nameWithTag(row.party, row.party.tag, language),
  },
  {
    field: 'detail',
    type: 'text',
    headerId: 'print-adhari-sheet-col-detail',
    from: 'normal',
    grow: true,
    value: (row, context) => row.lines.map((line) => collectionLineText(line, context)).join(' '),
  },
  {
    field: 'bill',
    type: 'money',
    headerId: 'print-adhari-sheet-col-bill',
    value: (row, { money }) => money(row.bill_paisa),
  },
  {
    field: 'today_credit',
    type: 'money',
    headerId: 'print-adhari-sheet-col-today-credit',
    value: (row, { money }) => money(row.today_credit_paisa),
  },
  {
    field: 'previous',
    type: 'money',
    headerId: 'print-adhari-sheet-col-previous',
    from: 'normal',
    value: (row, { money }) => money(row.previous_due.amount_paisa),
    unit: (row, { text }) => sideWord(row.previous_due, text),
  },
  {
    field: 'total_due',
    type: 'money',
    headerId: 'print-adhari-sheet-col-total',
    value: (row, { money }) => money(row.total_due.amount_paisa),
    unit: (row, { text }) => sideWord(row.total_due, text),
  },
];
