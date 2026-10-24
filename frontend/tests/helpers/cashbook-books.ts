// The books behind the cash book fakes (plan 17), in plain code with no test framework, so the
// component tests (cashbook.ts) and the development page /dev/rokar (dev-bridge.ts) answer the
// commands of the روکڑ area from the same numbers. It keeps the PRD worked day after Events 1 to 4
// in rupees (cash 53,484 before the expenses of Event 5; partner totals الف 2,196, ب 1,646, ج 1,646
// before the electricity bill, PRD 4.8 and E2E-39). The sums are a test double's: the screen only
// shows what comes back and never works out an amount (CLAUDE.md rule 1).

import type { Balance } from '../../src/bindings/Balance';
import type { CashCheck } from '../../src/bindings/CashCheck';
import type { Cashbook } from '../../src/bindings/Cashbook';
import type { CashbookInput } from '../../src/bindings/CashbookInput';
import type { CashbookRow } from '../../src/bindings/CashbookRow';
import type { ExpenseEntry } from '../../src/bindings/ExpenseEntry';
import type { ExpensePreview } from '../../src/bindings/ExpensePreview';
import type { FieldError } from '../../src/bindings/FieldError';
import type { KhataAccount } from '../../src/bindings/KhataAccount';
import type { KhataEntryRow } from '../../src/bindings/KhataEntryRow';
import type { KhataStatement } from '../../src/bindings/KhataStatement';
import type { KhataStatementInput } from '../../src/bindings/KhataStatementInput';
import type { LentList } from '../../src/bindings/LentList';
import type { LentListInput } from '../../src/bindings/LentListInput';
import type { LentPerson } from '../../src/bindings/LentPerson';
import type { LentPersonCreateInput } from '../../src/bindings/LentPersonCreateInput';
import type { LentRow } from '../../src/bindings/LentRow';
import type { PartnerTakes } from '../../src/bindings/PartnerTakes';
import type { PartnerTakesInput } from '../../src/bindings/PartnerTakesInput';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { RepeatLastMonth } from '../../src/bindings/RepeatLastMonth';
import type { TakeRow } from '../../src/bindings/TakeRow';
import type { VoucherEntry } from '../../src/bindings/VoucherEntry';
import type { VoucherPreview } from '../../src/bindings/VoucherPreview';
import type { VoucherRow } from '../../src/bindings/VoucherRow';
import type { VoucherSaved } from '../../src/bindings/VoucherSaved';
import type { VoucherType } from '../../src/bindings/VoucherType';
import type { VouchersListInput } from '../../src/bindings/VouchersListInput';
import type { VouchersList } from '../../src/bindings/VouchersList';
import type { Working } from '../../src/bindings/Working';
import type { ExpenseHeads } from '../../src/bindings/ExpenseHeads';
import type { ExpenseHeadUse } from '../../src/bindings/ExpenseHeadUse';
import { CommandError } from '../../src/shared/api';
import { paisaForTest } from './paisa';
import { refusal } from './ui-error';
import { voucherRow } from './voucher-books';
import { sliceOf } from './paged';

export const TODAY_ISO = '2026-10-10';

const money = (rupees: number) => paisaForTest(rupees * 100);
const ZERO: Balance = { amount_paisa: money(0), side: 'zero' };
const balanceOf = (rupees: number): Balance =>
  rupees === 0 ? ZERO : { amount_paisa: money(Math.abs(rupees)), side: rupees > 0 ? 'dr' : 'cr' };
const numberOf = (text: string): number => Number(text.replaceAll(',', '') || '0');
// Like the real backend, the stand-in never works with a word as a number: it counts it as 0 here
// and refuses it with ERR_NOT_A_NUMBER at the field (PRD FR-7.20, Code Standards 4.2).
const isNumber = (text: string): boolean => !Number.isNaN(numberOf(text));
const rupeesOf = (text: string): number => (isNumber(text) ? numberOf(text) : 0);
const isoOf = (text: string): string => text.split('-').reverse().join('-');

