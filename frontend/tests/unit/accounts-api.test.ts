// The account search wrappers of plan 08 (PRD FR-7.38 to FR-7.42) against the command shapes of
// the generated bindings: one balance call for all the rows, rows without a balance when it cannot
// be read, recent lists by scope, codes resolved, and the global search flattened. The bridge is
// mocked, so this checks what goes to the backend and what comes back.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as invoke from '../../src/shared/api/invoke';
import {
  accountsResolveCode,
  searchAccounts,
  searchAccountsRecent,
  searchAccountsTouch,
  searchGlobal,
} from '../../src/shared/accounts/api';
import { balance } from '../helpers/accounts';

vi.mock('../../src/shared/api/invoke');

const HIT = {
  id: 21,
  code: '03040001',
  name_ur: 'اکرم',
  name_en: 'Akram',
  extra_ur: '',
  extra_en: '',
  account_type: 'buyer' as const,
  control_id: 4,
  is_active: true,
};

const call = vi.mocked(invoke.callCommand);

/** Answers each command by name; a command not listed fails. */
function backend(answers: Record<string, unknown>) {
  call.mockImplementation((command: string) => {
    if (command in answers) return Promise.resolve(answers[command] as never);
    return Promise.reject(new Error(`no answer for ${command}`));
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('searchAccounts', () => {
  it('asks for the balances of all the hits in one call and puts each on its row', async () => {
    backend({
      search_accounts: [HIT, { ...HIT, id: 22, code: '03040002' }],
      accounts_balances: [
        { id: 21, balance: balance(2880, 'dr') },
        { id: 22, balance: balance(0, 'zero') },
      ],
    });
    const hits = await searchAccounts({ scope: 'buyer', controlId: null, text: 'ak', limit: 50 });
    expect(call).toHaveBeenCalledWith('search_accounts', {
      input: { scope: 'buyer', control_id: null, text: 'ak', limit: 50 },
    });
    expect(call).toHaveBeenCalledWith('accounts_balances', { input: { ids: [21, 22] } });
    expect(call).toHaveBeenCalledTimes(2);
    expect(hits.map((hit) => hit.balance?.side)).toEqual(['dr', 'zero']);
  });

  it('asks for no balances when nothing matched', async () => {
    backend({ search_accounts: [] });
    expect(await searchAccounts({ scope: 'any', controlId: null, text: 'x', limit: 5 })).toEqual(
      [],
    );
    expect(call).toHaveBeenCalledTimes(1);
  });

  it('still gives the hits, without a balance, when the balances cannot be read', async () => {
    backend({ search_accounts: [HIT] });
    const hits = await searchAccounts({ scope: 'buyer', controlId: null, text: 'ak', limit: 5 });
    expect(hits).toHaveLength(1);
    expect(hits[0]?.balance).toBeNull();
  });

  it('sends the control for a field that takes the accounts of one control', async () => {
    backend({ search_accounts: [] });
    await searchAccounts({ scope: 'control', controlId: 3, text: 'as', limit: 5 });
    expect(call).toHaveBeenCalledWith('search_accounts', {
      input: { scope: 'control', control_id: 3, text: 'as', limit: 5 },
    });
  });
});

describe('recent accounts', () => {
  const recentHit = (type: string) => ({
    id: 7,
    code: '030110001',
    name_ur: 'رحیم',
    name_en: 'Rahim',
    extra_ur: '',
    extra_en: '',
    account_type: type,
    control_id: 5,
    is_active: true,
  });

  it('asks the recent list of the field: seller, buyer, any account, or one control', async () => {
    backend({ search_accounts_recent: [], accounts_balances: [] });
    await searchAccountsRecent('seller', null);
    await searchAccountsRecent('buyer', null);
    await searchAccountsRecent('any', null);
    await searchAccountsRecent('control', 3);
    expect(call.mock.calls.map((entry) => JSON.stringify(entry))).toEqual([
      '["search_accounts_recent",{"input":{"scope":"seller","control_id":null}}]',
      '["search_accounts_recent",{"input":{"scope":"buyer","control_id":null}}]',
      '["search_accounts_recent",{"input":{"scope":"any","control_id":null}}]',
      '["search_accounts_recent",{"input":{"scope":"control","control_id":3}}]',
    ]);
  });

  it('takes the accounts as the backend sends them, with their balances read in one call', async () => {
    backend({
      search_accounts_recent: [recentHit('seller')],
      accounts_balances: [{ id: 7, balance: balance(100, 'cr') }],
    });
    const hits = await searchAccountsRecent('any', null);
    expect(hits[0]).toMatchObject({ id: 7, account_type: 'seller', control_id: 5 });
    expect(hits[0]?.balance?.side).toBe('cr');
    expect(call.mock.calls.filter((entry) => entry[0] === 'accounts_balances')).toHaveLength(1);
  });

  it('remembers a choice in the list of its field, with the control for a field of one control', async () => {
    backend({ search_accounts_touch: null });
    await searchAccountsTouch('buyer', null, 21);
    await searchAccountsTouch('control', 3, 22);
    expect(call).toHaveBeenNthCalledWith(1, 'search_accounts_touch', {
      input: { scope: 'buyer', control_id: null, id: 21 },
    });
    expect(call).toHaveBeenNthCalledWith(2, 'search_accounts_touch', {
      input: { scope: 'control', control_id: 3, id: 22 },
    });
  });
});

describe('accountsResolveCode', () => {
  it('sends the scope, the control and the text, and gives the account without a balance', async () => {
    backend({
      accounts_resolve_code: {
        status: 'found',
        account: HIT,
        candidates: [],
        suggested_code: null,
      },
    });
    const result = await accountsResolveCode('buyer', null, '1');
    expect(call).toHaveBeenCalledWith('accounts_resolve_code', {
      input: { scope: 'buyer', control_id: null, text: '1' },
    });
    expect(result.status).toBe('found');
    expect(result.account).toMatchObject({ id: 21, balance: null });
  });

  it('an inactive account comes back as such, with the account so the screen can name it', async () => {
    backend({
      accounts_resolve_code: {
        status: 'inactive',
        account: { ...HIT, is_active: false },
        candidates: [],
        suggested_code: null,
      },
    });
    const result = await accountsResolveCode('buyer', null, '1');
    expect(result.status).toBe('inactive');
    expect(result.account).toMatchObject({ id: 21, is_active: false });
  });

  it('keeps the several candidates and the suggested code of an unknown one', async () => {
    backend({
      accounts_resolve_code: {
        status: 'several',
        account: null,
        candidates: [HIT, { ...HIT, id: 31, account_type: 'seller' }],
        suggested_code: null,
      },
    });
    const several = await accountsResolveCode('any', null, '1');
    expect(several.candidates.map((hit) => hit.id)).toEqual([21, 31]);
    backend({
      accounts_resolve_code: {
        status: 'unknown',
        account: null,
        candidates: [],
        suggested_code: '03040003',
      },
    });
    expect((await accountsResolveCode('buyer', null, '03040003')).suggested_code).toBe('03040003');
  });
});

describe('searchGlobal', () => {
  it('flattens each hit and asks for a short list', async () => {
    backend({ search_global: [{ kind: 'account', hit: HIT }] });
    const found = await searchGlobal('ak');
    expect(call).toHaveBeenCalledWith('search_global', { input: { text: 'ak', limit: 20 } });
    expect(found).toEqual([
      {
        kind: 'account',
        id: 21,
        code: '03040001',
        name_ur: 'اکرم',
        name_en: 'Akram',
        extra_ur: '',
        extra_en: '',
      },
    ]);
  });
});
