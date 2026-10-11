// Test helper: an in-memory fake of the account and chart commands (plan 07 contract). The
// wrappers are mocked in the test file with vi.mock; this sets their answers from a small store,
// the way a healthy backend answers. Money is built the way a reply arrives (paisaForTest).

import { vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import type {
  AccountHit,
  AccountScope,
  PartyCardData,
  ResolvedCode,
  AccountRef,
  AccountRow,
  AccountType,
  Balance,
  CoaHead,
} from '../../src/shared/accounts';
import { paisaForTest } from './paisa';

export const ZERO: Balance = { amount_paisa: paisaForTest(0), side: 'none' };

export const money = (rupees: number): ReturnType<typeof paisaForTest> =>
  paisaForTest(rupees * 100);

export const balance = (rupees: number, side: Balance['side']): Balance => ({
  amount_paisa: money(rupees),
  side,
});

export const ref = (id: number, code: string, ur: string, en: string): AccountRef => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});

const CONTROL_OF: Record<AccountType, AccountRef> = {
  seller: ref(11, '03011', 'بیوپاری', 'Sellers'),
  buyer: ref(4, '0304', 'گاہک', 'Buyers'),
  money_lent: ref(3, '0103', 'ادھار رقم', 'Money Lent'),
  other: ref(1, '0101', 'نقد', 'Cash'),
};

export function account(
  id: number,
  type: AccountType,
  code: string,
  ur: string,
  en: string,
  extra: Partial<AccountRow> = {},
): AccountRow {
  const control = CONTROL_OF[type];
  return {
    id,
    version: 1,
    code,
    account_type: type,
    control_id: control.id,
    control,
    name_ur: ur,
    name_en: en,
    father_or_firm_ur: '',
    father_or_firm_en: '',
    city: null,
    adda: null,
    phone1: '',
    phone2: '',
    address: '',
    linked_account: null,
    opening_date: '2026-08-31',
    opening_dr_paisa: money(0),
    opening_cr_paisa: money(0),
    opening_is_advance: false,
    credit_allowed: true,
    credit_limit_paisa: money(0),
    default_marka: null,
    collector: null,
    collection_weekday: null,
    is_system: false,
    is_active: true,
    note: '',
    balance: ZERO,
    ...extra,
  };
}

export interface FakeAccounts {
  rows: AccountRow[];
  tree: CoaHead[];
}

/** The chart of PRD 10.1 in miniature, with balances as the backend would send them. */
export function sampleTree(): CoaHead[] {
  return [
    {
      id: 1,
      code: '01',
      name_ur: 'اثاثے',
      name_en: 'Assets',
      balance: balance(58484, 'dr'),
      controls: [
        {
          id: 101,
          code: '0101',
          name_ur: 'نقد',
          name_en: 'Cash',
          balance: balance(58484, 'dr'),
          accounts: [
            {
              id: 1010001,
              code: '01010001',
              name_ur: 'روکڑ',
              name_en: 'Cash in Hand',
              is_active: true,
              balance: balance(58484, 'dr'),
            },
          ],
        },
      ],
    },
    {
      id: 3,
      code: '03',
      name_ur: 'پارٹیاں',
      name_en: 'Parties',
      balance: balance(21564, 'dr'),
      controls: [
        {
          id: 4,
          code: '0304',
          name_ur: 'گاہک',
          name_en: 'Buyers',
          balance: balance(21564, 'dr'),
          accounts: [
            {
              id: 21,
              code: '03040001',
              name_ur: 'اکرم ٹول ٹیکس',
              name_en: 'Akram Toll Tax',
              is_active: true,
              balance: balance(2880, 'dr'),
            },
            {
              id: 22,
              code: '03040002',
              name_ur: 'ارشد ملتانی',
              name_en: 'Arshad Multani',
              is_active: true,
              balance: balance(18684, 'dr'),
            },
          ],
        },
        {
          id: 11,
          code: '03011',
          name_ur: 'بیوپاری',
          name_en: 'Sellers',
          balance: ZERO,
          accounts: [],
        },
      ],
    },
  ];
}

/**
 * Sets the answers of the account and chart commands from `rows` and returns the store, so a test
 * can read what was saved. The wrappers must have been mocked with vi.mock in the test file.
 */
export function fakeAccounts(
  rows: AccountRow[] = [],
  tree: CoaHead[] = sampleTree(),
): FakeAccounts {
  const store: FakeAccounts = { rows: [...rows], tree };
  let nextId = 5000;

  vi.mocked(accountApi.coaTree).mockResolvedValue(tree);
  vi.mocked(accountApi.coaHeadsList).mockResolvedValue(tree);
  vi.mocked(accountApi.coaControlsList).mockResolvedValue(tree.flatMap((head) => head.controls));
  vi.mocked(accountApi.accountsList).mockImplementation((filter) => {
    const wanted = filter.text.toLowerCase();
    const found = store.rows.filter(
      (row) =>
        (filter.include_inactive || row.is_active) &&
        (filter.account_type === null || row.account_type === filter.account_type) &&
        (wanted === '' ||
          [row.code, row.name_ur, row.name_en].some((part) => part.toLowerCase().includes(wanted))),
    );
    return Promise.resolve({
      rows: found,
      total_dr_paisa: money(65000),
      total_cr_paisa: money(65000),
    });
  });
  vi.mocked(accountApi.accountsGet).mockImplementation((id) => {
    const found = store.rows.find((row) => row.id === id);
    return found === undefined ? Promise.reject(new Error('not found')) : Promise.resolve(found);
  });
  vi.mocked(accountApi.accountsNextCode).mockImplementation((type) => {
    const control = CONTROL_OF[type];
    const highest = Math.max(
      0,
      ...store.rows
        .filter((row) => row.account_type === type)
        .map((row) => Number(row.code.slice(control.code.length)) || 0),
    );
    return Promise.resolve(`${control.code}${String(highest + 1).padStart(4, '0')}`);
  });
  vi.mocked(accountApi.accountsSimilar).mockResolvedValue([]);
  vi.mocked(accountApi.accountsSave).mockImplementation((draft) => {
    const saved = account(
      draft.id ?? (nextId += 1),
      draft.account_type,
      draft.code,
      draft.names.urdu,
      draft.names.english,
      { version: (draft.version ?? 0) + 1, city: draft.city, phone1: draft.phone1 },
    );
    const index = store.rows.findIndex((row) => row.id === saved.id);
    if (index >= 0) store.rows[index] = saved;
    else store.rows.push(saved);
    return Promise.resolve(saved);
  });
  vi.mocked(accountApi.accountsSetActive).mockImplementation((id, isActive) => {
    const index = store.rows.findIndex((row) => row.id === id);
    const found = store.rows[index];
    if (found === undefined) return Promise.reject(new Error('not found'));
    const changed = { ...found, is_active: isActive, version: found.version + 1 };
    store.rows[index] = changed;
    return Promise.resolve(changed);
  });
  return store;
}