/** The partners of PRD 4.2, with the ids the partners fakes use. */
export const PARTNERS: readonly (RecordRef & { bps: number })[] = [
  { id: 1, code: '1', name_ur: 'الف', name_en: 'Partner A', bps: 4000 },
  { id: 2, code: '2', name_ur: 'ب', name_en: 'Partner B', bps: 3000 },
  { id: 3, code: '3', name_ur: 'ج', name_en: 'Partner C', bps: 3000 },
];

/** The expense heads, the most used first (PRD FR-7.62). */
export interface HeadRec extends RecordRef {
  account_id: number;
}

export const HEADS: readonly HeadRec[] = [
  head(1, '05010001', 'بجلی', 'Electricity'),
  head(2, '05010002', 'دکان کرایہ', 'Shop Rent'),
  head(3, '05010003', 'چائے', 'Tea'),
  head(4, '05010004', 'تنخواہ', 'Salaries'),
];

function head(id: number, code: string, ur: string, en: string): HeadRec {
  return { id, code, name_ur: ur, name_en: en, account_id: 500 + id };
}

/** A head the way `masters_expense_heads` answers: the names are in `head` (the real shape). */
export function headChoice(rec: HeadRec, useCount = 0): ExpenseHeadUse {
  return {
    id: rec.id,
    head: { id: rec.id, code: rec.code, name_ur: rec.name_ur, name_en: rec.name_en },
    account_id: rec.account_id,
    use_count: useCount,
    last_used_date: null,
  };
}

/** A person who can borrow (control 0103) and a khata (control 0305). */
export const ASLAM: RecordRef = { id: 61, code: '01030001', name_ur: 'اسلم', name_en: 'Aslam' };
export const FAMILY_KHATA: RecordRef = {
  id: 71,
  code: '03050001',
  name_ur: 'خاندانی کھاتہ',
  name_en: 'Family Khata',
};

/** A cash movement in rupees (the book keeps plain numbers; Paisa is only built for the answers). */
export interface CashRec {
  date: string;
  type: VoucherType;
  number: number;
  narration: string;
  inRupees: number;
  outRupees: number;
  running: number;
}

/** Money taken by a partner, or lent to a person: the same shape (PRD FR-16.2, FR-16.5). */
export interface LoanRec {
  voucher_id: number;
  number: number;
  date: string;
  amount: number;
  reason: string;
  takenBy: string;
  returnBy: string | null;
  returned: number;
}

/** An entry of a khata in rupees; `signed` is minus when money was paid out. */
export interface KhataRec {
  kind: 'paid_out' | 'received';
  entry_id: number;
  voucher_id: number;
  number: number;
  date: string;
  reason: string;
  narration: string;
  signed: number;
  running: number;
}

/** A proposal of "repeat last month", in rupees. */
export interface RepeatRec {
  head: HeadRec;
  amount: number;
  paid_to: string;
  narration: string;
  last_date: string;
}

export interface CashbookBooksOptions {
  /** Cash in hand at the start, in rupees. */
  cash?: number;
  /** The cash book rows already there, oldest first. */
  cashRows?: CashRec[];
}

export interface CashbookBooks {
  /** Cash in hand now, in rupees. */
  cash: number;
  /** Each partner's کل رقم now, in rupees (signed). */
  totals: Map<number, number>;
  /** What was saved, in order, as sent. */
  saves: { type: VoucherType; entry: ExpenseEntry | VoucherEntry }[];
  /** The saved shop expenses, with their amounts in rupees. */
  vouchers: { row: VoucherRow; amount: number }[];
  cashRows: CashRec[];
  takes: Map<number, LoanRec[]>;
  loans: Map<number, LoanRec[]>;
  khataRows: Map<number, KhataRec[]>;
  persons: LentPerson[];
  khatas: KhataAccount[];
  /** The proposals of "repeat last month". */
  repeat: RepeatRec[];
}

function refused(code: string, field: string | null, values: Record<string, string | number> = {}) {
  return new CommandError(refusal([{ code, field, values }]));
}

/** Every problem together, as the backend answers (Code Standards 4.3). */
function refusedAll(errors: readonly FieldError[]) {
  return new CommandError(refusal(errors.map((item) => ({ ...item }))));
}

