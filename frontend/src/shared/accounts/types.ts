// INTERIM (plan 07 T3): the shape of the account and chart commands as the plan's contract
// describes them (`accounts_*`, `coa_*`). When backend sends the generated bindings,
// `account-api.ts` is the only file that changes. Code Standards F2: types of backend data come
// from bindings. Money from the backend is `Paisa` (shown, never worked out); money typed by the
// user is text.

import type { Paisa } from '../format';
import type { BalanceValue } from '../ui';

/** The four kinds of account of PRD 10.2. The type sets the control and the suggested code. */
export type AccountType = 'seller' | 'buyer' | 'money_lent' | 'other';

/** Which side a balance is on. The backend decides; the screen only shows the word. */
export type Side = 'dr' | 'cr' | 'none';

/** A balance: how much, never negative, and on which side (بنام or جمع). */
export type Balance = BalanceValue;

/** A record another one points at, for a picker to show. */
export interface AccountRef {
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
}

export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun';

/** One account as the list and the form read it. */
export interface AccountRow {
  id: number;
  version: number;
  code: string;
  account_type: AccountType;
  control_id: number;
  /** The control, for display: code and names. */
  control: AccountRef;
  name_ur: string;
  name_en: string;
  father_or_firm_ur: string;
  father_or_firm_en: string;
  city: AccountRef | null;
  adda: AccountRef | null;
  phone1: string;
  phone2: string;
  address: string;
  linked_account: AccountRef | null;
  /** `YYYY-MM-DD`. */
  opening_date: string;
  opening_dr_paisa: Paisa;
  opening_cr_paisa: Paisa;
  opening_is_advance: boolean;
  credit_allowed: boolean;
  credit_limit_paisa: Paisa;
  default_marka: AccountRef | null;
  collector: AccountRef | null;
  collection_weekday: Weekday | null;
  is_system: boolean;
  is_active: boolean;
  note: string;
  balance: Balance;
}

export interface AccountFilter {
  account_type: AccountType | null;
  text: string;
  include_inactive: boolean;
}

export interface AccountList {
  rows: AccountRow[];
  total_dr_paisa: Paisa;
  total_cr_paisa: Paisa;
}

/** A close match shown while a name is typed (PRD FR-7.63, FR-10.7). */
export interface AccountMatch {
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  father_or_firm_ur: string;
  father_or_firm_en: string;
  city: AccountRef | null;
  phone1: string;
  balance: Balance;
}

/** A node of the chart: a head, a control or an account, with its live balance (FR-10.1). */
export interface CoaAccount {
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  is_active: boolean;
  balance: Balance;
}

export interface CoaControl {
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  balance: Balance;
  accounts: CoaAccount[];
}

export interface CoaHead {
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  balance: Balance;
  controls: CoaControl[];
}

/** What the account form holds, all as text (CLAUDE.md rule 1). */
export interface AccountDraft {
  id: number | null;
  version: number | null;
  account_type: AccountType;
  control_id: number | null;
  code: string;
  names: { urdu: string; english: string };
  father: { urdu: string; english: string };
  city: AccountRef | null;
  adda: AccountRef | null;
  phone1: string;
  phone2: string;
  address: string;
  linked_account: AccountRef | null;
  /** `DD-MM-YYYY` as typed. */
  opening_date: string;
  opening_dr: string;
  opening_cr: string;
  opening_is_advance: boolean;
  credit_allowed: boolean;
  credit_limit: string;
  default_marka: AccountRef | null;
  collector: AccountRef | null;
  collection_weekday: Weekday | '';
  is_active: boolean;
  note: string;
}

// ---- Plan 08 (INTERIM until the generated bindings arrive): short codes, the party card, search.

/** Which accounts an account field takes. `any` is a field for every account (جنرل, ledgers). */
export type AccountScope = 'seller' | 'buyer' | 'any';

/** One account in a picker or a search: the row of a list, with his balance (FR-7.40). */
export interface AccountHit {
  id: number;
  code: string;
  account_type: AccountType;
  name_ur: string;
  name_en: string;
  /** Father or firm and city, to tell two people with the same name apart. */
  extra_ur: string;
  extra_en: string;
  is_active: boolean;
  balance: Balance;
}

/**
 * What a code typed in an account field means (PRD FR-7.39, FR-7.42), asked on Enter or on leaving
 * the field, never while typing: `1` may be the start of `12`.
 */
export interface ResolvedCode {
  status: 'found' | 'several' | 'unknown' | 'reserved';
  account: AccountHit | null;
  candidates: AccountHit[];
  /** For an unknown code: the full code a new account would get. */
  suggested_code: string | null;
}

export type CardWarning = 'inactive' | 'over_credit_limit' | 'owes_firm';

/** The party card (PRD FR-7.38): everything about a party, read from the books. */
export interface PartyCardData {
  account_id: number;
  code: string;
  account_type: AccountType;
  name_ur: string;
  name_en: string;
  father_or_firm_ur: string;
  father_or_firm_en: string;
  city_ur: string;
  city_en: string;
  phone: string;
  balance: Balance;
  /** `YYYY-MM-DD`, or null when nothing was ever posted. */
  last_entry_date: string | null;
  /** A buyer: his credit limit and how much of it is used. */
  credit_limit_paisa: Paisa | null;
  credit_used_paisa: Paisa | null;
  /** A seller: the advance still open. */
  advance_open_paisa: Paisa | null;
  warnings: CardWarning[];
}

/** One result of the global search (Ctrl+K). Accounts now; later modules add their own kinds. */
export interface GlobalHit {
  kind: 'account';
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  extra_ur: string;
  extra_en: string;
}