const SERIAL_LENGTH = 4;
const RESERVED_SERIAL = (serial: number) => serial === 0 || serial >= 9000;

function hitOf(row: AccountRow): AccountHit {
  return {
    id: row.id,
    code: row.code,
    account_type: row.account_type,
    name_ur: row.name_ur,
    name_en: row.name_en,
    extra_ur: row.city?.name_ur ?? '',
    extra_en: row.city?.name_en ?? '',
    is_active: row.is_active,
    balance: row.balance,
  };
}

function inScope(row: AccountRow, scope: AccountScope): boolean {
  return scope === 'any' ? true : row.account_type === scope;
}

/** The serial of an account: the last four digits of its code (PRD FR-10.3). */
const serialOf = (row: AccountRow) => Number(row.code.slice(-SERIAL_LENGTH));

/**
 * Search, short codes and the party card as the backend answers them (plan 08 Section 3): an exact
 * code first, then the exact serial within the scope, then names in either script. A code typed
 * alone is resolved on its own.
 */
export function fakeAccountSearch(
  rows: AccountRow[],
  cards: Partial<Record<number, PartyCardData>> = {},
) {
  const find = (scope: AccountScope, text: string): AccountRow[] => {
    const wanted = text.trim().toLowerCase();
    const digits = /^\d+$/.test(wanted);
    const scoped = rows.filter((row) => inScope(row, scope));
    if (digits) {
      const exact = scoped.filter((row) => row.code === wanted);
      if (exact.length > 0) return exact;
      if (wanted.length <= SERIAL_LENGTH) {
        const bySerial = scoped.filter((row) => serialOf(row) === Number(wanted));
        if (bySerial.length > 0) return bySerial;
      }
    }
    return scoped.filter((row) =>
      [row.code, row.name_ur, row.name_en].some((part) => part.toLowerCase().includes(wanted)),
    );
  };
  vi.mocked(accountApi.searchAccounts).mockImplementation(({ scope, text }) =>
    Promise.resolve(find(scope, text).map(hitOf)),
  );
  vi.mocked(accountApi.searchAccountsRecent).mockImplementation((scope) =>
    Promise.resolve(rows.filter((row) => inScope(row, scope)).map(hitOf)),
  );
  vi.mocked(accountApi.searchAccountsTouch).mockResolvedValue(null);
  vi.mocked(accountApi.accountsResolveCode).mockImplementation((scope, text) => {
    const digits = /^\d+$/.test(text.trim());
    const serial = Number(text.trim());
    if (digits && text.trim().length <= SERIAL_LENGTH && RESERVED_SERIAL(serial)) {
      return Promise.resolve<ResolvedCode>({
        status: 'reserved',
        account: null,
        candidates: [],
        suggested_code: null,
      });
    }
    const found = find(scope, text).filter(() => digits);
    const first = found[0];
    const result: ResolvedCode =
      found.length === 1 && first !== undefined
        ? { status: 'found', account: hitOf(first), candidates: [], suggested_code: null }
        : found.length > 1
          ? { status: 'several', account: null, candidates: found.map(hitOf), suggested_code: null }
          : {
              status: 'unknown',
              account: null,
              candidates: [],
              suggested_code: digits && text.trim().length > SERIAL_LENGTH ? text.trim() : null,
            };
    return Promise.resolve(result);
  });
  vi.mocked(accountApi.assistPartyCard).mockImplementation((id) => {
    const card = cards[id];
    if (card !== undefined) return Promise.resolve(card);
    const row = rows.find((candidate) => candidate.id === id);
    if (row === undefined) return Promise.reject(new Error('not found'));
    return Promise.resolve(cardOf(row));
  });
}

/** The party card of an account, the way the backend builds it. */
export function cardOf(row: AccountRow, extra: Partial<PartyCardData> = {}): PartyCardData {
  return {
    account_id: row.id,
    code: row.code,
    account_type: row.account_type,
    name_ur: row.name_ur,
    name_en: row.name_en,
    father_or_firm_ur: row.father_or_firm_ur,
    father_or_firm_en: row.father_or_firm_en,
    city_ur: row.city?.name_ur ?? '',
    city_en: row.city?.name_en ?? '',
    phone: row.phone1,
    balance: row.balance,
    last_entry_date: '2026-10-10',
    credit_limit_paisa: row.account_type === 'buyer' ? row.credit_limit_paisa : null,
    credit_used_paisa: row.account_type === 'buyer' ? money(2880) : null,
    advance_open_paisa: row.account_type === 'seller' ? money(15000) : null,
    warnings: [],
    ...extra,
  };
}
