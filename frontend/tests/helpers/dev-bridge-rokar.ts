// The answers of the development page /dev/rokar (plan 17): the commands of the روکڑ area, from
// the books of cashbook-books.ts, plus the few commands around them (the partners, the controls,
// the account search, the party card). dev-bridge.ts asks this table first while the address is
// /dev/rokar. Flags in the hash: `?screen=book|expense|lent|khata|general|capital|statement|overview|day`, `&type=personal_expense` ...,
// `&empty` (nothing yet), `&cash` (cash 1,000, so a 3,000 entry asks the cash question),
// `&repeat` (two regular expenses of last month). Never part of a build: only the development
// route imports it.

import type { CashbookInput } from '../../src/bindings/CashbookInput';
import type { ExpenseEntry } from '../../src/bindings/ExpenseEntry';
import type { KhataStatementInput } from '../../src/bindings/KhataStatementInput';
import type { KhataInInput } from '../../src/bindings/KhataInInput';
import type { KhataOutInput } from '../../src/bindings/KhataOutInput';
import type { MoneyLentInput } from '../../src/bindings/MoneyLentInput';
import type { MoneyLentReturnInput } from '../../src/bindings/MoneyLentReturnInput';
import type { PersonalExpenseInput } from '../../src/bindings/PersonalExpenseInput';
import type { PersonalReturnInput } from '../../src/bindings/PersonalReturnInput';
import type { ShopExpenseInput } from '../../src/bindings/ShopExpenseInput';
import type { LentListInput } from '../../src/bindings/LentListInput';
import type { LentPersonCreateInput } from '../../src/bindings/LentPersonCreateInput';
import type { PartnerTakesInput } from '../../src/bindings/PartnerTakesInput';
import type { VoucherEntry } from '../../src/bindings/VoucherEntry';
import type { VouchersListInput } from '../../src/bindings/VouchersListInput';
import {
  ASLAM,
  FAMILY_KHATA,
  HEADS,
  PARTNERS,
  createCashbookBooks,
  type HeadRec,
} from './cashbook-books';
import type { CapitalHistoryInput } from '../../src/bindings/CapitalHistoryInput';
import type { CapitalInput } from '../../src/bindings/CapitalInput';
import type { DayCloseSummary } from '../../src/bindings/DayCloseSummary';
import type { FruitCancelInput } from '../../src/bindings/FruitCancelInput';
import type { FruitEditInput } from '../../src/bindings/FruitEditInput';
import type { FruitGetInput } from '../../src/bindings/FruitGetInput';
import type { FruitInput } from '../../src/bindings/FruitInput';
import type { FruitListInput } from '../../src/bindings/FruitListInput';
import type { FruitLotsInput } from '../../src/bindings/FruitLotsInput';
import type { GeneralInput } from '../../src/bindings/GeneralInput';
import type { GeneralPreviewInput } from '../../src/bindings/GeneralPreviewInput';
import type { PartnersStatementInput } from '../../src/bindings/PartnersStatementInput';
import { createFruitBooks, HAJI_NAZIR } from './fruit-data';
import { GENERAL_ACCOUNTS, SHAHID, createPartnerMoneyBooks } from './partner-money-books';
import { paisaForTest } from './paisa';
import { overviewRow, partnerRow, THREE_PARTNERS } from './partners';

type Input = Record<string, unknown>;
type Handler = (input: Input) => unknown;

/** True on the development page of this area. */
export function isRokarPage(): boolean {
  return window.location.hash.startsWith('#/dev/rokar');
}

const flag = (name: string): boolean =>
  new URLSearchParams(window.location.hash.split('?')[1] ?? '').has(name);

const money = (rupees: number) => paisaForTest(rupees * 100);

