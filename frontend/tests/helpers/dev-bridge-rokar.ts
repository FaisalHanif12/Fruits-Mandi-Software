// The answers of the development page /dev/rokar (plan 17): the commands of the روکڑ area, from
// the books of cashbook-books.ts, plus the few commands around them (the partners, the controls,
// the account search, the party card). dev-bridge.ts asks this table first while the address is
// /dev/rokar. Flags in the hash: `?screen=book|expense|lent|khata|general|capital|statement|overview|day`, `&type=personal_expense` ...,
// `&empty` (nothing yet), `&cash` (cash 1,000, so a 3,000 entry asks the cash question),
// `&repeat` (two regular expenses of last month), `&entries` (shop expenses of a few days), `&takes` (two partners took money and one gave some back, plan 98).
// Never part of a build: only the development route imports it.

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
import type { LentPersonCreateRequest } from '../../src/bindings/LentPersonCreateRequest';
import type { Cashbook } from '../../src/bindings/Cashbook';
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
import type { PartnersStatementDayInput } from '../../src/bindings/PartnersStatementDayInput';
import type { PartnersStatementInput } from '../../src/bindings/PartnersStatementInput';
import type { PartnersStatementMonthInput } from '../../src/bindings/PartnersStatementMonthInput';
import { CASHBOOK_UMAR_REAL } from './cashbook-real';
import type { CashRec } from './cashbook-books';
import type { OpeningsToCashPreview } from '../../src/bindings/OpeningsToCashPreview';
import { nothingToMove } from './cash-groups';
import { devFlag } from './dev-bridge';
import { plan76Handlers } from './dev-bridge-plan76';
import { createFruitBooks, HAJI_NAZIR } from './fruit-data';
import { account as accountRow } from './accounts';
import { GENERAL_ACCOUNTS, createPartnerMoneyBooks } from './partner-money-books';
import { dayAnswerOf, monthAnswerOf } from './partner-month';
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

const who = (id: number, code: string, name_ur: string, name_en: string) => ({
  id,
  code,
  name_ur,
  name_en,
});
const named = (id: number, code: string, ur: string, en: string) => ({
  parties: [who(id, code, ur, en)],
  party_count: 1,
});

/**
 * The worked day's cash as PRD FR-16.14 names its rows: بکری 1 نقد, بکری 2 paid on the spot, وصولی 1,
 * ادائیگی 1, دکان خرچہ 1 and ذاتی خرچہ 1; with the day's opening 50,000 it closes at 58,484. `&named`
 * adds a buyer's پیشگی, a garden cost and an old and a reversing line (the switch shows them).
 */
