// The columns of the account list (PRD FR-10.5): code, Urdu and English name, city, opening date,
// opening debit and credit, credit allowed, current balance. The less used ones come with the
// wider layout steps, so the list never scrolls sideways (NFR-USE-15, Code Standards F22). Below
// the wide step a "More columns" tick adds the opening debit and credit (`accountColumns`).

import type { Language } from '../../i18n';
import { isoToDisplayDate } from '../../shared/format';
import { isFromStep, type LayoutStep } from '../../shared/hooks';
import { displayName, otherScriptName } from '../../shared/i18n';
import type { ListColumn } from '../../shared/grid';
import { sideMessageId } from '../../shared/ui';
import type { AccountRow } from '../../shared/accounts';

export const ACCOUNT_COLUMNS: readonly ListColumn<AccountRow>[] = [
  {
    field: 'code',
    type: 'code',
    headerId: 'accounts-col-code',
    sortKey: 'code',
    value: (row) => row.code,
  },
  {
    field: 'name',
    type: 'name',
    headerId: 'accounts-col-name',
    sortKey: (language) => (language === 'ur' ? 'name_ur' : 'name_en'),
    grow: true,
    // An inactive account is dimmed and says so in words, because colour is never the only signal
    // (NFR-USE-12). The word comes with the name, so it takes no column of its own.
    value: (row, { language, text }) =>
      row.is_active
        ? displayName(row, language)
        : `${displayName(row, language)} (${text('masters-status-inactive')})`,
  },
  {
    field: 'other_name',
    type: 'name',
    headerId: 'accounts-col-other-name',
    from: 'wide',
    grow: true,
    script: (language: Language) => (language === 'ur' ? 'en' : 'ur'),
    value: (row, { language }) => otherScriptName(row, language),
  },
  {
    field: 'city',
    type: 'text',
    headerId: 'accounts-col-city',
    sortKey: 'city',
    from: 'normal',
    value: (row, { language }) => (row.city === null ? '' : displayName(row.city, language)),
  },
  {
    field: 'opening_date',
    type: 'date',
    headerId: 'accounts-col-opening-date',
    sortKey: 'opening_date',
    from: 'normal',
    value: (row) => (row.opening_date === null ? '' : isoToDisplayDate(row.opening_date)),
  },
  {
    field: 'opening_dr',
    type: 'money',
    headerId: 'accounts-col-opening-dr',
    sortKey: 'opening_dr_paisa',
    from: 'wide',
    value: (row, { money }) => money(row.opening_dr_paisa),
  },
  {
    field: 'opening_cr',
    type: 'money',
    headerId: 'accounts-col-opening-cr',
    sortKey: 'opening_cr_paisa',
    from: 'wide',
    value: (row, { money }) => money(row.opening_cr_paisa),
  },
  {
    field: 'credit_allowed',
    type: 'status',
    headerId: 'accounts-col-credit',
    sortKey: 'credit_allowed',
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

/** The columns that "More columns" adds below the wide step: the opening debit and credit. */
const MORE_FIELDS: ReadonlySet<string> = new Set(['opening_dr', 'opening_cr']);

/**
 * The columns that step aside while "More columns" is on, at the normal step only. The two added
 * columns need 280 px, and the normal step has 1,074 px: with them the list would need 1,280 px
 * at its narrowest (the list-widths test adds it up). City and credit allowed are the least needed
 * beside an opening balance, so they give the room. The opening date stays, because it belongs to
 * the opening debit and credit. At the compact step they are not shown anyway, and at the wide
 * step everything fits, so there nothing is taken away.
 */
const MAKES_ROOM_FIELDS: ReadonlySet<string> = new Set(['city', 'credit_allowed']);

/** Does the "More columns" tick change anything at this step? Not at the wide step. */
export function hasMoreColumns(step: LayoutStep): boolean {
  return !isFromStep(step, 'wide');
}

/** Are the opening debit and credit on screen? At the wide step always, below it with the tick. */
export function showsOpeningColumns(step: LayoutStep, more: boolean): boolean {
  return !hasMoreColumns(step) || more;
}

/**
 * The columns of the list for the layout step and the "More columns" tick. Without the tick, or at
 * the wide step, they are the ones each column asks for with `from`. With it, the opening debit
 * and credit come at every step, and at the normal step city and credit allowed step aside
 * (MAKES_ROOM_FIELDS). The list never scrolls sideways in either case (NFR-USE-15).
 */
export function accountColumns(step: LayoutStep, more: boolean): readonly ListColumn<AccountRow>[] {
  if (!more || !hasMoreColumns(step)) return ACCOUNT_COLUMNS;
  return ACCOUNT_COLUMNS.filter(
    (column) => !(step === 'normal' && MAKES_ROOM_FIELDS.has(column.field)),
  ).map((column): ListColumn<AccountRow> =>
    MORE_FIELDS.has(column.field) ? { ...column, from: 'compact' } : column,
  );
}
