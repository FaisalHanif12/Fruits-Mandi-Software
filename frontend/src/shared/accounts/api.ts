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
  ChartHead,
  ControlItem,
  HeadItem,
  OpeningsInput,
  OpeningsPreview,
  OpeningsSaved,
} from './types';
import type { AccountBalance } from '../../bindings/AccountBalance';
import type { AccountHit as AccountHitBinding } from '../../bindings/AccountHit';
import type { CoaTree } from '../../bindings/CoaTree';
import type { GlobalHit as GlobalHitBinding } from '../../bindings/GlobalHit';
import type { ResolvedCode as ResolvedCodeBinding } from '../../bindings/ResolvedCode';

/** The chart in three levels with live balances (PRD FR-10.1). */
export async function coaTree(): Promise<ChartHead[]> {
  const tree = await callCommand<CoaTree>('coa_tree');
  return tree.heads;
}

export function coaHeadsList(): Promise<HeadItem[]> {
  return callCommand<HeadItem[]>('coa_heads_list');
}

/** The controls; each says whether an account of type "other" may be made under it. */
export function coaControlsList(): Promise<ControlItem[]> {
  return callCommand<ControlItem[]>('coa_controls_list');
}

/** One account by id, for the form that edits it. */
export function accountsGet(id: number): Promise<AccountRow> {
  return callCommand<AccountRow>('accounts_get', { input: { id } });
}

/**
 * 0 asks for every matching account: the backend returns up to 50,000 (2.5 times the 20,000
 * accounts of NFR-SCL-1), and the grid draws only the rows in view. The screen sets no cap.
 */
const LIST_PAGE_LIMIT = 0;

export function accountsList(filter: AccountFilter, offset = 0): Promise<AccountList> {
  return callCommand<AccountList>('accounts_list', {
    input: { filter, offset, limit: LIST_PAGE_LIMIT },
  });
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
  exclude_id: number | null;
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
      collection_weekday: draft.collection_weekday,
      is_active: draft.is_active,
      note: draft.note,
    },
  });
}

/** Makes an account inactive or active again. An account is never deleted (PRD FR-10.6). */
export function accountsSetActive(id: number, isActive: boolean): Promise<AccountRow> {
  return callCommand<AccountRow>('accounts_set_active', { input: { id, is_active: isActive } });
}

/** Checks the openings grid and says what ابتدائی فرق would be. Nothing is saved. */
export function accountsOpeningsPreview(input: OpeningsInput): Promise<OpeningsPreview> {
  return callCommand<OpeningsPreview>('accounts_openings_preview', { input });
}

/** Saves the openings grid in one transaction. Errors point to `lines[i].<field>`. */
export function accountsOpeningsSave(input: OpeningsInput): Promise<OpeningsSaved> {
  return callCommand<OpeningsSaved>('accounts_openings_save', { input });
}

// ---- Plan 08: short codes, the party card, search (PRD FR-7.38 to FR-7.42).

/** The balances of some accounts, for the rows of a picker (PRD FR-7.40). */
export function accountsBalances(ids: readonly number[]): Promise<AccountBalance[]> {
  return callCommand<AccountBalance[]>('accounts_balances', { input: { ids: [...ids] } });
}

/**
 * Adds the balance to each hit with one call for all of them. If the balances cannot be read the
 * hits still come, without a balance: a row never shows a balance that was not read.
 */
async function withBalances(hits: readonly AccountHitBinding[]): Promise<AccountHit[]> {
  if (hits.length === 0) return [];
  const balances = new Map<number, AccountBalance['balance']>();
  try {
    for (const row of await accountsBalances(hits.map((hit) => hit.id))) {
      balances.set(row.id, row.balance);
    }
  } catch {
    // The picker works without balances.
  }
  return hits.map((hit) => ({ ...hit, balance: balances.get(hit.id) ?? null }));
}

/**
 * Accounts for a picker: exact code first, then the exact serial, then names, then close
 * spellings (PRD FR-7.40). `controlId` is for a field that takes the accounts of one control.
 */
export async function searchAccounts(input: {
  scope: AccountScope;
  controlId: number | null;
  text: string;
  limit: number;
}): Promise<AccountHit[]> {
  const hits = await callCommand<AccountHitBinding[]>('search_accounts', {
    input: {
      scope: input.scope,
      control_id: input.controlId,
      text: input.text,
      limit: input.limit,
    },
  });
  return withBalances(hits);
}

/**
 * The recent accounts of a field, then the rest (PRD FR-7.41). Each field (a scope, and a control
 * for a field of one control) has its own list, and the hits come as accounts, with their type.
 */
export async function searchAccountsRecent(
  scope: AccountScope,
  controlId: number | null,
): Promise<AccountHit[]> {
  const hits = await callCommand<AccountHitBinding[]>('search_accounts_recent', {
    input: { scope, control_id: controlId },
  });
  return withBalances(hits);
}

/** Remembers that an account was chosen in a field, so it comes first next time. */
export function searchAccountsTouch(
  scope: AccountScope,
  controlId: number | null,
  id: number,
): Promise<null> {
  return callCommand<null>('search_accounts_touch', {
    input: { scope, control_id: controlId, id },
  });
}

/** What a code typed in an account field means. Called on Enter or on leaving the field only. */
export async function accountsResolveCode(
  scope: AccountScope,
  controlId: number | null,
  text: string,
): Promise<ResolvedCode> {
  const result = await callCommand<ResolvedCodeBinding>('accounts_resolve_code', {
    input: { scope, control_id: controlId, text },
  });
  const plain = (hit: AccountHitBinding): AccountHit => ({ ...hit, balance: null });
  return {
    status: result.status,
    account: result.account === null ? null : plain(result.account),
    candidates: result.candidates.map(plain),
    suggested_code: result.suggested_code,
  };
}

/** The party card of an account, read from the books (PRD FR-7.38). */
export function assistPartyCard(accountId: number): Promise<PartyCardData> {
  return callCommand<PartyCardData>('assist_party_card', { input: { account_id: accountId } });
}

/** The most results of the global search the box shows. */
const GLOBAL_SEARCH_LIMIT = 20;

/** The global search (Ctrl+K): accounts for now. */
export async function searchGlobal(text: string): Promise<GlobalHit[]> {
  const hits = await callCommand<GlobalHitBinding[]>('search_global', {
    input: { text, limit: GLOBAL_SEARCH_LIMIT },
  });
  return hits.map((item) => ({
    kind: item.kind,
    id: item.hit.id,
    code: item.hit.code,
    name_ur: item.hit.name_ur,
    name_en: item.hit.name_en,
    extra_ur: item.hit.extra_ur,
    extra_en: item.hit.extra_en,
  }));
}
