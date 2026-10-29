// Test helper: an in-memory fake of the account and chart commands (plan 07 contract). The
// wrappers are mocked in the test file with vi.mock; this sets their answers from a small store,
// the way a healthy backend answers. Money is built the way a reply arrives (paisaForTest).

import { vi } from 'vitest';
import type { AccountNode } from '../../src/bindings/AccountNode';
import * as accountApi from '../../src/shared/accounts/api';
import type {
  AccountHit,
  AccountScope,
  ChartAccountsPage,
  PartyCardData,
  ResolvedCode,
  AccountRef,
  AccountRow,
  AccountType,
  Balance,
  ChartHead,
} from '../../src/shared/accounts';
import { formatPaisa } from '../../src/shared/format';
import { paisaForTest } from './paisa';
import { sliceOf } from './paged';

export const ZERO: Balance = { amount_paisa: paisaForTest(0), side: 'zero' };

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

/**
 * The running number of a seller or a buyer: the code without its 3-digit type number and without
 * padding (plan 32, R2), so `0010` is 0 and `00110` is 10. Null for every other account, and for
 * the two پختہ کھاتہ accounts (`0030`, `0031`), which have no short code (R5).
 */
export function runningOf(type: AccountType, code: string): number | null {
  if (type !== 'seller' && type !== 'buyer') return null;
  const prefix = type === 'seller' ? '001' : '002';
  if (!code.startsWith(prefix) || code.length > 10) return null;
  const rest = code.slice(prefix.length);
  return /^(0|[1-9]\d*)$/.test(rest) ? Number(rest) : null;
}

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
    serial: runningOf(type, code) ?? Number(code.slice(-4)),
    account_type: type,
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
    credit_limit_paisa: null,
    default_marka: null,
    collector: null,
    collection_weekday: null,
    is_system: false,
    can_take_opening: true,
    is_pukhta_khata: false,
    is_active: true,
    note: '',
    balance: ZERO,
    ...extra,
  };
}

/**
 * Which accounts the Accounts list shows for a kind (plan 32, R6): "all" is the sellers, the
 * buyers and the two پختہ کھاتہ accounts; cash, payables, capital, نقدی منڈی, اپنا مال, people who
 * borrow, khatas and gardens are never in it. "every" is for the openings grid only.
 */
export function inListKind(row: AccountRow, kind: string): boolean {
  if (kind === 'every') return true;
  const party = (row.account_type === 'seller' || row.account_type === 'buyer') && !row.is_system;
  if (kind === 'pukhta') return row.is_pukhta_khata;
  if (kind === 'seller') return party && row.account_type === 'seller';
  if (kind === 'buyer') return party && row.account_type === 'buyer';
  return party || row.is_pukhta_khata;
}

export interface FakeAccounts {
  rows: AccountRow[];
  tree: ChartHead[];
}

/** The accounts of each control of the chart in miniature (what `coa_control_accounts` answers). */
export const SAMPLE_CONTROL_ACCOUNTS: Record<number, AccountNode[]> = {
  101: [
    {
      id: 1010001,
      code: '01010001',
      name_ur: 'روکڑ',
      name_en: 'Cash in Hand',
      is_active: true,
      is_system: true,
      balance: balance(58484, 'dr'),
    },
  ],
  4: [
    {
      id: 21,
      code: '0020',
      name_ur: 'اکرم ٹول ٹیکس',
      name_en: 'Akram Toll Tax',
      is_active: true,
      is_system: false,
      balance: balance(2880, 'dr'),
    },
    {
      id: 22,
      code: '0021',
      name_ur: 'ارشد ملتانی',
      name_en: 'Arshad Multani',
      is_active: true,
      is_system: false,
      balance: balance(18684, 'dr'),
    },
  ],
  11: [],
};

/**
 * One page of the accounts of a control, as `coa_control_accounts` answers: the rows from `offset`,
 * how many the control has, and the control's own balance (the tree's, over all of them).
 */
export function controlAccountsPage(
  tree: readonly ChartHead[],
  all: Record<number, AccountNode[]>,
  controlId: number,
  offset: number,
  limit: number,
): ChartAccountsPage {
  const accounts = all[controlId] ?? [];
  const control = tree.flatMap((head) => head.controls).find((node) => node.id === controlId);
  const size = limit === 0 ? 100 : limit;
  return {
    control_id: controlId,
    rows: accounts.slice(offset, offset + size),
    total_count: accounts.length,
    offset,
    limit: size,
    balance: control?.balance ?? ZERO,
  };
}