function workingOf(result: number, base: number, bps: number): Working {
  // The shape and message id the backend really sends (plan 17 fixture vouchers_preview_expense_shop).
  return {
    message_id: 'working-expense-share',
    args: [
      { name: 'amount', value: { kind: 'money', paisa: money(base) } },
      { name: 'bps', value: { kind: 'percent', bps } },
      { name: 'result', value: { kind: 'money', paisa: money(result) } },
    ],
    rule_id: null,
  };
}

function totalWorking(before: number, amount: number, after: number): Working {
  return {
    message_id: 'working-partner-total',
    args: [
      { name: 'before', value: { kind: 'money', paisa: money(before) } },
      { name: 'amount', value: { kind: 'money', paisa: money(amount) } },
      { name: 'after', value: { kind: 'money', paisa: money(after) } },
    ],
    rule_id: null,
  };
}

/** Each partner's part of a shop expense (C11): rounded down, the leftover to the largest share. */
export function splitShares(amount: number): number[] {
  const parts = PARTNERS.map((partner) => Math.floor((amount * partner.bps) / 10_000));
  const left = amount - parts.reduce((sum, part) => sum + part, 0);
  parts[0] = (parts[0] ?? 0) + left;
  return parts;
}

const CASH_OUT: ReadonlySet<string> = new Set([
  'shop_expense',
  'personal_expense',
  'money_lent',
  'khata_out',
]);

