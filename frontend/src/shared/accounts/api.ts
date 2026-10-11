// Typed wrappers around the account and chart commands (plan 07 contract; `accounts_*`, `coa_*`).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.

import { callCommand } from '../api';
import type {
  AccountHit,
  AccountScope,
  GlobalHit,
  PartyCardData,
  ResolvedCode,
  AccountDraft,
  AccountFilter,
  AccountList,
  AccountMatch,
  AccountRow,
  AccountType,
  CoaControl,
  CoaHead,
} from './types';

/** The chart in three levels with live balances (PRD FR-10.1). */
export function coaTree(): Promise<CoaHead[]> {
  return callCommand<CoaHead[]>('coa_tree');
}

export function coaHeadsList(): Promise<CoaHead[]> {
  return callCommand<CoaHead[]>('coa_heads_list');
}

export function coaControlsList(): Promise<CoaControl[]> {
  return callCommand<CoaControl[]>('coa_controls_list');
}

/** One account by id, for the form that edits it. */
export function accountsGet(id: number): Promise<AccountRow> {
  return callCommand<AccountRow>('accounts_get', { input: { id } });
}

export function accountsList(filter: AccountFilter): Promise<AccountList> {
  return callCommand<AccountList>('accounts_list', { input: { filter } });
}

/** The next free code for a type (and a control, for "other"), to fill the code field. */
export function accountsNextCode(
  accountType: AccountType,
  controlId: number | null,
): Promise<string> {
  return callCommand<string>('accounts_next_code', {
    input: { account_type: accountType, control_id: controlId },
  });
}

/** Accounts that look like the one being typed, for the duplicate warning (PRD FR-7.63). */
export function accountsSimilar(input: {
  name_ur: string;
  name_en: string;
  phone: string;
}): Promise<AccountMatch[]> {
  return callCommand<AccountMatch[]>('accounts_similar', { input });
}

/**
 * Creates (no id) or edits (with id and the version that was read) one account. Money and the
 * date go as the text typed; the backend parses them and checks every field again.
 */
export function accountsSave(
  draft: AccountDraft,
  confirmedRulesWords: readonly string[],
): Promise<AccountRow> {
  return callCommand<AccountRow>('accounts_save', {
    input: {
      id: draft.id,
      version: draft.version,
      account_type: draft.account_type,
      control_id: draft.control_id,
      code: draft.code,
      name_ur: draft.names.urdu,
      name_en: draft.names.english,
      father_or_firm_ur: draft.father.urdu,
      father_or_firm_en: draft.father.english,
      confirmed_rules_words: confirmedRulesWords,
      city_id: draft.city?.id ?? null,
      adda_id: draft.adda?.id ?? null,
      phone1: draft.phone1,
      phone2: draft.phone2,
      address: draft.address,
      linked_account_id: draft.linked_account?.id ?? null,
      opening_date: draft.opening_date,
      opening_dr: draft.opening_dr,
      opening_cr: draft.opening_cr,
      opening_is_advance: draft.opening_is_advance,
      credit_allowed: draft.credit_allowed,
      credit_limit: draft.credit_limit,
      default_marka_id: draft.default_marka?.id ?? null,
      collector_staff_id: draft.collector?.id ?? null,
      collection_weekday: draft.collection_weekday === '' ? null : draft.collection_weekday,
      is_active: draft.is_active,
      note: draft.note,
    },
  });
}

/** Makes an account inactive or active again. An account is never deleted (PRD FR-10.6). */
export function accountsSetActive(id: number, isActive: boolean): Promise<AccountRow> {
  return callCommand<AccountRow>('accounts_set_active', { input: { id, is_active: isActive } });
}

// ---- Plan 08: short codes, the party card, search (INTERIM until the bindings arrive).

/** Accounts for a picker: exact code first, then the exact serial, then names (PRD FR-7.40). */
export function searchAccounts(input: {
  scope: AccountScope;
  text: string;
  limit: number;
}): Promise<AccountHit[]> {
  return callCommand<AccountHit[]>('search_accounts', { input });
}

/** The recent accounts of a picker, then the rest (PRD FR-7.41). */
export function searchAccountsRecent(scope: AccountScope): Promise<AccountHit[]> {
  return callCommand<AccountHit[]>('search_accounts_recent', { input: { scope } });
}

/** Remembers that an account was chosen, so it comes first next time. */
export function searchAccountsTouch(scope: AccountScope, id: number): Promise<null> {
  return callCommand<null>('search_accounts_touch', { input: { scope, id } });
}

/** What a code typed in an account field means. Called on Enter or on leaving the field only. */
export function accountsResolveCode(scope: AccountScope, text: string): Promise<ResolvedCode> {
  return callCommand<ResolvedCode>('accounts_resolve_code', { input: { scope, text } });
}

/** The party card of an account, read from the books (PRD FR-7.38). */
export function assistPartyCard(accountId: number): Promise<PartyCardData> {
  return callCommand<PartyCardData>('assist_party_card', { input: { account_id: accountId } });
}

/** The global search (Ctrl+K): accounts for now. */
export function searchGlobal(text: string): Promise<GlobalHit[]> {
  return callCommand<GlobalHit[]>('search_global', { input: { text } });
}
