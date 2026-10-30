// The answers of the development page /dev/lists/<screen> (plan 38, A1): the lists and the screens of
// the انڈکس area, the rules, the audit log and the partner setup, for looking at them in a plain
// browser tab in both languages at every window size, with a few rows or with many (`?many`: 340
// rows, to see the page buttons). dev-bridge.ts asks this table first while the address is
// /dev/lists. Flags in the hash: `?many`, `&empty`, `&down` (every list refused). The rows are
// sample data: names and codes only, no money is worked out here (the opening figures are fixed
// numbers). Never part of a build: only the development route imports it.

import type { AuditRow } from '../../src/bindings/AuditRow';
import type { HeadNode } from '../../src/bindings/HeadNode';
import type { PartnerRow } from '../../src/bindings/PartnerRow';
import type { RuleRow } from '../../src/bindings/RuleRow';
import auditFirstPage from '../fixtures/b5_audit_list_first_page.json';
import auditOptions from '../fixtures/b5_audit_options.json';
import itemsList from '../fixtures/masters_list_items.json';
import rulesAll from '../fixtures/rules_list_all.json';
import { account, balance, ref } from './accounts';
import { sliceOf } from './paged';
import { paisaForTest } from './paisa';

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

/** True on the development page of this area. */
export function isListsPage(): boolean {
  return window.location.hash.startsWith('#/dev/lists');
}

const flags = (): URLSearchParams => new URLSearchParams(window.location.hash.split('?')[1] ?? '');
const flag = (name: string): boolean => flags().has(name);
const MANY = 340;

/** The backend's refusal for every list, for the error state (`&down`). */
function down(): Error {
  return Object.assign(new Error('ERR_INTERNAL'), {
    ok: false,
    errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
    ref: 'E-1',
  });
}

const FIRST_NAMES: readonly (readonly [string, string])[] = [
  ['اکرم', 'Akram'],
  ['ارشد', 'Arshad'],
  ['رحیم', 'Rahim'],
  ['کریم', 'Karim'],
  ['شاہد', 'Shahid'],
  ['ناصر', 'Nasir'],
  ['بلال', 'Bilal'],
  ['عمران', 'Imran'],
];
const FAMILY: readonly (readonly [string, string])[] = [
  ['ملتانی', 'Multani'],
  ['ٹول ٹیکس', 'Toll Tax'],
  ['فروٹ ہاؤس', 'Fruit House'],
  ['برادرز', 'Brothers'],
];

function nameOf(index: number): readonly [string, string] {
  const first = FIRST_NAMES[index % FIRST_NAMES.length] ?? FIRST_NAMES[0] ?? ['', ''];
  const family = FAMILY[Math.floor(index / FIRST_NAMES.length) % FAMILY.length] ??
    FAMILY[0] ?? ['', ''];
  const round = Math.floor(index / (FIRST_NAMES.length * FAMILY.length));
  const suffixUr = round === 0 ? '' : ` ${String(round + 1)}`;
  return [`${first[0]} ${family[0]}${suffixUr}`, `${first[1]} ${family[1]}${suffixUr}`];
}

