// Test data: the books of gardens (plan 17c contract, PRD 12.11), in memory, with no test runner in
// it, so the development page can use it too. A test sets the command wrappers from it
// (tests/helpers/gardens.ts); the development page /dev/gardens answers from it. It answers the way
// a healthy backend answers, from the worked numbers of PRD 27.2 E2E-40: Shahpur Garden, partner A
// at 60% from the shop's cash, partner B at 40% from his own pocket, partner C not in it; rent
// 50,000; installments 30,000 and 20,000, labour 5,000, a sale in our shop with 56,400 to the
// garden's account and 10,000 from another shop; then the closing of 66,400 into 39,840 and
// 26,560. The sums in a preview are this test double's, written in the fixture with the plan's
// numbers; the screen only shows what comes back and never works out an amount (CLAUDE.md rule 1).
// Every shape is the generated binding, so a change of the contract fails to compile here first.

import type { CashCheck } from '../../src/bindings/CashCheck';
import type { EntryPart } from '../../src/bindings/EntryPart';
import type { EntryParts } from '../../src/bindings/EntryParts';
import type { ErrorCode } from '../../src/bindings/ErrorCode';
import type { FieldError } from '../../src/bindings/FieldError';
import type { GardenCloseInput } from '../../src/bindings/GardenCloseInput';
import type { GardenClosePreview } from '../../src/bindings/GardenClosePreview';
import type { GardenClosed } from '../../src/bindings/GardenClosed';
import type { GardenCreatePreview } from '../../src/bindings/GardenCreatePreview';
import type { GardenCreated } from '../../src/bindings/GardenCreated';
import type { GardenDefaults } from '../../src/bindings/GardenDefaults';
import type { GardenDetail } from '../../src/bindings/GardenDetail';
import type { GardenEntryCancelInput } from '../../src/bindings/GardenEntryCancelInput';
import type { GardenEntryCancelled } from '../../src/bindings/GardenEntryCancelled';
import type { GardenEntryInput } from '../../src/bindings/GardenEntryInput';
import type { GardenEntryPreview } from '../../src/bindings/GardenEntryPreview';
import type { GardenEntrySaved } from '../../src/bindings/GardenEntrySaved';
import type { GardenGetInput } from '../../src/bindings/GardenGetInput';
import type { GardenHistoryRow } from '../../src/bindings/GardenHistoryRow';
import type { GardenInput } from '../../src/bindings/GardenInput';
import type { GardenPartnerResult } from '../../src/bindings/GardenPartnerResult';
import type { GardenReopenInput } from '../../src/bindings/GardenReopenInput';
import type { GardenReopened } from '../../src/bindings/GardenReopened';
import type { GardenRow } from '../../src/bindings/GardenRow';
import type { GardensList } from '../../src/bindings/GardensList';
import type { GardensListInput } from '../../src/bindings/GardensListInput';
import type { GardensSummary } from '../../src/bindings/GardensSummary';
import type { MoneySource } from '../../src/bindings/MoneySource';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { ShareAmount } from '../../src/bindings/ShareAmount';
import type { VenturePartnerRow } from '../../src/bindings/VenturePartnerRow';
import type { Working } from '../../src/bindings/Working';
import type { WorkingArg } from '../../src/bindings/WorkingArg';
import { CommandError } from '../../src/shared/api';
import { paisaForTest } from './paisa';
import { refusal, type RefusalItem } from './ui-error';

const money = (rupees: number): ReturnType<typeof paisaForTest> => paisaForTest(rupees * 100);

export const GARDEN_ID = 1;

const person = (id: number, ur: string, en: string): RecordRef => ({
  id,
  code: '',
  name_ur: ur,
  name_en: en,
});
export const PARTNER_A = person(1, 'الف', 'Partner A');
export const PARTNER_B = person(2, 'ب', 'Partner B');
export const PARTNER_C = person(3, 'ج', 'Partner C');
export const GARDEN_ACCOUNT: RecordRef = {
  id: 41,
  code: '0080',
  name_ur: 'شاہ پور باغ',
  name_en: 'Shahpur Garden',
};
export const BANANA: RecordRef = { id: 62, code: '0062', name_ur: 'کیلا', name_en: 'Banana' };

