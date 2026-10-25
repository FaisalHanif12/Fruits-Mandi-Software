// The account form as text in the screen's state (Code Standards F39): a new draft, a draft from a
// saved account, and whether anything has changed. Nothing here works out money: an opening
// amount is shown by the formatter and typed back as text for the backend to read.

import { formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import type { NamePairValue } from '../../shared/ui';
import type { AccountDraft, AccountRow, AccountType } from '../../shared/accounts';

/** The form with the spelling marks of its two name pairs next to the names. */
export interface AccountFormValues extends Omit<AccountDraft, 'names' | 'father'> {
  names: NamePairValue;
  father: NamePairValue;
}

/** A new account: the suggested code, ticked "credit allowed" (PRD 10.2), today as opening date. */
export function newAccountValues(
  accountType: AccountType,
  controlId: number | null,
  code: string,
  today: string,
): AccountFormValues {
  return {
    id: null,
    version: null,
    account_type: accountType,
    control_id: controlId,
    code,
    names: { urdu: '', english: '' },
    tag: '',
    father: { urdu: '', english: '' },
    city: null,
    adda: null,
    phone1: '',
    phone2: '',
    address: '',
    linked_account: null,
    opening_date: today === '' ? '' : isoToDisplayDate(today),
    opening_dr: '',
    opening_cr: '',
    opening_is_advance: false,
    credit_allowed: true,
    credit_limit: '',
    default_marka: null,
    collector: null,
    collection_weekday: null,
    is_active: true,
    note: '',
  };
}

/** An amount from the backend as the text the form shows: nothing for zero. */
function openingText(amount: AccountRow['opening_dr_paisa'] | null, grouping: Grouping): string {
  if (amount === null) return '';
  const text = formatPaisa(amount, { grouping });
  return text === '0' ? '' : text;
}

/** The form of a saved account, exactly as it is saved. */
export function accountValuesOf(row: AccountRow, grouping: Grouping): AccountFormValues {
  return {
    id: row.id,
    version: row.version,
    account_type: row.account_type,
    control_id: row.control.id,
    code: row.code,
    names: { urdu: row.name_ur, english: row.name_en },
    tag: row.tag ?? '',
    father: { urdu: row.father_or_firm_ur, english: row.father_or_firm_en },
    city: row.city,
    adda: row.adda,
    phone1: row.phone1,
    phone2: row.phone2,
    address: row.address,
    linked_account: row.linked_account,
    opening_date: row.opening_date === null ? '' : isoToDisplayDate(row.opening_date),
    opening_dr: openingText(row.opening_dr_paisa, grouping),
    opening_cr: openingText(row.opening_cr_paisa, grouping),
    opening_is_advance: row.opening_is_advance,
    credit_allowed: row.credit_allowed,
    credit_limit: openingText(row.credit_limit_paisa, grouping),
    default_marka: row.default_marka,
    collector: row.collector,
    collection_weekday: row.collection_weekday,
    is_active: row.is_active,
    note: row.note,
  };
}

const TEXT_FIELDS = [
  'code',
  'tag',
  'phone1',
  'phone2',
  'address',
  'opening_date',
  'opening_dr',
  'opening_cr',
  'credit_limit',
  'collection_weekday',
  'note',
] as const;
const FLAG_FIELDS = ['opening_is_advance', 'credit_allowed', 'is_active'] as const;
const REF_FIELDS = ['city', 'adda', 'linked_account', 'default_marka', 'collector'] as const;

/** True when nothing differs: same texts, ticks, chosen records and names. */
export function sameAccountValues(a: AccountFormValues, b: AccountFormValues): boolean {
  return (
    a.account_type === b.account_type &&
    a.control_id === b.control_id &&
    a.names.urdu === b.names.urdu &&
    a.names.english === b.names.english &&
    a.father.urdu === b.father.urdu &&
    a.father.english === b.father.english &&
    TEXT_FIELDS.every((field) => a[field] === b[field]) &&
    FLAG_FIELDS.every((field) => a[field] === b[field]) &&
    REF_FIELDS.every((field) => (a[field]?.id ?? null) === (b[field]?.id ?? null))
  );
}