function accounts(count: number) {
  return Array.from({ length: count }, (_, index) => {
    const type = index % 2 === 0 ? 'buyer' : 'seller';
    const serial = Math.floor(index / 2) + 1;
    const code =
      type === 'buyer'
        ? `002${String(serial).padStart(4, '0')}`
        : `001${String(serial).padStart(4, '0')}`;
    const [ur, en] = nameOf(index);
    const owes = (index * 7919) % 90000;
    return account(1000 + index, type, code, ur, en, {
      city: ref(1, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan'),
      phone1: `0300${String(1000000 + ((index * 7331) % 8999999))}`,
      opening_date: '2026-09-01',
      opening_dr_paisa: paisaForTest(type === 'buyer' ? owes * 100 : 0),
      opening_cr_paisa: paisaForTest(type === 'seller' ? owes * 100 : 0),
      credit_allowed: true,
      is_active: index % 23 !== 7,
      balance: type === 'buyer' ? balance(owes, 'dr') : balance(owes, 'cr'),
    });
  });
}

function allAccounts() {
  return flag('empty') ? [] : accounts(flag('many') ? MANY : 9);
}

const CITIES = Array.from({ length: MANY }, (_, index) => ({
  kind: 'city' as const,
  id: index + 1,
  version: 1,
  code: String(index + 1).padStart(4, '0'),
  name_ur: `شہر ${String(index + 1)}`,
  name_en: `City ${String(index + 1)}`,
  is_active: index % 17 !== 5,
}));

function masterRows(kind: string) {
  const many = flag('many');
  if (flag('empty')) return [];
  if (kind === 'item') {
    const real = itemsList.rows;
    if (!many) return real;
    return Array.from({ length: MANY }, (_, index) => {
      const base = real[index % real.length];
      return {
        ...base,
        id: 100 + index,
        code: String(1000 + index),
        name_en: `${base?.name_en ?? ''} ${String(index + 1)}`,
      };
    });
  }
  if (kind === 'packing') {
    return Array.from({ length: many ? MANY : 5 }, (_, index) => ({
      kind: 'packing' as const,
      id: index + 1,
      version: 1,
      code: String(index + 1).padStart(4, '0'),
      name_ur: `پیکنگ ${String(index + 1)}`,
      name_en: `Packing ${String(index + 1)}`,
      approx_kg: index % 2 === 0 ? '12.500' : null,
      is_active: true,
    }));
  }
  return many ? CITIES : CITIES.slice(0, 6);
}

function rules(): RuleRow[] {
  const real = rulesAll.rows as unknown as RuleRow[];
  if (flag('empty')) return [];
  if (!flag('many')) return real;
  return Array.from({ length: MANY }, (_, index) => {
    const base = real[index % real.length] ?? real[0];
    return base === undefined
      ? []
      : [
          {
            ...base,
            id: 100 + index,
            is_default: false,
            note: `${base.note} ${String(index + 1)}`,
          },
        ];
  }).flat();
}

function auditRows(): AuditRow[] {
  const real = auditFirstPage.rows as unknown as AuditRow[];
  if (flag('empty')) return [];
  if (!flag('many')) return real;
  return Array.from({ length: MANY }, (_, index) => {
    const base = real[index % real.length] ?? real[0];
    return base === undefined ? [] : [{ ...base, id: 5000 - index }];
  }).flat();
}

const PARTNERS: PartnerRow[] = [
  ['الف', 'Partner A', 4000],
  ['ب', 'Partner B', 3000],
  ['ج', 'Partner C', 3000],
].map(([ur, en, bps], index) => ({
  id: index + 1,
  name_ur: String(ur),
  name_en: String(en),
  capital_account_id: 200 + index,
  capital_account_code: `0602000${String(index + 1)}`,
  drawings_account_id: 300 + index,
  drawings_account_code: `0603000${String(index + 1)}`,
  venture_account_id: 400 + index,
  venture_account_code: `0604000${String(index + 1)}`,
  share_bps: Number(bps),
  is_active: true,
  version: 1,
}));

const CHART: HeadNode[] = [
  [
    '01',
    'اثاثے',
    'Assets',
    'asset',
    [
      ['0101', 'نقد', 'Cash', 'other', 1],
      ['0103', 'ادھار رقم', 'Money Lent', 'other', 3],
    ],
  ],
  [
    '03',
    'پارٹیاں',
    'Parties',
    'party',
    [
      ['03011', 'بیوپاری', 'Sellers', 'seller', 120],
      ['0304', 'گاہک', 'Buyers', 'buyer', 220],
    ],
  ],
  ['04', 'آمدن', 'Income', 'income', [['0401', 'کمیشن آمدن', 'Commission Income', 'other', 5]]],
].map(([code, ur, en, nature, controls], index) => ({
  id: index + 1,
  code: String(code),
  name_ur: String(ur),
  name_en: String(en),
  nature: String(nature),
  balance: balance(index * 12000 + 8000, index === 1 ? 'dr' : 'cr'),
  controls: (controls as unknown as [string, string, string, string, number][]).map(
    ([controlCode, controlUr, controlEn, party, count], position) => ({
      id: (index + 1) * 10 + position,
      code: controlCode,
      name_ur: controlUr,
      name_en: controlEn,
      party_type: party,
      balance: balance(position * 5000 + 1000, 'dr'),
      account_count: count,
    }),
  ),
}));

/** The accounts of one control of the chart: as many as the control says it has (`account_count`). */
function controlAccounts(input: Input) {
  const controlId = Number(input['control_id']);
  const control = CHART.flatMap((head) => head.controls).find((node) => node.id === controlId);
  const count =
    control === undefined
      ? 0
      : flag('many')
        ? control.account_count
        : Math.min(control.account_count, 6);
  const all = accounts(count).map((row) => ({
    id: row.id,
    code: row.code,
    name_ur: row.name_ur,
    name_en: row.name_en,
    is_active: row.is_active,
    is_system: false,
    balance: row.balance,
  }));
  const offset = Number(input['offset'] ?? 0);
  const limit = Number(input['limit'] ?? 0) === 0 ? 100 : Number(input['limit']);
  return {
    control_id: controlId,
    rows: all.slice(offset, offset + limit),
    total_count: all.length,
    offset,
    limit,
    balance: control?.balance ?? balance(0, 'zero'),
  };
}

export const listsHandlers: Record<string, Handler> = {
  coa_control_accounts: controlAccounts,
  accounts_list: (input) => {
    if (flag('down')) throw down();
    const filter = (input['filter'] ?? {}) as {
      kind?: string;
      text?: string;
      include_inactive?: boolean;
    };
    const text = (filter.text ?? '').trim().toLowerCase();
    const all = allAccounts().filter(
      (row) =>
        (filter.include_inactive === true || row.is_active) &&
        (filter.kind === undefined ||
          filter.kind === 'all' ||
          filter.kind === 'every' ||
          row.account_type === filter.kind) &&
        (text === '' ||
          row.name_en.toLowerCase().includes(text) ||
          row.name_ur.includes(text) ||
          row.code.includes(text)),
    );
    return {
      rows: sliceOf(all, Number(input['offset'] ?? 0), Number(input['limit'] ?? 0)),
      count: all.length,
      total_dr_paisa: paisaForTest(all.length * 4500000),
      total_cr_paisa: paisaForTest(all.length * 3900000),
    };
  },
  coa_tree: () => {
    if (flag('down')) throw down();
    return { heads: CHART };
  },
  masters_list: (input) => {
    if (flag('down')) throw down();
    const rows = masterRows(String(input['kind']));
    const wanted = input['id'];
    const shown = typeof wanted === 'number' ? rows.filter((row) => row.id === wanted) : rows;
    const kept = input['include_inactive'] === true ? shown : shown.filter((row) => row.is_active);
    return {
      rows: sliceOf(kept, Number(input['offset'] ?? 0), Number(input['limit'] ?? 0)),
      total_count: kept.length,
      totals: { active_count: shown.filter((row) => row.is_active).length },
    };
  },
  search_masters: () => [],
  rules_list: (input) => {
    if (flag('down')) throw down();
    const all = rules();
    return {
      rows: sliceOf(all, Number(input['offset'] ?? 0), Number(input['limit'] ?? 0)),
      total_count: all.length,
      totals: { active_count: all.filter((row) => row.is_active).length },
    };
  },
  audit_options: () => auditOptions,
  audit_list: (input) => {
    if (flag('down')) throw down();
    const all = auditRows();
    const offset = Number(input['offset'] ?? 0);
    const limit = Number(input['limit'] ?? 100);
    return {
      ...auditFirstPage,
      rows: sliceOf(all, offset, limit),
      offset,
      limit,
      total_count: all.length,
      months: [],
    };
  },
  partners_list: () => {
    if (flag('down')) throw down();
    return PARTNERS;
  },
  partners_check_shares: () => ({ total_bps: 10000, is_100: true, errors: [] }),
  prefs_screen_get: (input) => ({ screen: input['screen'], value: null }),
  prefs_screen_set: (input) => ({ screen: input['screen'], value: null }),
};