/** A working the way the backend names it: a message and its values. */
export function working(id: string, args: readonly WorkingArg[]): Working {
  return { message_id: id, args: [...args], rule_id: null };
}
const moneyArg = (name: string, rupees: number): WorkingArg => ({
  name,
  value: { kind: 'money', paisa: money(rupees) },
});
const countArg = (name: string, value: number): WorkingArg => ({
  name,
  value: { kind: 'count', value },
});

/** The two partners of the garden of E2E-40: A at 60% from the shop, B at 40% from his pocket. */
export function gardenPartners(
  sources: readonly [MoneySource, MoneySource] = ['shop', 'pocket'],
): VenturePartnerRow[] {
  return [
    { partner: PARTNER_A, share_bps: 6000, source: sources[0] },
    { partner: PARTNER_B, share_bps: 4000, source: sources[1] },
  ];
}

/** The partners as a new garden form starts: every active partner, with the firm's shares. */
export function firmPartners(): VenturePartnerRow[] {
  return [
    { partner: PARTNER_A, share_bps: 4000, source: 'shop' },
    { partner: PARTNER_B, share_bps: 3000, source: 'shop' },
    { partner: PARTNER_C, share_bps: 3000, source: 'shop' },
  ];
}

const balanceWorking = (inRupees: number, paidRupees: number, result: number): Working =>
  working('working-garden-balance', [
    moneyArg('money_in', inRupees),
    moneyArg('paid', paidRupees),
    moneyArg('result', result),
  ]);

/** The code of the n-th garden: the type number 008 and n - 1, so the first is 0080 (plan 34). */
export const gardenCode = (number: number) => `008${String(number - 1)}`;

/** A new garden with nothing paid: rent 50,000 still due. Its code follows its number, unless given. */
export function gardenRow(over: Partial<GardenRow> = {}): GardenRow {
  const number = over.number ?? 1;
  return {
    id: GARDEN_ID,
    version: 1,
    number,
    code: gardenCode(number),
    name_ur: 'شاہ پور باغ',
    name_en: 'Shahpur Garden',
    account: GARDEN_ACCOUNT,
    entry_date: '2026-10-10',
    item: BANANA,
    rent_paisa: money(50_000),
    season_end_date: '2026-12-31',
    note: null,
    status: 'open',
    closed_date: null,
    shares_locked: false,
    partners: gardenPartners(),
    installments_paisa: money(0),
    costs_paisa: money(0),
    paid_paisa: money(0),
    paid_shop_paisa: money(0),
    paid_pocket_paisa: money(0),
    rent_due_paisa: money(50_000),
    money_in_paisa: money(0),
    other_shop_paisa: money(0),
    balance_paisa: money(0),
    balance_working: balanceWorking(0, 0, 0),
    account_balance_paisa: money(0),
    last_date: null,
    ...over,
  };
}

/** E2E-40 before the closing: paid 55,000 (33,000 from the shop, 22,000 from a pocket), money in 66,400. */
export function e40Garden(over: Partial<GardenRow> = {}): GardenRow {
  return gardenRow({
    shares_locked: true,
    installments_paisa: money(50_000),
    costs_paisa: money(5_000),
    paid_paisa: money(55_000),
    paid_shop_paisa: money(33_000),
    paid_pocket_paisa: money(22_000),
    rent_due_paisa: money(0),
    money_in_paisa: money(66_400),
    other_shop_paisa: money(10_000),
    balance_paisa: money(11_400),
    balance_working: balanceWorking(66_400, 55_000, 11_400),
    account_balance_paisa: money(66_400),
    last_date: '2026-10-12',
    ...over,
  });
}

