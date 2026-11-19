// The columns of the seller dues page (PRD FR-9.6): the seller's code, his name with his tag, his city, what the shop
// owes him (his open payable, FR-7.59), what he owes the shop beyond his advances (FR-15.9), his balance with
// its side word (the balance of his party card and R13) and the day it last changed. Every figure is the
// backend's; the list adds up nothing. Less used columns come with the wider layout steps so the list never
// scrolls sideways (NFR-USE-15, F22). Money and dates are centred (NFR-USE-19).

import type { SellerDuesRow } from '../../bindings/SellerDuesRow';
import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { displayName } from '../../shared/i18n';
import { nameWithTag } from '../reports';

/** The side word of a balance: بنام or جمع. A zero balance has none. */
export function sellerSideWord(row: SellerDuesRow, text: (id: string) => string): string {
  if (row.balance.side === 'zero') return '';
  return text(row.balance.side === 'dr' ? 'report-side-dr' : 'report-side-cr');
}

export const SELLER_DUES_COLUMNS: readonly ListColumn<SellerDuesRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    from: 'normal',
    value: (row) => row.party.code,
  },
  {
    field: 'seller',
    type: 'name',
    headerId: 'seller-dues-col-seller',
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
    field: 'payable',
    type: 'money',
    headerId: 'seller-dues-col-payable',
    sortKey: 'payable',
    from: 'normal',
    value: (row, { money }) => money(row.payable_paisa),
  },
  {
    field: 'owes',
    type: 'money',
    headerId: 'seller-dues-col-owes',
    sortKey: 'owes',
    from: 'normal',
    value: (row, { money }) => money(row.owes_paisa),
  },
  {
    field: 'balance',
    type: 'balance',
    headerId: 'dashboard-dues-col-balance',
    sortKey: 'balance',
    value: (row, { money }) => money(row.balance.amount_paisa),
    unit: (row, { text }) => sellerSideWord(row, text),
  },
  {
    field: 'last_change',
    type: 'date',
    headerId: 'dues-col-last-change',
    sortKey: 'last_change_date',
    from: 'normal',
    value: (row) => (row.last_change_date === null ? '' : isoToDisplayDate(row.last_change_date)),
  },
];
