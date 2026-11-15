// The columns of the buyer dues page (PRD FR-9.4): the buyer's code, his name with his tag, his city, what he
// owes now with its side word, and the day his balance last changed. No aging buckets (that is R20). Every
// figure is the backend's; the list adds up nothing. Less used columns come with the wider layout steps so
// the list never scrolls sideways (NFR-USE-15, F22). Money and dates are centred (NFR-USE-19).

import type { BuyerDuesRow } from '../../bindings/BuyerDuesRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { nameWithTag } from '../reports';

/** The side word of a balance: بنام or جمع. A zero balance has none. */
function sideWord(row: BuyerDuesRow, text: (id: string) => string): string {
  if (row.balance.side === 'zero') return '';
  return text(row.balance.side === 'dr' ? 'report-side-dr' : 'report-side-cr');
}

export const BUYER_DUES_COLUMNS: readonly ListColumn<BuyerDuesRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.party.code,
  },
  {
    field: 'buyer',
    type: 'name',
    headerId: 'dashboard-dues-col-party',
    // The backend sorts by name in the Urdu order, or by the English name (`SortInput.language`).
    sortKey: 'party',
    grow: true,
    value: (row, { language }) => nameWithTag(row.party, row.party.tag, language),
  },
  {
    field: 'city',
    type: 'text',
    headerId: 'dashboard-dues-col-city',
    sortKey: 'city',
    from: 'wide',
    value: (row, { language }) =>
      row.city === null
        ? ''
        : displayName({ name_ur: row.city.ur, name_en: row.city.en }, language),
  },
  {
    field: 'balance',
    type: 'money',
    headerId: 'dashboard-dues-col-balance',
    sortKey: 'balance',
    value: (row, { money }) => money(row.balance.amount_paisa),
    unit: (row, { text }) => sideWord(row, text),
  },
  {
    field: 'last_change',
    type: 'date',
    headerId: 'dues-col-last-change',
    sortKey: 'last_change_date',
    value: (row) => (row.last_change_date === null ? '' : isoToDisplayDate(row.last_change_date)),
  },
];