/** E2E-40 after the closing on 13-10-26: the garden's account is 0. */
export function e40Closed(): GardenRow {
  return e40Garden({
    status: 'closed',
    closed_date: '2026-10-13',
    account_balance_paisa: money(0),
    version: 2,
  });
}

const partShare = (index: 0 | 1) => (index === 0 ? 6000 : 4000);
const PARTS = [PARTNER_A, PARTNER_B] as const;

function entryPart(index: 0 | 1, rupees: number, source: MoneySource, total: number): EntryPart {
  return {
    partner: PARTS[index],
    share_bps: partShare(index),
    source,
    amount_paisa: money(rupees),
    working: working('working-venture-part', [
      moneyArg('amount', total),
      countArg('bps', partShare(index)),
      moneyArg('result', rupees),
    ]),
  };
}

/** The parts of a payment of `total` rupees, split `a` and `b`, each from the source given. */
export function paymentParts(
  total: number,
  split: readonly [number, number],
  sources: readonly [MoneySource, MoneySource] = ['shop', 'pocket'],
): EntryPart[] {
  return [entryPart(0, split[0], sources[0], total), entryPart(1, split[1], sources[1], total)];
}

/**
 * The two sums of a payment's parts, from the whole rupees the fixture wrote them with (this test
 * double adds rupees it holds itself; the lint rule keeps tests from reading a Paisa).
 */
function partsView(
  parts: EntryPart[],
  rupees: readonly [number, number],
): { view: EntryParts; shop: number } {
  let shop = 0;
  let pocket = 0;
  parts.forEach((part, index) => {
    const amount = rupees[index] ?? 0;
    if (part.source === 'shop') shop += amount;
    else pocket += amount;
  });
  return {
    view: { parts, shop_total_paisa: money(shop), pocket_total_paisa: money(pocket) },
    shop,
  };
}

const credit = (index: 0 | 1, rupees: number, total: number): ShareAmount => ({
  partner: PARTS[index],
  share_bps: partShare(index),
  amount_paisa: money(rupees),
  working: working('working-venture-credit', [
    moneyArg('amount', total),
    countArg('bps', partShare(index)),
    moneyArg('result', rupees),
  ]),
});

/** The closing of E2E-40: 66,400 to 39,840 and 26,560. */
export function closingCredits(): ShareAmount[] {
  return [credit(0, 39_840, 66_400), credit(1, 26_560, 66_400)];
}

const resultWorking = (shop: number, pocket: number, back: number, net: number): Working =>
  working('working-trip-partner-net', [
    moneyArg('in_shop', shop),
    moneyArg('in_pocket', pocket),
    moneyArg('back', back),
    moneyArg('result', net),
  ]);

/** The partners' money in the garden of E2E-40: open (nothing back yet) or closed. */
export function e40Results(closed: boolean): GardenPartnerResult[] {
  return [
    {
      partner: PARTNER_A,
      share_bps: 6000,
      paid_shop_paisa: money(33_000),
      paid_pocket_paisa: money(0),
      money_back_paisa: money(closed ? 39_840 : 0),
      net_paisa: money(closed ? 6_840 : -33_000),
      working: resultWorking(33_000, 0, closed ? 39_840 : 0, closed ? 6_840 : -33_000),
    },
    {
      partner: PARTNER_B,
      share_bps: 4000,
      paid_shop_paisa: money(0),
      paid_pocket_paisa: money(22_000),
      money_back_paisa: money(closed ? 26_560 : 0),
      net_paisa: money(closed ? 4_560 : -22_000),
      working: resultWorking(0, 22_000, closed ? 26_560 : 0, closed ? 4_560 : -22_000),
    },
  ];
}