export function createCashbookBooks(options: CashbookBooksOptions = {}) {
  const store: CashbookBooks = {
    cash: options.cash ?? 53_484,
    totals: new Map([
      [1, 2196],
      [2, 1646],
      [3, 1646],
    ]),
    saves: [],
    vouchers: [],
    cashRows: [...(options.cashRows ?? [])],
    takes: new Map(),
    loans: new Map(),
    khataRows: new Map(),
    persons: [{ account: ASLAM, staff: null, phone: '' }],
    khatas: [{ account: FAMILY_KHATA, note: '', balance_paisa: money(0) }],
    repeat: [],
  };
  const counters = new Map<string, number>();
  const nextNumber = (type: string) => {
    const next = (counters.get(type) ?? 0) + 1;
    counters.set(type, next);
    return next;
  };
  const partnerOf = (id: number | null) => PARTNERS.find((partner) => partner.id === id) ?? null;
  const openSum = (rows: readonly LoanRec[]) =>
    rows.reduce((sum, row) => sum + row.amount - row.returned, 0);
  const openOf = (partnerId: number) => openSum(store.takes.get(partnerId) ?? []);
  const loanOpen = (personId: number) => openSum(store.loans.get(personId) ?? []);
  const khataBalance = (khataId: number) => {
    const rows = store.khataRows.get(khataId) ?? [];
    return rows[rows.length - 1]?.running ?? 0;
  };
  /** Returns settle the oldest rows first (PRD FR-16.2, FR-16.5). */
  const settle = (rows: LoanRec[], amount: number) => {
    let left = amount;
    for (const row of rows) {
      const used = Math.min(row.amount - row.returned, left);
      row.returned += used;
      left -= used;
    }
  };

  function cashCheck(type: string, amount: number): CashCheck {
    const out = CASH_OUT.has(type);
    const change = out ? -amount : amount;
    const after = store.cash + change;
    return {
      cash_before_paisa: money(store.cash),
      change_paisa: money(change),
      cash_after_paisa: money(after),
      below_zero: after < 0,
      lowers_cash: out && amount > 0,
      needs_confirm: out && after < 0,
    };
  }

  function amountErrors(amountText: string, dateText: string): FieldError[] {
    const errors: FieldError[] = [];
    if (!isNumber(amountText)) {
      errors.push({ code: 'ERR_NOT_A_NUMBER', field: 'amount', values: {} });
    } else if (amountText.includes('.')) {
      errors.push({ code: 'ERR_DECIMALS_NOT_ALLOWED', field: 'amount', values: {} });
    } else if (rupeesOf(amountText) <= 0) {
      errors.push({ code: 'ERR_MUST_BE_MORE_THAN_ZERO', field: 'amount', values: {} });
    }
    if (dateText.trim() === '') errors.push({ code: 'ERR_REQUIRED', field: 'date', values: {} });
    return errors;
  }

  function expenseErrors(entry: ExpenseEntry): FieldError[] {
    const errors = amountErrors(entry.amount, entry.date);
    if (entry.type === 'shop_expense') {
      if (entry.expense_head_id === null) {
        errors.push({ code: 'ERR_EXPENSE_HEAD_REQUIRED', field: 'expense_head_id', values: {} });
      }
      return errors;
    }
    if (entry.partner_id === null) {
      errors.push({ code: 'ERR_PARTNER_REQUIRED', field: 'partner_id', values: {} });
    }
    if (entry.type === 'personal_expense') {
      if (entry.reason.trim() === '') {
        errors.push({ code: 'ERR_REASON_REQUIRED', field: 'reason', values: {} });
      }
      if (entry.return_by !== null && isoOf(entry.return_by) < isoOf(entry.date)) {
        errors.push({ code: 'ERR_RETURN_BY_BEFORE_ENTRY', field: 'return_by', values: {} });
      }
    } else if (entry.partner_id !== null) {
      const open = openOf(entry.partner_id);
      if (rupeesOf(entry.amount) > open) {
        errors.push({
          code: 'ERR_RETURN_EXCEEDS_OPEN',
          field: 'amount',
          values: { open_paisa: open * 100 },
        });
      }
    }
    return errors;
  }

  function expensePreview(entry: ExpenseEntry): ExpensePreview {
    const amount = rupeesOf(entry.amount);
    const errors = expenseErrors(entry);
    const partner = entry.type === 'shop_expense' ? null : partnerOf(entry.partner_id);
    const headRow =
      entry.type === 'shop_expense'
        ? (HEADS.find((candidate) => candidate.id === entry.expense_head_id) ?? null)
        : null;
    // FR-7.68: the text typed as "paid to" is a partner's name in either script.
    const paidTo = entry.type === 'shop_expense' ? (entry.paid_to ?? '').trim().toLowerCase() : '';
    const paidToPartner =
      paidTo === ''
        ? null
        : (PARTNERS.find(
            (candidate) =>
              candidate.name_en.toLowerCase() === paidTo || candidate.name_ur === paidTo,
          ) ?? null);
    const shares =
      entry.type === 'shop_expense' && amount > 0
        ? splitShares(amount).map((part, index) => {
            const one = PARTNERS[index];
            if (one === undefined) throw new Error('no partner');
            return {
              partner: { id: one.id, code: one.code, name_ur: one.name_ur, name_en: one.name_en },
              share_bps: one.bps,
              amount_paisa: money(part),
              working: workingOf(part, amount, one.bps),
            };
          })
        : [];
    const before = partner === null ? 0 : (store.totals.get(partner.id) ?? 0);
    const open = partner === null ? 0 : openOf(partner.id);
    const effect =
      partner === null || entry.type === 'shop_expense'
        ? null
        : {
            partner: {
              id: partner.id,
              code: partner.code,
              name_ur: partner.name_ur,
              name_en: partner.name_en,
            },
            total_before_paisa: money(before),
            total_after_paisa: money(
              entry.type === 'personal_expense' ? before - amount : before + amount,
            ),
            open_before_paisa: money(open),
            open_after_paisa: money(
              entry.type === 'personal_expense' ? open + amount : open - amount,
            ),
            working: totalWorking(
              before,
              amount,
              entry.type === 'personal_expense' ? before - amount : before + amount,
            ),
          };
    return {
      voucher_type: entry.type,
      amount_paisa: money(amount),
      head:
        headRow === null
          ? null
          : {
              id: headRow.id,
              code: headRow.code,
              name_ur: headRow.name_ur,
              name_en: headRow.name_en,
            },
      partner:
        partner === null
          ? null
          : {
              id: partner.id,
              code: partner.code,
              name_ur: partner.name_ur,
              name_en: partner.name_en,
            },
      shares,
      partner_effect: effect,
      others_not_affected: effect !== null,
      paid_to_partner:
        paidToPartner === null
          ? null
          : {
              id: paidToPartner.id,
              code: paidToPartner.code,
              name_ur: paidToPartner.name_ur,
              name_en: paidToPartner.name_en,
            },
      cash: cashCheck(entry.type, amount),
      errors,
      warnings: [],
    };
  }

  function saved(type: VoucherType, balance: Balance = ZERO): VoucherSaved {
    const number = nextNumber(type);
    return {
      voucher_id: 700 + store.saves.length,
      voucher_type: type,
      number,
      entry_id: 1700 + store.saves.length,
      new_balance: balance,
    };
  }

  function moveCash(type: VoucherType, amount: number, confirmed: boolean) {
    const after = store.cash + (CASH_OUT.has(type) ? -amount : amount);
    if (after < 0 && CASH_OUT.has(type) && !confirmed) {
      throw refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', null, { cash_after_paisa: after * 100 });
    }
    store.cash = after;
  }

  function saveExpense(entry: ExpenseEntry): VoucherSaved {
    const errors = expenseErrors(entry);
    if (errors.length > 0) throw refusedAll(errors);
    const amount = rupeesOf(entry.amount);
    const confirmed = entry.type !== 'personal_return' && entry.confirm_cash_below_zero;
    moveCash(entry.type, amount, confirmed);
    const result = saved(entry.type);
    store.saves.push({ type: entry.type, entry });
    if (entry.type === 'shop_expense') {
      const shares = splitShares(amount);
      PARTNERS.forEach((partner, index) => {
        store.totals.set(partner.id, (store.totals.get(partner.id) ?? 0) - (shares[index] ?? 0));
      });
      const headRow = HEADS.find((candidate) => candidate.id === entry.expense_head_id);
      store.vouchers.push({
        amount,
        row: voucherRow({
          id: result.voucher_id,
          voucher_type: 'shop_expense',
          number: result.number,
          entry_date: isoOf(entry.date),
          account:
            headRow === undefined
              ? null
              : {
                  id: headRow.account_id,
                  code: headRow.code,
                  name_ur: headRow.name_ur,
                  name_en: headRow.name_en,
                },
          amount_paisa: money(amount),
          narration: entry.narration ?? '',
        }),
      });
    } else if (entry.partner_id !== null) {
      const sign = entry.type === 'personal_expense' ? -1 : 1;
      store.totals.set(entry.partner_id, (store.totals.get(entry.partner_id) ?? 0) + sign * amount);
      const rows = store.takes.get(entry.partner_id) ?? [];
      if (entry.type === 'personal_expense') {
        rows.push({
          voucher_id: result.voucher_id,
          number: result.number,
          date: isoOf(entry.date),
          amount,
          reason: entry.reason,
          takenBy: entry.taken_by ?? '',
          returnBy: entry.return_by === null ? null : isoOf(entry.return_by),
          returned: 0,
        });
      } else {
        settle(rows, amount);
      }
      store.takes.set(entry.partner_id, rows);
    }
    pushCashRow(entry.type, result.number, entry.date, amount, entry.narration ?? '');
    return result;
  }

  function pushCashRow(
    type: VoucherType,
    number: number,
    date: string,
    amount: number,
    note: string,
  ) {
    const out = CASH_OUT.has(type);
    store.cashRows.push({
      date: isoOf(date),
      type,
      number,
      narration: note,
      inRupees: out ? 0 : amount,
      outRupees: out ? amount : 0,
      running: store.cash,
    });
  }

  // ---- ادھار رقم and الگ کھاتہ through the generic voucher preview and the type saves.

  function entryPreview(entry: VoucherEntry): VoucherPreview {
    if (entry.type === 'money_lent' || entry.type === 'money_lent_return') {
      const amount = rupeesOf(entry.amount);
      const errors = amountErrors(entry.amount, entry.date);
      if (entry.person_account_id === null) {
        errors.push({ code: 'ERR_LENT_PERSON_REQUIRED', field: 'person_account_id', values: {} });
      }
      if (entry.type === 'money_lent') {
        if (entry.reason.trim() === '') {
          errors.push({ code: 'ERR_REASON_REQUIRED', field: 'reason', values: {} });
        }
        if (entry.return_by !== null && isoOf(entry.return_by) < isoOf(entry.date)) {
          errors.push({ code: 'ERR_RETURN_BY_BEFORE_ENTRY', field: 'return_by', values: {} });
        }
      } else if (entry.person_account_id !== null) {
        const open = loanOpen(entry.person_account_id);
        if (amount > open) {
          errors.push({
            code: 'ERR_RETURN_EXCEEDS_OPEN',
            field: 'amount',
            values: { open_paisa: open * 100 },
          });
        }
      }
      const open = entry.person_account_id === null ? 0 : loanOpen(entry.person_account_id);
      const cashMoves = entry.type === 'money_lent' || entry.mode === 'cash';
      return previewOf(
        store.persons.find((person) => person.account.id === entry.person_account_id)?.account ??
          null,
        amount,
        balanceOf(open),
        balanceOf(entry.type === 'money_lent' ? open + amount : open - amount),
        cashMoves ? cashCheck(entry.type, amount) : cashCheck('none', 0),
        errors,
      );
    }
    if (entry.type === 'khata_out' || entry.type === 'khata_in') {
      const amount = rupeesOf(entry.amount);
      const errors = amountErrors(entry.amount, entry.date);
      if (entry.khata_account_id === null) {
        errors.push({ code: 'ERR_KHATA_REQUIRED', field: 'khata_account_id', values: {} });
      }
      if (entry.reason.trim() === '') {
        errors.push({ code: 'ERR_REASON_REQUIRED', field: 'reason', values: {} });
      }
      const plus = entry.khata_account_id === null ? 0 : khataBalance(entry.khata_account_id);
      const after = entry.type === 'khata_out' ? plus - amount : plus + amount;
      // A khata's plus is a credit balance and its minus a debit balance (PRD 16.6).
      const asBalance = (value: number): Balance =>
        value === 0
          ? ZERO
          : { amount_paisa: money(Math.abs(value)), side: value > 0 ? 'cr' : 'dr' };
      return previewOf(
        store.khatas.find((khata) => khata.account.id === entry.khata_account_id)?.account ?? null,
        amount,
        asBalance(plus),
        asBalance(after),
        cashCheck(entry.type, amount),
        errors,
      );
    }
    throw refused('ERR_NOT_IMPLEMENTED', null);
  }

  function previewOf(
    account: RecordRef | null,
    amount: number,
    now: Balance,
    after: Balance,
    cash: CashCheck,
    errors: FieldError[],
  ): VoucherPreview {
    return {
      account,
      amount_paisa: money(amount),
      current_balance: now,
      new_balance: after,
      bill_effects: [],
      extra_credit_paisa: money(0),
      advance_effects: [],
      owes_settled_paisa: money(0),
      return_limit_paisa: null,
      payable_open_paisa: null,
      payable_is_zero_for_seller: false,
      cash,
      errors,
      warnings: [],
    };
  }

  function saveLentOrKhata(entry: VoucherEntry): VoucherSaved {
    const answer = entryPreview(entry);
    if (answer.errors.length > 0) throw refusedAll(answer.errors);
    const amount = rupeesOf('amount' in entry ? entry.amount : '0');
    const confirmed = 'confirm_cash_below_zero' in entry && entry.confirm_cash_below_zero;
    const moves =
      entry.type === 'money_lent' ||
      entry.type === 'khata_out' ||
      entry.type === 'khata_in' ||
      (entry.type === 'money_lent_return' && entry.mode === 'cash');
    if (moves)
      moveCash(
        entry.type,
        amount,
        entry.type === 'money_lent' || entry.type === 'khata_out' ? confirmed : true,
      );
    const result = saved(entry.type);
    store.saves.push({ type: entry.type, entry });
    if (entry.type === 'money_lent' && entry.person_account_id !== null) {
      const rows = store.loans.get(entry.person_account_id) ?? [];
      rows.push({
        voucher_id: result.voucher_id,
        number: result.number,
        date: isoOf(entry.date),
        amount,
        reason: entry.reason,
        takenBy: '',
        returnBy: entry.return_by === null ? null : isoOf(entry.return_by),
        returned: 0,
      });
      store.loans.set(entry.person_account_id, rows);
    } else if (entry.type === 'money_lent_return' && entry.person_account_id !== null) {
      settle(store.loans.get(entry.person_account_id) ?? [], amount);
    } else if (
      (entry.type === 'khata_out' || entry.type === 'khata_in') &&
      entry.khata_account_id !== null
    ) {
      const rows = store.khataRows.get(entry.khata_account_id) ?? [];
      const signed = entry.type === 'khata_out' ? -amount : amount;
      rows.push({
        kind: entry.type === 'khata_out' ? 'paid_out' : 'received',
        entry_id: result.entry_id,
        voucher_id: result.voucher_id,
        number: result.number,
        date: isoOf(entry.date),
        reason: entry.reason,
        narration: entry.narration ?? '',
        signed,
        running: khataBalance(entry.khata_account_id) + signed,
      });
      store.khataRows.set(entry.khata_account_id, rows);
    }
    pushCashRow(entry.type, result.number, entry.date, amount, '');
    return result;
  }

  function page<T>(rows: readonly T[], offset: number, limit: number): T[] {
    const size = limit === 0 ? 100 : limit;
    return rows.slice(offset, offset + size);
  }

  function cashbook(input: CashbookInput): Cashbook {
    const from = input.from === null || input.from === '' ? null : isoOf(input.from);
    const to = input.to === null || input.to === '' ? null : isoOf(input.to);
    const inRange = store.cashRows.filter(
      (row) => (from === null || row.date >= from) && (to === null || row.date <= to),
    );
    const first = inRange[0];
    const last = inRange[inRange.length - 1];
    const totalIn = inRange.reduce((total, row) => total + row.inRupees, 0);
    const totalOut = inRange.reduce((total, row) => total + row.outRupees, 0);
    const closing = last === undefined ? store.cash : last.running;
    const opening = first === undefined ? closing : closing - totalIn + totalOut;
    const rows: CashbookRow[] = page(inRange, input.offset, input.limit).map((row, index) => ({
      entry_id: 1700 + input.offset + index,
      entry_date: row.date,
      posting_type: row.type,
      number: row.number,
      account: null,
      narration: row.narration,
      in_paisa: money(row.inRupees),
      out_paisa: money(row.outRupees),
      running_paisa: money(row.running),
    }));
    return {
      opening_paisa: money(opening),
      closing_paisa: money(closing),
      rows,
      total_count: inRange.length,
      totals: { in_paisa: money(totalIn), out_paisa: money(totalOut) },
    };
  }

  function vouchersList(input: VouchersListInput): VouchersList {
    const found = store.vouchers.filter(
      ({ row }) =>
        (input.voucher_type === null || row.voucher_type === input.voucher_type) &&
        row.entry_date >= isoOf(input.from) &&
        row.entry_date <= isoOf(input.to),
    );
    const total = found
      .filter(({ row }) => row.status !== 'cancelled')
      .reduce((sum, one) => sum + one.amount, 0);
    const all = found.map(({ row }) => row);
    return {
      rows: sliceOf(all, input.offset, input.limit),
      total_count: all.length,
      totals: {
        cancelled_count: all.filter((row) => row.status === 'cancelled').length,
        total_paisa: found.length === 0 ? null : money(total),
      },
    };
  }

  const loanRow = (row: LoanRec): LentRow => ({
    voucher_id: row.voucher_id,
    number: row.number,
    entry_date: row.date,
    amount_paisa: money(row.amount),
    reason: row.reason,
    return_by: row.returnBy,
    returned_paisa: money(row.returned),
    open_paisa: money(row.amount - row.returned),
    days_past_return_by: null,
  });

  function partnerTakes(input: PartnerTakesInput): PartnerTakes {
    const partner = partnerOf(input.partner_id);
    if (partner === null) throw refused('ERR_NOT_FOUND', 'partner_id');
    const rows = store.takes.get(partner.id) ?? [];
    const takeRows: TakeRow[] = page(rows, input.offset, input.limit).map((row) => ({
      source: 'voucher',
      voucher_id: row.voucher_id,
      number: row.number,
      entry_date: row.date,
      amount_paisa: money(row.amount),
      reason: row.reason,
      taken_by: row.takenBy,
      return_by: row.returnBy,
      returned_paisa: money(row.returned),
      open_paisa: money(row.amount - row.returned),
      days_past_return_by: null,
    }));
    return {
      partner: {
        id: partner.id,
        code: partner.code,
        name_ur: partner.name_ur,
        name_en: partner.name_en,
      },
      rows: takeRows,
      total_count: rows.length,
      totals: {
        taken_paisa: money(rows.reduce((total, row) => total + row.amount, 0)),
        returned_paisa: money(rows.reduce((total, row) => total + row.returned, 0)),
        open_paisa: money(openSum(rows)),
      },
    };
  }

  function lentList(input: LentListInput): LentList {
    const person = store.persons.find((one) => one.account.id === input.person_id);
    if (person === undefined) throw refused('ERR_NOT_FOUND', 'person_id');
    const rows = store.loans.get(person.account.id) ?? [];
    return {
      person,
      balance: balanceOf(loanOpen(person.account.id)),
      rows: page(rows, input.offset, input.limit).map(loanRow),
      total_count: rows.length,
      totals: {
        lent_paisa: money(rows.reduce((total, row) => total + row.amount, 0)),
        returned_paisa: money(rows.reduce((total, row) => total + row.returned, 0)),
        open_paisa: money(openSum(rows)),
      },
    };
  }

  function khataStatement(input: KhataStatementInput): KhataStatement {
    const khata = store.khatas.find((one) => one.account.id === input.khata_account_id);
    if (khata === undefined) throw refused('ERR_NOT_FOUND', 'khata_account_id');
    const rows = store.khataRows.get(khata.account.id) ?? [];
    const sum = (wanted: 'paid_out' | 'received') =>
      rows
        .filter((row) => row.kind === wanted)
        .reduce((total, row) => total + Math.abs(row.signed), 0);
    const entryRows: KhataEntryRow[] = page(rows, input.offset, input.limit).map((row) => ({
      kind: row.kind,
      entry_id: row.entry_id,
      voucher_id: row.voucher_id,
      number: row.number,
      entry_date: row.date,
      reason: row.reason,
      narration: row.narration,
      amount_paisa: money(row.signed),
      running_paisa: money(row.running),
    }));
    return {
      khata: { ...khata, balance_paisa: money(khataBalance(khata.account.id)) },
      opening_paisa: money(0),
      closing_paisa: money(khataBalance(khata.account.id)),
      rows: entryRows,
      total_count: rows.length,
      totals: { paid_out_paisa: money(sum('paid_out')), received_paisa: money(sum('received')) },
    };
  }

  return {
    store,
    cashbook,
    heads: (): ExpenseHeads => ({ rows: HEADS.map((rec) => headChoice(rec)) }),
    expensePreview,
    entryPreview,
    saveExpense,
    saveLentOrKhata,
    vouchersList,
    partnerTakes,
    lentList,
    khataStatement,
    repeat: (month: string): RepeatLastMonth => ({
      month,
      last_month: '2026-09',
      rows: store.repeat.map((row) => ({
        head: {
          id: row.head.id,
          code: row.head.code,
          name_ur: row.head.name_ur,
          name_en: row.head.name_en,
        },
        expense_head_id: row.head.id,
        amount_paisa: money(row.amount),
        paid_to: row.paid_to,
        narration: row.narration,
        last_date: row.last_date,
      })),
      total_paisa: money(store.repeat.reduce((sum, row) => sum + row.amount, 0)),
    }),
    createPerson: (input: LentPersonCreateInput): LentPerson => {
      const person: LentPerson = {
        account: {
          id: 62 + store.persons.length,
          code: `0103000${String(store.persons.length + 1)}`,
          name_ur: input.name_ur,
          name_en: input.name_en,
        },
        staff: null,
        phone: input.phone ?? '',
      };
      store.persons.push(person);
      return person;
    },
    createKhata: (input: {
      name_ur: string;
      name_en: string;
      note: string | null;
    }): KhataAccount => {
      const khata: KhataAccount = {
        account: {
          id: 72 + store.khatas.length,
          code: `0305000${String(store.khatas.length + 1)}`,
          name_ur: input.name_ur,
          name_en: input.name_en,
        },
        note: input.note ?? '',
        balance_paisa: money(0),
      };
      store.khatas.push(khata);
      return khata;
    },
  };
}

export type CashbookFakeBooks = ReturnType<typeof createCashbookBooks>;
