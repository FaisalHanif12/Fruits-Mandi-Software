// Typed wrappers around the account and chart commands (plan 07 contract; `accounts_*`, `coa_*`).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.

import type { SortInput } from '../../bindings/SortInput';
import { callCommand } from '../api';
import { accountInputOf } from './account-input';
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
import type { CoaControlAccounts } from '../../bindings/CoaControlAccounts';
import type { CoaTree } from '../../bindings/CoaTree';
import type { GlobalHit as GlobalHitBinding } from '../../bindings/GlobalHit';
import type { ResolvedCode as ResolvedCodeBinding } from '../../bindings/ResolvedCode';

/** The chart in three levels with live balances (PRD FR-10.1). */
export async function coaTree(): Promise<ChartHead[]> {
  const tree = await callCommand<CoaTree>('coa_tree');
  return tree.heads;
}

/**
 * One page of the accounts of one control, with the control's own balance over all of them (plan 38
 * B1, Code Standards B31). A control can hold thousands of accounts, so the tree asks when it opens.
 */
export function coaControlAccounts(
  controlId: number,
  offset: number,
  limit: number,
): Promise<CoaControlAccounts> {
  return callCommand<CoaControlAccounts>('coa_control_accounts', {
    input: { control_id: controlId, offset, limit },
  });
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
 * One page of the accounts for the filter, with the count of the whole filter and the opening
 * totals over all of it (PRD FR-10.5, FR-19.11, Code Standards B31). A page is `limit` rows from
 * `offset`; 0 means the default page of 100.
 */
export function accountsList(
  filter: AccountFilter,
  offset: number,
  limit: number,
  sort: SortInput | null = null,
): Promise<AccountList> {
  return callCommand<AccountList>('accounts_list', { input: { filter, offset, limit, sort } });
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
  /** The tag typed (empty if none): with the names and the phone it decides what is a duplicate. */
  tag: string;
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
    input: accountInputOf(draft, confirmedRulesWords),
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
 * Accounts for a picker: exact code first, then the running number (a short code), then names,
 * then close spellings (PRD FR-7.40). Empty text is every account of the field, the last chosen
 * first, then by code (plan 32, R7). `offset` and `limit` cut out one page. `controlId` is for a
 * field that takes the accounts of one control.
 */
export async function searchAccounts(input: {
  scope: AccountScope;
  controlId: number | null;
  text: string;
  offset?: number;
  limit: number;
  /** A person in the dead amount may be offered (PRD FR-10.10): the receipt and ادھار رقم واپسی fields only. */
  includeDead?: boolean;
}): Promise<AccountHit[]> {
  const hits = await callCommand<AccountHitBinding[]>('search_accounts', {
    input: {
      scope: input.scope,
      control_id: input.controlId,
      text: input.text,
      offset: input.offset ?? 0,
      limit: input.limit,
      // Sent only when it is true: a missing key is "no dead people" (plan 40).
      ...(input.includeDead === true ? { include_dead: true } : {}),
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
  includeDead = false,
): Promise<AccountHit[]> {
  const hits = await callCommand<AccountHitBinding[]>('search_accounts_recent', {
    input: { scope, control_id: controlId, ...(includeDead ? { include_dead: true } : {}) },
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

/** The global search (Ctrl+K): accounts, and a بکری by number or truck (plan 13). */
export async function searchGlobal(text: string): Promise<GlobalHit[]> {
  const hits = await callCommand<GlobalHitBinding[]>('search_global', {
    input: { text, limit: GLOBAL_SEARCH_LIMIT },
  });
  return hits.map((item): GlobalHit => {
    if (item.kind === 'bikri') {
      return {
        kind: 'bikri',
        id: item.hit.id,
        number: item.hit.number,
        entry_date: item.hit.entry_date,
        truck_no: item.hit.truck_no,
        status: item.hit.status,
        matched: item.hit.matched,
      };
    }
    if (item.kind === 'voucher') {
      return {
        kind: 'voucher',
        id: item.hit.id,
        voucher_type: item.hit.voucher_type,
        number: item.hit.number,
        entry_date: item.hit.entry_date,
        account: item.hit.account,
        amount_paisa: item.hit.amount_paisa,
        status: item.hit.status,
      };
    }
    return {
      kind: 'account',
      id: item.hit.id,
      code: item.hit.code,
      name_ur: item.hit.name_ur,
      name_en: item.hit.name_en,
      ...(item.hit.tag === undefined ? {} : { tag: item.hit.tag }),
      extra_ur: item.hit.extra_ur,
      extra_en: item.hit.extra_en,
    };
  });
}