function historyRow(over: Partial<GardenHistoryRow>): GardenHistoryRow {
  return {
    entry_date: '2026-10-10',
    kind: 'installment',
    entry_id: null,
    version: null,
    bikri_id: null,
    bikri_number: null,
    amount_paisa: money(0),
    effect_paisa: money(0),
    running_balance_paisa: money(0),
    reason: null,
    shop_name: null,
    qty: null,
    journal_entry_id: null,
    cancelled: false,
    cancel_reason: null,
    parts: [],
    credits: [],
    can_cancel: false,
    ...over,
  };
}

/** The history of E2E-40, oldest day first, with the running balance after each row. */
export function e40History(closed = false): GardenHistoryRow[] {
  const rows: GardenHistoryRow[] = [
    historyRow({
      entry_date: '2026-10-10',
      kind: 'installment',
      entry_id: 11,
      version: 1,
      amount_paisa: money(30_000),
      effect_paisa: money(-30_000),
      running_balance_paisa: money(-30_000),
      reason: 'First installment',
      journal_entry_id: 31,
      parts: paymentParts(30_000, [18_000, 12_000]),
      can_cancel: !closed,
    }),
    historyRow({
      entry_date: '2026-10-11',
      kind: 'installment',
      entry_id: 12,
      version: 1,
      amount_paisa: money(20_000),
      effect_paisa: money(-20_000),
      running_balance_paisa: money(-50_000),
      reason: 'Second installment',
      journal_entry_id: 32,
      parts: paymentParts(20_000, [12_000, 8_000]),
      can_cancel: !closed,
    }),
    historyRow({
      entry_date: '2026-10-11',
      kind: 'cost',
      entry_id: 13,
      version: 1,
      amount_paisa: money(5_000),
      effect_paisa: money(-5_000),
      running_balance_paisa: money(-55_000),
      reason: 'Labour',
      journal_entry_id: 33,
      parts: paymentParts(5_000, [3_000, 2_000]),
      can_cancel: !closed,
    }),
    historyRow({
      entry_date: '2026-10-12',
      kind: 'sale',
      bikri_id: 5,
      bikri_number: 5,
      amount_paisa: money(56_400),
      effect_paisa: money(56_400),
      running_balance_paisa: money(1_400),
      journal_entry_id: 34,
    }),
    historyRow({
      entry_date: '2026-10-12',
      kind: 'other_sale',
      entry_id: 14,
      version: 1,
      amount_paisa: money(10_000),
      effect_paisa: money(10_000),
      running_balance_paisa: money(11_400),
      shop_name: 'Haji Shop',
      qty: 20,
      journal_entry_id: 35,
      can_cancel: !closed,
    }),
  ];
  if (closed) {
    rows.push(
      historyRow({
        entry_date: '2026-10-13',
        kind: 'close',
        entry_id: 15,
        version: 1,
        amount_paisa: money(66_400),
        effect_paisa: money(0),
        running_balance_paisa: money(11_400),
        journal_entry_id: 36,
        credits: closingCredits(),
      }),
    );
  }
  return rows;
}

export interface GardenStore {
  gardens: GardenRow[];
  history: GardenHistoryRow[];
  /** The cash in hand before an entry, in rupees (50,000 in the opening position). */
  cashBefore: number;
  /** Every input that was saved, as sent. */
  saved: {
    gardens: GardenInput[];
    entries: GardenEntryInput[];
    closings: GardenCloseInput[];
    cancels: GardenEntryCancelInput[];
    reopens: GardenReopenInput[];
  };
  /** Set to refuse the next save of a kind with these problems. */
  refuse: {
    garden?: RefusalItem[];
    entry?: RefusalItem[];
    close?: RefusalItem[];
    cancel?: RefusalItem[];
    reopen?: RefusalItem[];
  };
}

export interface GardensOptions {
  gardens?: GardenRow[];
  history?: GardenHistoryRow[];
  cashBefore?: number;
  defaults?: GardenDefaults;
}