/** The worked day's cash (PRD 4.9): in 10,000, out 4,316 + 3,000 + 2,000. */
const DAY_ROWS = [
  { type: 'receipt', number: 1, inRupees: 10000, outRupees: 0, running: 60000 },
  { type: 'payment', number: 1, inRupees: 0, outRupees: 4316, running: 55684 },
  { type: 'shop_expense', number: 1, inRupees: 0, outRupees: 3000, running: 52684 },
  { type: 'personal_expense', number: 1, inRupees: 0, outRupees: 2000, running: 50684 },
] as const;

function headAt(index: number): HeadRec {
  const found = HEADS[index];
  if (found === undefined) throw new Error('no such head');
  return found;
}

const books = createCashbookBooks({
  cash: flag('cash') ? 1000 : 50684,
  cashRows: flag('empty')
    ? []
    : DAY_ROWS.map((row) => ({ date: '2026-10-10', narration: '', ...row })),
});
if (flag('repeat')) {
  books.store.repeat.push(
    {
      head: headAt(1),
      amount: 25000,
      paid_to: '',
      narration: '',
      last_date: '2026-09-05',
    },
    {
      head: headAt(3),
      amount: 40000,
      paid_to: '',
      narration: '',
      last_date: '2026-09-30',
    },
  );
}

const moneyBooks = createPartnerMoneyBooks(books);

/** The overview cards of E2E-37: the partners with their plan 17 columns. */
const OVERVIEW = (() => {
  const [a, b, c] = THREE_PARTNERS;
  if (a === undefined || b === undefined || c === undefined) throw new Error('no partners');
  return [
    overviewRow(a, {
      month_profit_paisa: money(996),
      month_taken_paisa: money(2000),
      month_net_paisa: money(-1004),
      total_paisa: money(-1004),
    }),
    overviewRow(b, {
      month_profit_paisa: money(746),
      capital_balance_paisa: money(50000),
      month_capital_in_paisa: money(50000),
      total_paisa: money(50746),
    }),
    overviewRow(c, {
      month_profit_paisa: money(746),
      month_net_paisa: money(746),
      total_paisa: money(746),
    }),
  ];
})();

/** PRD 4.9: the day summary of the worked day. */
const DAY_SUMMARY: DayCloseSummary = {
  bikri_count: 2,
  gross_paisa: money(41400),
  commission_paisa: money(5448),
  shared_income_paisa: money(5488),
  partners: PARTNERS.map((partner, index) => ({
    partner: { id: partner.id, code: '', name_ur: partner.name_ur, name_en: partner.name_en },
    income_paisa: money([2196, 1646, 1646][index] ?? 0),
    expense_paisa: money([1200, 900, 900][index] ?? 0),
    profit_paisa: money([996, 746, 746][index] ?? 0),
  })),
  open_lots: 0,
  open_qty: 0,
};

const PEOPLE = [
  { ...ASLAM, type: 'money_lent' as const, control: 103 },
  { ...FAMILY_KHATA, type: 'other' as const, control: 305 },
  // The other commission agent of the پھل tab: a seller account (plan 17d, PRD FR-16.11).
  { ...HAJI_NAZIR, type: 'seller' as const, control: 11 },
];

const asNumber = (value: unknown): number => (typeof value === 'number' ? value : 0);
const asText = (value: unknown): string => (typeof value === 'string' ? value : '');

function hitOf(person: (typeof PEOPLE)[number]) {
  return {
    id: person.id,
    code: person.code,
    name_ur: person.name_ur,
    name_en: person.name_en,
    extra_ur: '',
    extra_en: '',
    account_type: person.type,
    control_id: person.control,
    is_active: true,
  };
}

/**
 * The books of the پھل tab (plan 17d, tests/helpers/fruit-data.ts). Unless `&empty`, two entries of
 * E2E-42 are in it: A took 2 پیٹی کیلا home at 580, and the guests had 3 پیٹی سیب at 1,000 from
 * حاجی نذیر's shop.
 */