/** The chart of PRD 10.1 in miniature, with balances as the backend would send them. */
export function sampleTree(): ChartHead[] {
  return [
    {
      id: 1,
      code: '01',
      name_ur: 'اثاثے',
      name_en: 'Assets',
      nature: 'asset',
      balance: balance(58484, 'dr'),
      controls: [
        {
          id: 101,
          code: '0101',
          name_ur: 'نقد',
          name_en: 'Cash',
          party_type: 'other',
          balance: balance(58484, 'dr'),
          account_count: 1,
        },
      ],
    },
    {
      id: 3,
      code: '03',
      name_ur: 'پارٹیاں',
      name_en: 'Parties',
      nature: 'party',
      balance: balance(21564, 'dr'),
      controls: [
        {
          id: 4,
          code: '0304',
          name_ur: 'گاہک',
          name_en: 'Buyers',
          party_type: 'buyer',
          balance: balance(21564, 'dr'),
          account_count: 2,
        },
        {
          id: 11,
          code: '03011',
          name_ur: 'بیوپاری',
          name_en: 'Sellers',
          party_type: 'seller',
          balance: ZERO,
          account_count: 0,
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
  tree: ChartHead[] = sampleTree(),
): FakeAccounts {
  const store: FakeAccounts = { rows: [...rows], tree };
  let nextId = 5000;

  vi.mocked(accountApi.coaTree).mockResolvedValue(tree);
  vi.mocked(accountApi.coaControlAccounts).mockImplementation((controlId, offset, limit) =>
    Promise.resolve(controlAccountsPage(tree, SAMPLE_CONTROL_ACCOUNTS, controlId, offset, limit)),
  );
  vi.mocked(accountApi.coaHeadsList).mockResolvedValue(
    tree.map((head) => ({
      id: head.id,
      code: head.code,
      name_ur: head.name_ur,
      name_en: head.name_en,
      nature: head.nature,
    })),
  );
  vi.mocked(accountApi.coaControlsList).mockResolvedValue(
    tree.flatMap((head) =>
      head.controls.map((control) => ({
        id: control.id,
        head_id: head.id,
        code: control.code,
        name_ur: control.name_ur,
        name_en: control.name_en,
        party_type: control.party_type,
        allowed_for_other: control.party_type === 'other',
      })),
    ),
  );
  vi.mocked(accountApi.accountsList).mockImplementation((filter, offset, limit) => {
    const wanted = filter.text.toLowerCase();
    const found = store.rows.filter(
      (row) =>
        (filter.include_inactive || row.is_active) &&
        inListKind(row, filter.kind) &&
        (wanted === '' ||
          [row.code, row.name_ur, row.name_en].some((part) => part.toLowerCase().includes(wanted))),
    );
    // A page of the rows, with the count and the opening totals of the whole filter.
    return Promise.resolve({
      rows: sliceOf(found, offset, limit),
      count: found.length,
      total_dr_paisa: money(65000),
      total_cr_paisa: money(65000),
    });
  });
  vi.mocked(accountApi.accountsGet).mockImplementation((id) => {
    const found = store.rows.find((row) => row.id === id);
    return found === undefined ? Promise.reject(new Error('not found')) : Promise.resolve(found);
  });
  vi.mocked(accountApi.accountsNextCode).mockImplementation((type) => {
    if (type === 'seller' || type === 'buyer') {
      const prefix = type === 'seller' ? '001' : '002';
      const used = store.rows
        .filter((row) => row.account_type === type)
        .map((row) => runningOf(type, row.code) ?? -1);
      return Promise.resolve(`${prefix}${String(Math.max(-1, ...used) + 1)}`);
    }
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
      {
        version: (draft.version ?? 0) + 1,
        city: draft.city,
        phone1: draft.phone1,
        // The backend makes the tag capital and keeps none when it is empty (PRD 10.2).
        ...(draft.tag.trim() === '' ? {} : { tag: draft.tag.trim().toUpperCase() }),
      },
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

function hitOf(row: AccountRow): AccountHit {
  return {
    id: row.id,
    code: row.code,
    account_type: row.account_type,
    control_id: row.control.id,
    name_ur: row.name_ur,
    name_en: row.name_en,
    ...(row.tag === undefined ? {} : { tag: row.tag }),
    extra_ur: row.city?.name_ur ?? '',
    extra_en: row.city?.name_en ?? '',
    is_active: row.is_active,
    balance: row.balance,
  };
}

/** The اپنا مال account: offered only in the seller field of a lot (PRD FR-12.26). */
const OWN_GOODS_CODE = '030119998';

/** A الگ کھاتہ (control 0305): the seller of a lot may be one (PRD FR-16.7), but no other field. */
const isKhata = (row: AccountRow) => row.account_type === 'other' && row.code.startsWith('0305');
/**
 * A garden's account (control 0306, PRD 12.11): offered like a khata, only as the seller of a lot.
 * Its code is the garden's: type 008 and a running number (plan 34), `0080`.
 */
const isGarden = (row: AccountRow) =>
  row.account_type === 'other' && /^008(0|[1-9]\d*)$/.test(row.code);

function inScope(row: AccountRow, scope: AccountScope): boolean {
  if (scope === 'any' || scope === 'control') return true;
  // The seller of a lot may be any seller, اپنا مال, a separate khata and an open garden; every
  // other seller field, and every buyer field, leaves اپنا مال, the khatas and the gardens out.
  if (scope === 'lot_seller') return row.account_type === 'seller' || isKhata(row) || isGarden(row);
  if (scope === 'seller') return row.account_type === 'seller' && row.code !== OWN_GOODS_CODE;
  return row.account_type === scope;
}

/** A typed number that can be a running number: `0`, or digits with no leading zero (plan 32, R5). */
const SHORT_CODE = /^(0|[1-9]\d{0,6})$/;

/**
 * Search, short codes and the party card as the backend answers them (plan 32): an exact full code
 * first, then the running number (a short code) within the scope, then the code or the names with
 * the text in them. Empty text is every account of the scope, a page at a time.
 */
export function fakeAccountSearch(
  rows: AccountRow[],
  cards: Partial<Record<number, PartyCardData>> = {},
) {
  const find = (scope: AccountScope, text: string, withInactive = false): AccountRow[] => {
    const wanted = text.trim().toLowerCase();
    const digits = /^\d+$/.test(wanted);
    // Only active accounts are offered; resolving a code also sees the inactive ones.
    const scoped = rows.filter((row) => inScope(row, scope) && (withInactive || row.is_active));
    if (digits) {
      const exact = scoped.filter((row) => row.code === wanted);
      if (exact.length > 0) return exact;
      if (SHORT_CODE.test(wanted)) {
        // A khata, a garden and a پختہ کھاتہ account have no short code: their full code works.
        const byRunning = scoped.filter(
          (row) => runningOf(row.account_type, row.code) === Number(wanted),
        );
        if (byRunning.length > 0) return byRunning;
      }
    }
    return scoped.filter((row) =>
      [row.code, row.name_ur, row.name_en].some((part) => part.toLowerCase().includes(wanted)),
    );
  };
  vi.mocked(accountApi.searchAccounts).mockImplementation(({ scope, text, offset = 0, limit }) =>
    Promise.resolve(
      find(scope, text)
        .slice(offset, offset + limit)
        .map(hitOf),
    ),
  );
  vi.mocked(accountApi.searchAccountsRecent).mockImplementation((scope) =>
    Promise.resolve(rows.filter((row) => inScope(row, scope)).map(hitOf)),
  );
  vi.mocked(accountApi.searchAccountsTouch).mockResolvedValue(null);
  vi.mocked(accountApi.accountsResolveCode).mockImplementation((scope, _controlId, text) => {
    const digits = /^\d+$/.test(text.trim());
    const found = find(scope, text, true).filter(() => digits);
    const first = found[0];
    const type = scope === 'buyer' ? 'buyer' : 'seller';
    const used = rows
      .filter((row) => row.account_type === type)
      .map((row) => runningOf(type, row.code) ?? -1);
    const next = `${type === 'buyer' ? '002' : '001'}${String(Math.max(-1, ...used) + 1)}`;
    const result: ResolvedCode =
      found.length === 1 && first !== undefined
        ? {
            status: first.is_active ? 'found' : 'inactive',
            account: hitOf(first),
            candidates: [],
            suggested_code: null,
          }
        : found.length > 1
          ? { status: 'several', account: null, candidates: found.map(hitOf), suggested_code: null }
          : {
              status: 'unknown',
              account: null,
              candidates: [],
              // An F4 add would take the next code of the field's type (plan 32, R2).
              suggested_code: digits && (scope === 'seller' || scope === 'buyer') ? next : null,
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
    id: row.id,
    code: row.code,
    account_type: row.account_type,
    is_active: row.is_active,
    name_ur: row.name_ur,
    name_en: row.name_en,
    ...(row.tag === undefined ? {} : { tag: row.tag }),
    father_or_firm_ur: row.father_or_firm_ur,
    father_or_firm_en: row.father_or_firm_en,
    city_ur: row.city?.name_ur ?? '',
    city_en: row.city?.name_en ?? '',
    phone: row.phone1,
    balance: row.balance,
    last_entry_date: '2026-10-10',
    credit_limit: row.account_type === 'buyer' ? row.credit_limit_paisa : null,
    credit_used: row.account_type === 'buyer' ? money(2880) : null,
    advance_open: row.account_type === 'seller' ? money(15000) : null,
    warnings: [],
    ...extra,
  };
}

const rupeesOf = (text: string): number => Number(text.replaceAll(',', '') || '0');

/** `DD-MM-YYYY` as the backend stores it, `YYYY-MM-DD`. */
const isoOf = (text: string): string => text.split('-').reverse().join('-');

/** The side and amount of ابتدائی فرق for these totals: it takes the opposite side of the excess. */
function differenceOf(totalDr: number, totalCr: number): Balance {
  if (totalDr === totalCr) return ZERO;
  return totalDr > totalCr ? balance(totalDr - totalCr, 'cr') : balance(totalCr - totalDr, 'dr');
}

/**
 * Sets the answers of the openings grid commands from the store's accounts, the way a healthy
 * backend answers: a line with both sides is refused on its debit cell, the totals are those of
 * the lines, and the difference is that of every opening after the lines are applied. The sums
 * here are the fake backend's; the screen only shows them.
 */
export function fakeOpenings(store: FakeAccounts) {
  const after = (
    lines: readonly { account_id: number; opening_dr: string; opening_cr: string }[],
  ) => {
    let dr = 0;
    let cr = 0;
    for (const row of store.rows) {
      const line = lines.find((candidate) => candidate.account_id === row.id);
      dr +=
        line === undefined
          ? rupeesOf(formatPaisa(row.opening_dr_paisa, { grouping: 'international' }))
          : rupeesOf(line.opening_dr);
      cr +=
        line === undefined
          ? rupeesOf(formatPaisa(row.opening_cr_paisa, { grouping: 'international' }))
          : rupeesOf(line.opening_cr);
    }
    return { dr, cr };
  };
  vi.mocked(accountApi.accountsOpeningsPreview).mockImplementation(({ lines }) => {
    let totalDr = 0;
    let totalCr = 0;
    const rows = lines.map((line) => {
      const found = store.rows.find((row) => row.id === line.account_id);
      const dr = rupeesOf(line.opening_dr);
      const cr = rupeesOf(line.opening_cr);
      const both = dr > 0 && cr > 0;
      if (!both) {
        totalDr += dr;
        totalCr += cr;
      }
      return {
        account:
          found === undefined ? null : ref(found.id, found.code, found.name_ur, found.name_en),
        errors: both
          ? [{ code: 'ERR_OPENING_BOTH_SIDES' as const, field: 'opening_dr', values: {} }]
          : [],
        opening_dr_paisa: money(both ? 0 : dr),
        opening_cr_paisa: money(both ? 0 : cr),
      };
    });
    const everything = after(lines);
    return Promise.resolve({
      rows,
      total_dr_paisa: money(totalDr),
      total_cr_paisa: money(totalCr),
      opening_difference_after: differenceOf(everything.dr, everything.cr),
    });
  });
  vi.mocked(accountApi.accountsOpeningsSave).mockImplementation(({ lines }) => {
    const saved: AccountRow[] = [];
    for (const line of lines) {
      const index = store.rows.findIndex((row) => row.id === line.account_id);
      const found = store.rows[index];
      if (found === undefined) continue;
      const changed: AccountRow = {
        ...found,
        version: found.version + 1,
        opening_date: isoOf(line.opening_date),
        opening_dr_paisa: money(rupeesOf(line.opening_dr)),
        opening_cr_paisa: money(rupeesOf(line.opening_cr)),
        opening_is_advance: line.opening_is_advance,
      };
      store.rows[index] = changed;
      saved.push(changed);
    }
    const everything = after([]);
    return Promise.resolve({
      rows: saved,
      opening_difference: differenceOf(everything.dr, everything.cr),
    });
  });
}