/** The backend refusing: thrown by the books, so a caller can turn it into a rejected promise. */
function refuse(items: RefusalItem[]): never {
  throw new CommandError(refusal(items));
}

function cashOf(before: number, shopRupees: number): CashCheck {
  const after = before - shopRupees;
  return {
    cash_before_paisa: money(before),
    change_paisa: money(-shopRupees),
    cash_after_paisa: money(after),
    below_zero: after < 0,
    lowers_cash: shopRupees > 0,
    needs_confirm: after < 0 && shopRupees > 0,
  };
}

const problem = (code: ErrorCode, field: string): FieldError => ({ code, field, values: {} });

const sourcesOf = (
  parts: readonly { partner_id: number; source: MoneySource }[],
): [MoneySource, MoneySource] => [
  parts.find((part) => part.partner_id === 1)?.source ?? 'shop',
  parts.find((part) => part.partner_id === 2)?.source ?? 'pocket',
];

/** The split of the payments of E2E-40, by the amount typed: the fixture wrote them with the plan. */
const SPLITS: Record<string, readonly [number, number]> = {
  '30000': [18_000, 12_000],
  '20000': [12_000, 8_000],
  '5000': [3_000, 2_000],
};
const splitOf = (amount: string): readonly [number, number] => SPLITS[amount.trim()] ?? [600, 400];

/**
 * The commands of gardens as a healthy backend answers them, in memory. Every method answers at once
 * or throws the refusal (a `CommandError`); the test wrapper turns that into a promise, and the
 * development page's stand-in does the same.
 */
