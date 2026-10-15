// The columns of the account list (PRD FR-10.5): code, Urdu and English name, city, opening date,
// opening debit and credit, credit allowed, current balance. The less used ones come with the
// wider layout steps, so the list never scrolls sideways (NFR-USE-15, Code Standards F22).

import type { Language } from '../../i18n';
import { isoToDisplayDate } from '../../shared/format';
import { displayName, otherScriptName } from '../../shared/i18n';
import type { ListColumn } from '../../shared/grid';
import { sideMessageId } from '../../shared/ui';
import type { AccountRow } from '../../shared/accounts';

const nameOf = (row: AccountRow) => ({ name_ur: row.name_ur, name_en: row.name_en });

export const ACCOUNT_COLUMNS: readonly ListColumn<AccountRow>[] = [
  { field: 'code', type: 'code', headerId: 'accounts-col-code', value: (row) => row.code },
  {
    field: 'name',
    type: 'name',
    headerId: 'accounts-col-name',
    grow: true,
    // An inactive account is dimmed and says so in words, because colour is never the only signal
    // (NFR-USE-12). The word comes with the name, so it takes no column of its own.
    value: (row, { language, text }) =>
      row.is_active
        ? displayName(nameOf(row), language)
        : `${displayName(nameOf(row), language)} (${text('masters-status-inactive')})`,
  },
  {
    field: 'other_name',
    type: 'name',
    headerId: 'accounts-col-other-name',
    from: 'wide',
    grow: true,
    script: (language: Language) => (language === 'ur' ? 'en' : 'ur'),
    value: (row, { language }) => otherScriptName(nameOf(row), language),
  },
  {
    field: 'city',
    type: 'text',
    headerId: 'accounts-col-city',
    from: 'normal',
    value: (row, { language }) => (row.city === null ? '' : displayName(row.city, language)),
  },
  {
    field: 'opening_date',
    type: 'date',
    headerId: 'accounts-col-opening-date',
    from: 'normal',
    value: (row) => (row.opening_date === null ? '' : isoToDisplayDate(row.opening_date)),
  },
  {
    field: 'opening_dr',
    type: 'money',
    headerId: 'accounts-col-opening-dr',
    from: 'wide',
    value: (row, { money }) => money(row.opening_dr_paisa),
  },
  {
    field: 'opening_cr',
    type: 'money',
    headerId: 'accounts-col-opening-cr',
    from: 'wide',
    value: (row, { money }) => money(row.opening_cr_paisa),
  },
  {
    field: 'credit_allowed',
    type: 'status',
    headerId: 'accounts-col-credit',
    from: 'normal',
    value: (row, { text }) => text(row.credit_allowed ? 'common-yes' : 'common-no'),
  },
  {
    field: 'balance',
    type: 'money',
    headerId: 'accounts-col-balance',
    value: (row, { money }) => money(row.balance.amount_paisa),
  },
  {
    field: 'side',
    type: 'status',
    headerId: 'accounts-col-side',
    value: (row, { text }) => {
      const id = sideMessageId(row.balance.side);
      return id === null ? '' : text(id);
    },
  },
];