const fruitBooks = createFruitBooks();
if (!flag('empty')) {
  fruitBooks.save({
    for_guests: false,
    partner_id: 1,
    date: '10-10-2026',
    source: 'shop',
    lot_id: 40,
    agent_account_id: null,
    item_id: null,
    packing_id: null,
    qty: '2',
    weight: null,
    rate: '580',
    note: null,
    editing_id: null,
  });
  fruitBooks.save({
    for_guests: true,
    partner_id: null,
    date: '10-10-2026',
    source: 'other_shop',
    lot_id: null,
    agent_account_id: HAJI_NAZIR.id,
    item_id: 63,
    packing_id: 71,
    qty: '3',
    weight: null,
    rate: '1000',
    note: null,
    editing_id: null,
  });
}

export const rokarHandlers: Record<string, Handler> = {
  fruit_preview: (input) => fruitBooks.preview(input as unknown as FruitInput),
  fruit_save: (input) => fruitBooks.save(input as unknown as FruitInput),
  fruit_edit: (input) => fruitBooks.edit(input as unknown as FruitEditInput),
  fruit_cancel: (input) => fruitBooks.cancel(input as unknown as FruitCancelInput),
  fruit_get: (input) => fruitBooks.get(input as unknown as FruitGetInput),
  fruit_list: (input) => fruitBooks.list(input as unknown as FruitListInput),
  fruit_lots: (input) => fruitBooks.lots(input as unknown as FruitLotsInput),
  cashbook_get: (input) => books.cashbook(input as unknown as CashbookInput),
  masters_expense_heads: () => books.heads(),
  vouchers_preview_expense: (input) => books.expensePreview(input['entry'] as ExpenseEntry),
  vouchers_preview: (input) => {
    const entry = input['entry'] as VoucherEntry;
    return entry.type === 'capital_in' || entry.type === 'capital_out'
      ? moneyBooks.capitalPreview(entry)
      : books.entryPreview(entry);
  },
  general_preview: (input) => moneyBooks.generalPreview(input as unknown as GeneralPreviewInput),
  general_save: (input) => moneyBooks.generalSave(input as unknown as GeneralInput),
  vouchers_save_capital_in: (input) =>
    moneyBooks.saveCapital('capital_in', input as unknown as CapitalInput),
  vouchers_save_capital_out: (input) =>
    moneyBooks.saveCapital('capital_out', input as unknown as CapitalInput),
  capital_history: (input) => moneyBooks.capitalHistory(input as unknown as CapitalHistoryInput),
  partners_statement: (input) => moneyBooks.statement(input as unknown as PartnersStatementInput),
  partners_overview: () => OVERVIEW,
  search_pukhta_people: () => [SHAHID],
  day_close_preview: () => ({
    from_date: '2026-10-10',
    up_to: '2026-10-10',
    days: 1,
    trial_dr_paisa: money(85048),
    trial_cr_paisa: money(85048),
    trial_difference: { amount_paisa: money(0), side: 'zero' },
    cash_in_hand: { amount_paisa: money(58484), side: 'dr' },
    entries: 12,
    summary: DAY_SUMMARY,
  }),
  vouchers_list: (input) => books.vouchersList(input as unknown as VouchersListInput),
  vouchers_save_shop_expense: (input) =>
    books.saveExpense({ type: 'shop_expense', ...(input as unknown as ShopExpenseInput) }),
  vouchers_save_personal_expense: (input) =>
    books.saveExpense({ type: 'personal_expense', ...(input as unknown as PersonalExpenseInput) }),
  vouchers_save_personal_return: (input) =>
    books.saveExpense({ type: 'personal_return', ...(input as unknown as PersonalReturnInput) }),
  vouchers_save_money_lent: (input) =>
    books.saveLentOrKhata({ type: 'money_lent', ...(input as unknown as MoneyLentInput) }),
  vouchers_save_money_lent_return: (input) =>
    books.saveLentOrKhata({
      type: 'money_lent_return',
      ...(input as unknown as MoneyLentReturnInput),
    }),
  vouchers_save_khata_out: (input) =>
    books.saveLentOrKhata({ type: 'khata_out', ...(input as unknown as KhataOutInput) }),
  vouchers_save_khata_in: (input) =>
    books.saveLentOrKhata({ type: 'khata_in', ...(input as unknown as KhataInInput) }),
  partner_takes: (input) => books.partnerTakes(input as unknown as PartnerTakesInput),
  vouchers_repeat_last_month_preview: (input) => books.repeat(asText(input['month'])),
  lent_person_create: (input) => books.createPerson(input as unknown as LentPersonCreateInput),
  lent_list: (input) => books.lentList(input as unknown as LentListInput),
  khata_create: (input) =>
    books.createKhata(
      input as unknown as { name_ur: string; name_en: string; note: string | null },
    ),
  khata_statement: (input) => books.khataStatement(input as unknown as KhataStatementInput),
  partners_list: () =>
    PARTNERS.map((partner) =>
      partnerRow(partner.id, partner.name_ur, partner.name_en, partner.bps),
    ),
  coa_controls_list: () => [
    {
      id: 103,
      head_id: 1,
      code: '0103',
      name_ur: 'ادھار رقم',
      name_en: 'Money Lent',
      party_type: 'other',
      allowed_for_other: false,
    },
    {
      id: 305,
      head_id: 3,
      code: '0305',
      name_ur: 'الگ کھاتے',
      name_en: 'Separate Khatas',
      party_type: 'separate',
      allowed_for_other: false,
    },
  ],
  search_accounts: (input) => {
    const wanted = asText(input['text']).trim().toLowerCase();
    // The جنرل screen takes any account (scope `any`).
    if (asText(input['scope']) === 'any') {
      return GENERAL_ACCOUNTS.filter((one) =>
        [one.code, one.name_ur, one.name_en].some((part) => part.toLowerCase().includes(wanted)),
      ).map((one) => ({
        id: one.id,
        code: one.code,
        name_ur: one.name_ur,
        name_en: one.name_en,
        extra_ur: '',
        extra_en: '',
        account_type: one.kind === 'buyer' || one.kind === 'pukhta' ? 'buyer' : 'other',
        control_id: 1,
        is_active: true,
      }));
    }
    if (asText(input['scope']) === 'seller') {
      return PEOPLE.filter(
        (person) =>
          person.type === 'seller' &&
          [person.code, person.name_ur, person.name_en].some((part) =>
            part.toLowerCase().includes(wanted),
          ),
      ).map(hitOf);
    }
    return PEOPLE.filter(
      (person) =>
        asNumber(input['control_id']) === person.control &&
        [person.code, person.name_ur, person.name_en].some((part) =>
          part.toLowerCase().includes(wanted),
        ),
    ).map(hitOf);
  },
  search_accounts_recent: (input) =>
    PEOPLE.filter((person) =>
      asText(input['scope']) === 'seller'
        ? person.type === 'seller'
        : asNumber(input['control_id']) === person.control,
    ).map(hitOf),
  search_accounts_touch: () => null,
  accounts_balances: () => [],
  assist_party_card: (input) => {
    const id = asNumber(input['account_id']);
    const general = GENERAL_ACCOUNTS.find((candidate) => candidate.id === id);
    const person =
      PEOPLE.find((candidate) => candidate.id === id) ??
      (general === undefined
        ? undefined
        : {
            id: general.id,
            code: general.code,
            name_ur: general.name_ur,
            name_en: general.name_en,
            type: general.kind === 'buyer' || general.kind === 'pukhta' ? 'buyer' : 'other',
          });
    // The real command always answers for an account that exists; a stand-in has none to give.
    if (person === undefined) throw new Error('no such account');
    return {
      id: person.id,
      code: person.code,
      account_type: person.type,
      name_ur: person.name_ur,
      name_en: person.name_en,
      father_or_firm_ur: '',
      father_or_firm_en: '',
      city_ur: '',
      city_en: '',
      phone: '03001234567',
      is_active: true,
      balance: { amount_paisa: money(0), side: 'zero' },
      last_entry_date: '2026-10-09',
      credit_limit: null,
      credit_used: null,
      advance_open: null,
      warnings: [],
    };
  },
};