export function createGardenBooks(options: GardensOptions = {}) {
  const store: GardenStore = {
    gardens: options.gardens ?? [e40Garden()],
    history: options.history ?? e40History(),
    cashBefore: options.cashBefore ?? 50_000,
    saved: { gardens: [], entries: [], closings: [], cancels: [], reopens: [] },
    refuse: {},
  };
  const defaults: GardenDefaults = options.defaults ?? { partners: firmPartners() };
  const gardenOf = (id: number | null): GardenRow =>
    store.gardens.find((row) => row.id === id) ??
    refuse([{ code: 'ERR_NOT_FOUND', field: 'garden_id' }]);

  const entryNumbers = (balanceBefore: number, balanceAfter: number, rentDue: number) => ({
    balance_before_paisa: money(balanceBefore),
    balance_after_paisa: money(balanceAfter),
    rent_due_after_paisa: money(rentDue),
    balance_working: balanceWorking(0, 0, balanceAfter),
    // Only a sale in another shop linked to goods arrived has these (PRD FR-12.41).
    qty_from_our_shop: null,
    qty_left_after: null,
    qty_other_left_after: null,
    qty_our_left_after: null,
  });

  return {
    store,
    defaults: (): GardenDefaults => defaults,

    createPreview: (input: GardenInput): GardenCreatePreview => {
      const errors: FieldError[] = [];
      if (input.name_ur === '') errors.push(problem('ERR_REQUIRED', 'name_ur'));
      if (input.name_en === '') errors.push(problem('ERR_REQUIRED', 'name_en'));
      if (input.rent === '') errors.push(problem('ERR_REQUIRED', 'rent'));
      // The reason of the first installment is required once its amount is typed (plan 17c).
      const paidTyped = input.paid_now !== null && input.paid_now !== '';
      if (paidTyped && (input.paid_now_reason ?? '').trim() === '') {
        errors.push(problem('ERR_REASON_REQUIRED', 'paid_now_reason'));
      }
      const typed = input.partners.filter((row) => row.share.trim() !== '0');
      const total = typed.reduce((sum, row) => sum + Number(row.share), 0);
      if (total !== 100) errors.push(problem('ERR_SHARES_NOT_100', 'partners'));
      const sources = sourcesOf(input.partners);
      const paid = input.paid_now !== null && input.paid_now !== '';
      const { view, shop } = partsView(
        paymentParts(30_000, [18_000, 12_000], sources),
        [18_000, 12_000],
      );
      return {
        first_installment: paid ? view : null,
        cash: cashOf(store.cashBefore, paid ? shop : 0),
        errors,
        warnings: [],
      };
    },
    create: (input: GardenInput): GardenCreated => {
      if (store.refuse.garden !== undefined) return refuse(store.refuse.garden);
      if (
        input.paid_now !== null &&
        input.paid_now !== '' &&
        (input.paid_now_reason ?? '').trim() === ''
      ) {
        return refuse([{ code: 'ERR_REASON_REQUIRED', field: 'paid_now_reason' }]);
      }
      const sources = sourcesOf(input.partners);
      const paid = input.paid_now !== null && input.paid_now !== '';
      const { shop } = partsView(paymentParts(30_000, [18_000, 12_000], sources), [18_000, 12_000]);
      if (paid && cashOf(store.cashBefore, shop).needs_confirm && !input.confirm_cash_below_zero) {
        return refuse([
          { code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', field: 'confirm_cash_below_zero' },
        ]);
      }
      store.saved.gardens.push(input);
      return {
        row: gardenRow({ id: 2, number: 2, name_ur: input.name_ur, name_en: input.name_en }),
        first_entry_id: paid ? 21 : null,
        journal_entry_id: paid ? 41 : null,
        preview: {
          first_installment: null,
          cash: cashOf(store.cashBefore, 0),
          errors: [],
          warnings: [],
        },
      };
    },

    list: (input: GardensListInput): GardensList => {
      let rows = store.gardens;
      if (input.status !== null) rows = rows.filter((row) => row.status === input.status);
      if (input.search !== null) {
        const needle = input.search.toLowerCase();
        rows = rows.filter(
          (row) =>
            row.name_en.toLowerCase().includes(needle) || row.name_ur.includes(input.search ?? ''),
        );
      }
      const limit = input.limit === 0 ? 100 : input.limit;
      return {
        rows: rows.slice(input.offset, input.offset + limit),
        total_count: rows.length,
        totals: {
          rent_paisa: money(50_000),
          paid_paisa: money(55_000),
          money_in_paisa: money(66_400),
          balance_paisa: money(11_400),
          rent_due_paisa: money(0),
        },
      };
    },
    get: (input: GardenGetInput): GardenDetail => {
      const row = store.gardens.find((garden) => garden.id === input.id);
      if (row === undefined) return refuse([{ code: 'ERR_NOT_FOUND', field: 'id' }]);
      const limit = input.history_limit === 0 ? 100 : input.history_limit;
      return {
        row,
        history: store.history.slice(input.history_offset, input.history_offset + limit),
        history_count: store.history.length,
        partner_results: e40Results(row.status === 'closed'),
      };
    },
    summary: (): GardensSummary => ({
      open_gardens: 1,
      balance_total_paisa: money(11_400),
      rows: [
        {
          id: GARDEN_ID,
          number: 1,
          code: '0080',
          name_ur: 'شاہ پور باغ',
          name_en: 'Shahpur Garden',
          balance_paisa: money(11_400),
          rent_due_paisa: money(0),
        },
      ],
    }),

    entryPreview: (input: GardenEntryInput): GardenEntryPreview => {
      const garden = gardenOf(input.garden_id);
      if (input.type === 'other_sale') {
        if (input.qty === '' || input.amount === '') {
          const errors = [
            ...(input.qty === '' ? [problem('ERR_REQUIRED', 'qty')] : []),
            ...(input.amount === '' ? [problem('ERR_REQUIRED', 'amount')] : []),
          ];
          return {
            kind: 'other_sale',
            numbers: null,
            cash: cashOf(store.cashBefore, 0),
            errors,
            warnings: [],
          };
        }
        return {
          kind: 'other_sale',
          numbers: {
            amount: money(10_000),
            parts: null,
            ...entryNumbers(1_400, 11_400, 0),
          },
          cash: cashOf(store.cashBefore, 0),
          errors: [],
          warnings: [],
        };
      }
      if (input.amount === '' || input.reason === '') {
        const errors = [
          ...(input.amount === '' ? [problem('ERR_REQUIRED', 'amount')] : []),
          ...(input.reason === '' ? [problem('ERR_REASON_REQUIRED', 'reason')] : []),
        ];
        return {
          kind: input.type,
          numbers: null,
          cash: cashOf(store.cashBefore, 0),
          errors,
          warnings: [],
        };
      }
      if (garden.status === 'closed') {
        return {
          kind: input.type,
          numbers: null,
          cash: cashOf(store.cashBefore, 0),
          errors: [problem('ERR_GARDEN_CLOSED', 'garden_id')],
          warnings: [],
        };
      }
      const split = splitOf(input.amount);
      const [a, b] = sourcesOf(input.parts);
      const total = split[0] + split[1];
      const { view, shop } = partsView(paymentParts(total, split, [a, b]), split);
      return {
        kind: input.type,
        numbers: {
          amount: money(total),
          parts: view,
          ...entryNumbers(0, -total, 50_000 - total),
        },
        cash: cashOf(store.cashBefore, shop),
        errors: [],
        warnings: [],
      };
    },
    entrySave: (input: GardenEntryInput): GardenEntrySaved => {
      if (store.refuse.entry !== undefined) return refuse(store.refuse.entry);
      if (input.type !== 'other_sale') {
        const split = splitOf(input.amount);
        const [a, b] = sourcesOf(input.parts);
        const { shop } = partsView(paymentParts(split[0] + split[1], split, [a, b]), split);
        if (cashOf(store.cashBefore, shop).needs_confirm && !input.confirm_cash_below_zero) {
          return refuse([
            { code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', field: 'confirm_cash_below_zero' },
          ]);
        }
      }
      store.saved.entries.push(input);
      return {
        id: 16,
        number: 1,
        version: 1,
        journal_entry_id: 37,
        preview: {
          kind: input.type,
          numbers: null,
          cash: cashOf(store.cashBefore, 0),
          errors: [],
          warnings: [],
        },
      };
    },
    entryCancel: (input: GardenEntryCancelInput): GardenEntryCancelled => {
      if (store.refuse.cancel !== undefined) return refuse(store.refuse.cancel);
      store.saved.cancels.push(input);
      return { id: input.id, number: 1, reversal_entry_id: 38 };
    },

    closePreview: (input: GardenCloseInput): GardenClosePreview => {
      const garden = gardenOf(input.id);
      if (garden.status === 'closed') {
        return {
          numbers: null,
          earliest_date: null,
          errors: [problem('ERR_GARDEN_CLOSED', 'id')],
          warnings: [],
        };
      }
      return {
        numbers: {
          balance_paisa: money(66_400),
          total_cost_paisa: money(55_000),
          money_in_paisa: money(66_400),
          result_paisa: money(11_400),
          credits: closingCredits(),
          partner_results: e40Results(true),
          posts_entry: true,
        },
        earliest_date: '2026-10-12',
        errors: [],
        warnings: [],
      };
    },
    close: (input: GardenCloseInput): GardenClosed => {
      if (store.refuse.close !== undefined) return refuse(store.refuse.close);
      store.saved.closings.push(input);
      return {
        row: e40Closed(),
        close_entry_id: 15,
        journal_entry_id: 36,
        preview: {
          numbers: null,
          earliest_date: null,
          errors: [],
          warnings: [],
        },
      };
    },
    reopen: (input: GardenReopenInput): GardenReopened => {
      if (store.refuse.reopen !== undefined) return refuse(store.refuse.reopen);
      store.saved.reopens.push(input);
      return { row: e40Garden({ version: 3 }), reversal_entry_id: 39 };
    },
  };
}