const DAY_ROWS: CashRec[] = [
  {
    date: '2026-10-10',
    type: 'payment',
    number: 1,
    narration: '',
    kind: 'bikri_seller_cash',
    refs: named(10, '0010', 'رحیم', 'Rahim'),
    inRupees: 0,
    outRupees: 5000,
    running: 45000,
  },
  {
    date: '2026-10-10',
    type: 'receipt',
    number: 2,
    narration: '',
    kind: 'bikri_paid',
    refs: named(30, '0030', 'شاہد', 'Shahid'),
    inRupees: 12800,
    outRupees: 0,
    running: 57800,
  },
  {
    date: '2026-10-10',
    type: 'receipt',
    number: 1,
    narration: '',
    refs: named(20, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax'),
    inRupees: 10000,
    outRupees: 0,
    running: 67800,
  },
  {
    date: '2026-10-10',
    type: 'payment',
    number: 1,
    narration: '',
    refs: named(10, '0010', 'رحیم', 'Rahim'),
    inRupees: 0,
    outRupees: 4316,
    running: 63484,
  },
  {
    date: '2026-10-10',
    type: 'shop_expense',
    number: 1,
    narration: '',
    refs: { expense_head: who(501, '05010001', 'بجلی', 'Electricity') },
    inRupees: 0,
    outRupees: 3000,
    running: 60484,
  },
  {
    date: '2026-10-10',
    type: 'personal_expense',
    number: 1,
    narration: 'گھر کا خرچ',
    refs: { partner: who(1, '', 'الف', 'Partner A') },
    inRupees: 0,
    outRupees: 2000,
    running: 58484,
  },
];

const NAMED_ROWS: CashRec[] = [
  {
    date: '2026-10-10',
    type: 'payment',
    number: 2,
    narration: '',
    kind: 'buyer_advance',
    refs: named(24, '0024', 'مانی', 'Mani'),
    inRupees: 0,
    outRupees: 5000,
    running: 53484,
  },
  {
    date: '2026-10-10',
    type: 'payment',
    number: 1,
    narration: '',
    kind: 'garden_cost',
    refs: { garden: who(8, '0080', 'پومپ والا باغ', 'Pump Garden') },
    inRupees: 0,
    outRupees: 4000,
    running: 49484,
  },
  {
    date: '2026-10-10',
    type: 'receipt',
    number: 3,
    narration: '',
    refs: named(21, '0021', 'ارشد ملتانی', 'Arshad Multani'),
    status: 'reversed',
    inRupees: 700,
    outRupees: 0,
    running: 50184,
  },
  {
    date: '2026-10-10',
    type: 'receipt',
    number: 3,
    narration: '',
    refs: named(21, '0021', 'ارشد ملتانی', 'Arshad Multani'),
    status: 'reversal',
    inRupees: 0,
    outRupees: 700,
    running: 49484,
  },
];

function headAt(index: number): HeadRec {
  const found = HEADS[index];
  if (found === undefined) throw new Error('no such head');
  return found;
}

const books = createCashbookBooks({
  cash: flag('cash') ? 1000 : 58484,
  cashRows: flag('empty') ? [] : [...DAY_ROWS, ...(flag('named') ? NAMED_ROWS : [])],
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
// `&entries`: shop expenses of a few days, so the 30-day list under the form (plan 100 F) can be looked at with data.
if (flag('entries')) {
  const expense = (date: string, amount: string, head: number, narration: string) =>
    books.saveExpense({
      type: 'shop_expense',
      date,
      expense_head_id: headAt(head).id,
      amount,
      paid_to: null,
      narration,
      confirm_cash_below_zero: true,
    });
  expense('22-09-2026', '1200', 1, '');
  expense('05-10-2026', '700', 3, '');
  expense('10-10-2026', '3000', 0, 'ستمبر کا بل');
}
// `&takes`: two partners have taken money, so the list under the خرچہ form (plan 98) can be looked at with data.
if (flag('takes')) {
  const take = (partner_id: number, amount: string, reason: string) =>
    books.saveExpense({
      type: 'personal_expense',
      date: '10-10-2026',
      partner_id,
      amount,
      reason,
      return_by: null,
      taken_by: null,
      narration: null,
      confirm_cash_below_zero: true,
    });
  take(1, '2000', 'گھر کا خرچ');
  take(2, '500', 'شادی');
  books.saveExpense({
    type: 'personal_return',
    date: '10-10-2026',
    partner_id: 1,
    amount: '700',
    narration: null,
  });
}

const moneyBooks = createPartnerMoneyBooks(books);

/** The overview cards of E2E-37: the partners with their plan 17 columns. */
const OVERVIEW = (() => {
  const [a, b, c] = THREE_PARTNERS;
  if (a === undefined || b === undefined || c === undefined) throw new Error('no partners');
  return [
    overviewRow(a, {
      open_gardens_count: 1,
      open_trips_count: 1,
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
  partners_locked: false,
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

interface Person {
  id: number;
  code: string;
  name_ur: string;
  name_en: string;
  type: 'money_lent' | 'other' | 'seller';
  control: number;
  /** A seller account marked آڑھتی دکان: the only kind the آڑھتی box of the پھل tab lists (PRD FR-16.11 v1.59). */
  agentShop?: boolean;
}

const PEOPLE: Person[] = [
  { ...ASLAM, type: 'money_lent', control: 103 },
  { ...FAMILY_KHATA, type: 'other', control: 305 },
  // The other commission agent of the پھل tab: a seller account marked آڑھتی دکان (plan 17d, PRD FR-16.11).
  { ...HAJI_NAZIR, type: 'seller', control: 11, agentShop: true },
];
let nextShopId = 900;

const asNumber = (value: unknown): number => (typeof value === 'number' ? value : 0);
const asText = (value: unknown): string => (typeof value === 'string' ? value : '');

function hitOf(person: Person) {
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
    deduction: null,
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
    deduction: null,
    note: null,
    editing_id: null,
  });
}

/** Plan 99 Section 4: openings بنام 125,000 out of cash, جمع 40,000 in; cash 1,057,840 to 972,840. */
const OLD_OPENINGS: OpeningsToCashPreview = {
  rows: [
    ['مانی', 'Mani', 24, '0024', 'buyer', 30000, 0],
    ['انس', 'Anas', 25, '0025', 'buyer', 15000, 0],
    ['عبدالرحمان', 'Abdul Rehman', 27, '0027', 'buyer', 20000, 0],
    ['عدنان', 'Adnan', 16, '0016', 'seller', 60000, 0],
    ['فیصل', 'Faisal', 17, '0017', 'buyer', 0, 10000],
    ['ہارون', 'Haroon', 26, '0026', 'buyer', 0, 10000],
    ['احسان', 'Ehsan', 18, '0018', 'seller', 0, 20000],
  ].map(([ur, en, id, code, kind, dr, cr]) => ({
    account: who(Number(id), String(code), String(ur), String(en)),
    kind: kind === 'seller' ? ('seller' as const) : ('buyer' as const),
    date: '2026-10-29',
    dr_paisa: money(Number(dr)),
    cr_paisa: money(Number(cr)),
    cash_change_paisa: money(Number(cr) - Number(dr)),
    day_closed: true,
  })),
  total_dr_paisa: money(125000),
  total_cr_paisa: money(40000),
  cash_change_paisa: money(-85000),
  cash_before_paisa: money(1057840),
  cash_after_paisa: money(972840),
  needs_cash_confirm: false,
  opening_difference_before: { amount_paisa: money(85000), side: 'cr' },
  opening_difference_after: { amount_paisa: money(0), side: 'zero' },
  reopens_from: '2026-10-29',
};

/** `&many`: the book with its lines repeated up to 14, each with its own entry id (a long list, plan 96 T2). */
const MANY_ROWS = 14;
function paddedBook(book: Cashbook): Cashbook {
  if (!devFlag('many') || book.rows.length === 0) return book;
  const rows = Array.from({ length: MANY_ROWS }, (_, index) => {
    const source = book.rows[index % book.rows.length];
    return source === undefined
      ? undefined
      : { ...source, entry_id: source.entry_id + index * 1000 };
  }).filter((row): row is NonNullable<typeof row> => row !== undefined);
  return { ...book, rows, total_count: rows.length };
}

export const rokarHandlers: Record<string, Handler> = {
  ...plan76Handlers,
  fruit_preview: (input) => fruitBooks.preview(input as unknown as FruitInput),
  fruit_save: (input) => fruitBooks.save(input as unknown as FruitInput),
  fruit_edit: (input) => fruitBooks.edit(input as unknown as FruitEditInput),
  fruit_cancel: (input) => fruitBooks.cancel(input as unknown as FruitCancelInput),
  fruit_get: (input) => fruitBooks.get(input as unknown as FruitGetInput),
  fruit_list: (input) => fruitBooks.list(input as unknown as FruitListInput),
  fruit_lots: (input) => fruitBooks.lots(input as unknown as FruitLotsInput),
  // `&many` pads the book to 14 lines, so a list of more than ten rows (the 'fill' height of a long list) can be photographed.
  // `&umar-book` is the real answer of Umar's books (11 lines, plan 96 T2).
  cashbook_get: (input) =>
    devFlag('umar-book')
      ? CASHBOOK_UMAR_REAL
      : paddedBook(books.cashbook(input as unknown as CashbookInput)),
  // The one-time fix of old openings (PRD FR-10.9): `&old-openings` has two to move, else nothing.
  accounts_openings_to_cash_preview: () =>
    devFlag('old-openings') ? OLD_OPENINGS : nothingToMove(),
  accounts_openings_to_cash: () => ({
    rows: OLD_OPENINGS.rows,
    total_dr_paisa: OLD_OPENINGS.total_dr_paisa,
    total_cr_paisa: OLD_OPENINGS.total_cr_paisa,
    cash_after_paisa: OLD_OPENINGS.cash_after_paisa,
    opening_difference_after: OLD_OPENINGS.opening_difference_after,
    reopened_from: '2026-10-29',
    closed_up_to: '2026-11-19',
  }),
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
  // The first day with data of a list, for the back button of its window (plan 76 point 8).
  day_status: () => ({
    today: '2026-10-10',
    closed_up_to: null,
    today_open: true,
    first_entry_date: '2026-09-20',
  }),
  windows_bounds: () => ({ first_date: '2026-09-20', last_date: '2026-10-10' }),
  partners_statement_month: (input) => {
    const wanted = input as unknown as PartnersStatementMonthInput;
    const partner = moneyBooks.partnerRef(wanted.partner_id);
    if (partner === null) throw new Error('no such partner');
    return monthAnswerOf(
      partner,
      moneyBooks.statementLines(wanted.partner_id),
      wanted.month === null || wanted.month === '' ? null : wanted.month,
      '2026-11-12',
    );
  },
  partners_statement_day: (input) => {
    const wanted = input as unknown as PartnersStatementDayInput;
    const partner = moneyBooks.partnerRef(wanted.partner_id);
    if (partner === null) throw new Error('no such partner');
    return dayAnswerOf(partner, moneyBooks.statementLines(wanted.partner_id), wanted.date);
  },
  partners_overview: () => OVERVIEW,
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
  lent_person_create: (input) => books.createPerson(input as unknown as LentPersonCreateRequest),
  lent_list: (input) => books.lentList(input as unknown as LentListInput),
  khata_create: (input) =>
    books.createKhata(
      input as unknown as { name_ur: string; name_en: string; note: string | null },
    ),
  khata_statement: (input) => books.khataStatement(input as unknown as KhataStatementInput),
  partners_list: () => ({
    partners: PARTNERS.map((partner) =>
      partnerRow(partner.id, partner.name_ur, partner.name_en, partner.bps),
    ),
    partners_locked: false,
  }),
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
        account_type: one.kind === 'buyer' ? 'buyer' : 'other',
        control_id: 1,
        is_active: true,
      }));
    }
    // The آڑھتی box of the پھل tab lists only the shops marked آڑھتی دکان (PRD FR-16.11 v1.59).
    if (asText(input['scope']) === 'agent_shop') {
      return PEOPLE.filter(
        (person) =>
          person.agentShop === true &&
          [person.code, person.name_ur, person.name_en].some((part) =>
            part.toLowerCase().includes(wanted),
          ),
      ).map(hitOf);
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
      asText(input['scope']) === 'agent_shop'
        ? person.agentShop === true
        : asText(input['scope']) === 'seller'
          ? person.type === 'seller'
          : asNumber(input['control_id']) === person.control,
    ).map(hitOf),
  // The short form of a new commission agent's shop (PRD FR-7.69, FR-16.11 v1.59).
  fruit_agent_shop_add: (input) => {
    nextShopId += 1;
    const person: Person = {
      id: nextShopId,
      code: `001${String(nextShopId - 880)}`,
      name_ur: asText(input['name_ur']),
      name_en: asText(input['name_en']),
      type: 'seller',
      control: 11,
      agentShop: true,
    };
    PEOPLE.push(person);
    return {
      ...accountRow(person.id, 'seller', person.code, person.name_ur, person.name_en),
      is_agent_shop: true,
    };
  },
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
            type: general.kind === 'buyer' ? 'buyer' : 'other',
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
      advance_covered: null,
      net_payable: null,
      warnings: [],
      dead: null,
    };
  },
};
