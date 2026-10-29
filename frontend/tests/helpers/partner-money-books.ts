// The books behind the fakes of the second half of plan 17 (F2): the جنرل entry, partner capital,
// the partner statement and the full day close summary. Plain code with no test framework, so the
// component tests (partner-money.ts) and the development page /dev/rokar answer from the same
// numbers. It builds on the books of the cash book fakes (cashbook-books.ts): the same cash, the
// same partner totals, the same rows in the روکڑ بک. The sums are a test double's: the screens only
// show what comes back and never work out an amount (CLAUDE.md rule 1).
//
// The numbers are the PRD ones: the statement of partner ب is E2E-37 (1,184, 462, -900, then 50,000
// of capital, ending at 50,746); the statement of partner الف is the E2E-34 trip run of D4-A33
// (-4,160, +480, +3,384, ending at -296, then the two note lines).

import type { Balance } from '../../src/bindings/Balance';
import type { CapitalHistory } from '../../src/bindings/CapitalHistory';
import type { CapitalHistoryInput } from '../../src/bindings/CapitalHistoryInput';
import type { CapitalInput } from '../../src/bindings/CapitalInput';
import type { CapitalRow } from '../../src/bindings/CapitalRow';
import type { CashCheck } from '../../src/bindings/CashCheck';
import type { ExpenseShare } from '../../src/bindings/ExpenseShare';
import type { FieldError } from '../../src/bindings/FieldError';
import type { GeneralInput } from '../../src/bindings/GeneralInput';
import type { GeneralLinePreview } from '../../src/bindings/GeneralLinePreview';
import type { GeneralPreview } from '../../src/bindings/GeneralPreview';
import type { GeneralPreviewInput } from '../../src/bindings/GeneralPreviewInput';
import type { PartnerStatement } from '../../src/bindings/PartnerStatement';
import type { PartnersStatementInput } from '../../src/bindings/PartnersStatementInput';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { StatementLine } from '../../src/bindings/StatementLine';
import type { VoucherEntry } from '../../src/bindings/VoucherEntry';
import type { VoucherPreview } from '../../src/bindings/VoucherPreview';
import type { VoucherSaved } from '../../src/bindings/VoucherSaved';
import { CommandError } from '../../src/shared/api';
import { PARTNERS, splitShares, type CashbookFakeBooks } from './cashbook-books';
import { paisaForTest } from './paisa';
import { refusal } from './ui-error';

const money = (rupees: number) => paisaForTest(rupees * 100);
const ZERO: Balance = { amount_paisa: money(0), side: 'zero' };
const numberOf = (text: string): number => Number(text.replaceAll(',', '') || '0');
// Like the real backend, the stand-in never works with a word as a number: it counts it as 0 here
// and refuses it with ERR_NOT_A_NUMBER at the field (PRD FR-7.20, Code Standards 4.2).
const isNumber = (text: string): boolean => !Number.isNaN(numberOf(text));
const rupeesOf = (text: string): number => (isNumber(text) ? numberOf(text) : 0);
const isoOf = (text: string): string => text.split('-').reverse().join('-');
const balanceOf = (rupees: number): Balance =>
  rupees === 0 ? ZERO : { amount_paisa: money(Math.abs(rupees)), side: rupees > 0 ? 'cr' : 'dr' };

/** An account the جنرل screen can pick. `kind` tells the books how to treat its line. */
export interface GeneralAccount extends RecordRef {
  kind: 'cash' | 'buyer' | 'seller' | 'pukhta' | 'expense' | 'income' | 'other';
}

/** The accounts of the worked day that a جنرل correction would touch. */
export const GENERAL_ACCOUNTS: readonly GeneralAccount[] = [
  { id: 1, code: '01010001', name_ur: 'روکڑ', name_en: 'Cash', kind: 'cash' },
  { id: 21, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax', kind: 'buyer' },
  { id: 31, code: '0010', name_ur: 'رحیم', name_en: 'Rahim', kind: 'seller' },
  {
    id: 23,
    code: '0030',
    name_ur: 'پختہ کھاتہ (خریدار)',
    name_en: 'Pukhta Khata (Buyer)',
    kind: 'pukhta',
  },
  { id: 501, code: '05010001', name_ur: 'بجلی', name_en: 'Electricity', kind: 'expense' },
  {
    id: 401,
    code: '04010001',
    name_ur: 'کمیشن آمدن',
    name_en: 'Commission Income',
    kind: 'income',
  },
];

/** The one-time person of the books, for the پختہ کھاتہ line. */
export const SHAHID = {
  name_ur: 'شاہد',
  name_en: 'Shahid',
  phone: '03000000000',
  last_date: '2026-10-10',
  person_key: 'shahid|03000000000',
};

function refused(code: string, field: string | null, values: Record<string, string | number> = {}) {
  return new CommandError(refusal([{ code, field, values }]));
}

function refusedAll(errors: readonly FieldError[]) {
  return new CommandError(refusal(errors.map((item) => ({ ...item }))));
}

/** One line of a statement, in rupees. `note` lines have no amount. */
interface StatementRec {
  date: string;
  heading: StatementLine['heading'];
  detail: StatementLine['detail'];
  number: number | null;
  secondary: number | null;
  subject: RecordRef | null;
  reason: string;
  amount: number | null;
  values: Record<string, string | number>;
}

const BIJLI: RecordRef = { id: 501, code: '05010001', name_ur: 'بجلی', name_en: 'Electricity' };
const ORANGE: RecordRef = { id: 91, code: '', name_ur: 'مالٹا', name_en: 'Orange' };

/** Partner ب, E2E-37: 1,184 + 462 - 900 = 746, then 50,000 of capital on 11-10-26. */
const STATEMENT_B: StatementRec[] = [
  rec('2026-10-10', 'bikri', 'income_share', 1, null, null, '', 1184),
  rec('2026-10-10', 'bikri', 'income_share', 2, null, null, '', 462),
  rec('2026-10-10', 'expense', 'expense_share', 1, null, BIJLI, '', -900),
];

/** Partner الف, D4-A33 (the E2E-34 run): -4,160, +480, +3,384, then two note lines. */
const STATEMENT_A: StatementRec[] = [
  {
    ...rec('2026-10-10', 'trip', 'trip_paid', 1, 1, null, '', -4160),
    values: { kind: 'purchase' },
  },
  rec('2026-10-10', 'bikri', 'income_share', 1, null, null, '', 480),
  {
    ...rec('2026-10-10', 'trip', 'trip_credit', 1, 1, ORANGE, '', 3384),
    values: {
      qty: 15,
      sales_paisa: 846000,
      cost_paisa: 780000,
      result_paisa: 66000,
      share_bps: 4000,
    },
  },
  rec('2026-11-11', 'trip', 'trip_write_off', 1, null, null, '', null),
  {
    ...rec('2026-11-11', 'trip', 'trip_closed', 1, null, null, '', null),
    values: {
      cost_paisa: 1040000,
      sales_paisa: 846000,
      result_paisa: -194000,
      share_bps: 4000,
      net_paisa: -77600,
    },
  },
];

function rec(
  date: string,
  heading: StatementLine['heading'],
  detail: StatementLine['detail'],
  number: number | null,
  secondary: number | null,
  subject: RecordRef | null,
  reason: string,
  amount: number | null,
): StatementRec {
  return {
    date,
    heading,
    detail,
    number,
    secondary,
    subject,
    reason,
    amount,
    values: {},
  };
}

export interface PartnerMoneyOptions {
  /** The accounts the جنرل screen knows, besides the standard ones. */
  accounts?: readonly GeneralAccount[];
}

export function createPartnerMoneyBooks(
  cash: CashbookFakeBooks,
  options: PartnerMoneyOptions = {},
) {
  const accounts = options.accounts ?? GENERAL_ACCOUNTS;
  const store = {
    /** Capital history rows by partner, in rupees. */
    capital: new Map<number, { row: CapitalRow; amount: number }[]>(),
    /** Statement lines by partner. */
    statements: new Map<number, StatementRec[]>([
      [1, [...STATEMENT_A]],
      [2, [...STATEMENT_B]],
    ]),
    /** What was saved, as sent. */
    capitalSaves: [] as { type: 'capital_in' | 'capital_out'; input: CapitalInput }[],
    generalSaves: [] as GeneralInput[],
    /** The number of the next voucher of each series. */
    numbers: new Map<string, number>(),
  };
  const nextNumber = (series: string) => {
    const next = (store.numbers.get(series) ?? 0) + 1;
    store.numbers.set(series, next);
    return next;
  };
  const partnerOf = (id: number | null) => PARTNERS.find((partner) => partner.id === id) ?? null;
  const refOf = (partner: (typeof PARTNERS)[number]): RecordRef => ({
    id: partner.id,
    code: partner.code,
    name_ur: partner.name_ur,
    name_en: partner.name_en,
  });
  const capitalBalance = (partnerId: number) =>
    (store.capital.get(partnerId) ?? []).reduce((total, one) => total + one.amount, 0);
  const accountOf = (id: number | null) => accounts.find((one) => one.id === id) ?? null;

  function cashCheck(out: boolean, amount: number): CashCheck {
    const change = out ? -amount : amount;
    const after = cash.store.cash + change;
    return {
      cash_before_paisa: money(cash.store.cash),
      change_paisa: money(change),
      cash_after_paisa: money(after),
      below_zero: after < 0,
      lowers_cash: out && amount > 0,
      needs_confirm: out && after < 0,
    };
  }

  // ---- Capital (PRD 17.4).

  function capitalErrors(entry: CapitalInput): FieldError[] {
    const errors: FieldError[] = [];
    if (entry.partner_id === null) {
      errors.push({ code: 'ERR_PARTNER_REQUIRED', field: 'partner_id', values: {} });
    }
    if (!isNumber(entry.amount)) {
      errors.push({ code: 'ERR_NOT_A_NUMBER', field: 'amount', values: {} });
    } else if (entry.amount.includes('.')) {
      errors.push({ code: 'ERR_DECIMALS_NOT_ALLOWED', field: 'amount', values: {} });
    } else if (rupeesOf(entry.amount) <= 0) {
      errors.push({ code: 'ERR_MUST_BE_MORE_THAN_ZERO', field: 'amount', values: {} });
    }
    if (entry.date.trim() === '') errors.push({ code: 'ERR_REQUIRED', field: 'date', values: {} });
    return errors;
  }

  function capitalPreview(entry: Extract<VoucherEntry, { type: 'capital_in' | 'capital_out' }>) {
    const amount = rupeesOf(entry.amount);
    const errors = capitalErrors(entry);
    const partner = partnerOf(entry.partner_id);
    const now = entry.partner_id === null ? 0 : capitalBalance(entry.partner_id);
    const after = entry.type === 'capital_in' ? now + amount : now - amount;
    // FR-17.8: taking back more than the balance only warns.
    const warnings: FieldError[] =
      entry.type === 'capital_out' && amount > now && entry.partner_id !== null
        ? [
            {
              code: 'ERR_CAPITAL_EXCEEDS_BALANCE',
              field: 'amount',
              values: { balance_paisa: now * 100, over_paisa: (amount - now) * 100 },
            },
          ]
        : [];
    const answer: VoucherPreview = {
      account: partner === null ? null : refOf(partner),
      amount_paisa: money(amount),
      current_balance: balanceOf(now),
      new_balance: balanceOf(after),
      bill_effects: [],
      extra_credit_paisa: money(0),
      advance_effects: [],
      owes_settled_paisa: money(0),
      return_limit_paisa: null,
      payable_open_paisa: null,
      payable_is_zero_for_seller: false,
      cash: cashCheck(entry.type === 'capital_out', amount),
      errors,
      warnings,
    };
    return answer;
  }

  function saveCapital(type: 'capital_in' | 'capital_out', input: CapitalInput): VoucherSaved {
    const errors = capitalErrors(input);
    if (errors.length > 0) throw refusedAll(errors);
    const amount = rupeesOf(input.amount);
    const out = type === 'capital_out';
    const after = cash.store.cash + (out ? -amount : amount);
    if (out && after < 0 && !input.confirm_cash_below_zero) {
      throw refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', null, { cash_after_paisa: after * 100 });
    }
    cash.store.cash = after;
    const partnerId = input.partner_id ?? 0;
    const number = nextNumber(type);
    const signed = out ? -amount : amount;
    const rows = store.capital.get(partnerId) ?? [];
    rows.push({
      amount: signed,
      row: {
        kind: out ? 'taken_back' : 'invested',
        voucher_id: 800 + store.capitalSaves.length,
        number,
        entry_date: isoOf(input.date),
        amount_paisa: money(signed),
        narration: input.narration ?? '',
        running_paisa: money(capitalBalance(partnerId) + signed),
      },
    });
    store.capital.set(partnerId, rows);
    store.capitalSaves.push({ type, input });
    cash.store.cashRows.push({
      date: isoOf(input.date),
      type,
      number,
      narration: input.narration ?? '',
      inRupees: out ? 0 : amount,
      outRupees: out ? amount : 0,
      running: after,
    });
    // His کل رقم and his statement follow (C22): capital is part of it.
    cash.store.totals.set(partnerId, (cash.store.totals.get(partnerId) ?? 0) + signed);
    const lines = store.statements.get(partnerId) ?? [];
    lines.push({
      ...rec(
        isoOf(input.date),
        'capital',
        out ? 'capital_out' : 'capital_in',
        number,
        null,
        null,
        '',
        signed,
      ),
    });
    store.statements.set(partnerId, lines);
    return {
      voucher_id: 800 + store.capitalSaves.length - 1,
      voucher_type: type,
      number,
      entry_id: 1800 + store.capitalSaves.length,
      new_balance: balanceOf(capitalBalance(partnerId)),
    };
  }

  function capitalHistory(input: CapitalHistoryInput): CapitalHistory {
    const partner = partnerOf(input.partner_id);
    if (partner === null) throw refused('ERR_NOT_FOUND', 'partner_id');
    const rows = store.capital.get(partner.id) ?? [];
    const size = input.limit === 0 ? 100 : input.limit;
    const invested = rows.filter((one) => one.amount > 0).reduce((t, one) => t + one.amount, 0);
    const taken = rows.filter((one) => one.amount < 0).reduce((t, one) => t - one.amount, 0);
    return {
      partner: refOf(partner),
      balance_paisa: money(capitalBalance(partner.id)),
      rows: rows.slice(input.offset, input.offset + size).map((one) => one.row),
      total_count: rows.length,
      totals: { invested_paisa: money(invested), taken_back_paisa: money(taken) },
      months: [],
    };
  }

  // ---- The partner statement (PRD FR-17.9, FR-17.10).

  function statement(input: PartnersStatementInput): PartnerStatement {
    const partner = partnerOf(input.partner_id);
    if (partner === null) throw refused('ERR_NOT_FOUND', 'partner_id');
    const from = input.from === null || input.from === '' ? null : isoOf(input.from);
    const to = input.to === null || input.to === '' ? null : isoOf(input.to);
    const all = store.statements.get(partner.id) ?? [];
    let running = 0;
    let opening = 0;
    const inRange: { rec: StatementRec; running: number }[] = [];
    for (const line of all) {
      running += line.amount ?? 0;
      if (from !== null && line.date < from) {
        opening = running;
        continue;
      }
      if (to !== null && line.date > to) continue;
      inRange.push({ rec: line, running });
    }
    const size = input.limit === 0 ? 100 : input.limit;
    const lines: StatementLine[] = inRange
      .slice(input.offset, input.offset + size)
      .map(({ rec: one, running: total }, index) => ({
        entry_date: one.date,
        heading: one.heading,
        detail: one.detail,
        entry_id: one.amount === null ? null : 1900 + input.offset + index,
        number: one.number,
        // The garden and trip heading lines show their code (plan 34): type 008 or 007.
        venture_code:
          (one.heading === 'garden' || one.heading === 'trip') && one.number !== null
            ? `${one.heading === 'garden' ? '008' : '007'}${String(one.number - 1)}`
            : null,
        secondary_number: one.secondary,
        subject: one.subject,
        packing: null,
        reason: one.reason,
        amount_paisa: one.amount === null ? null : money(one.amount),
        running_paisa: money(total),
        values: one.values,
      }));
    const plus = inRange.reduce((t, { rec: one }) => t + Math.max(one.amount ?? 0, 0), 0);
    const minus = inRange.reduce((t, { rec: one }) => t + Math.max(-(one.amount ?? 0), 0), 0);
    const closing = inRange[inRange.length - 1]?.running ?? opening;
    return {
      partner: { ...refOf(partner), code: '' },
      opening_total_paisa: money(opening),
      closing_total_paisa: money(closing),
      lines,
      total_count: inRange.length,
      totals: { plus_paisa: money(plus), minus_paisa: money(minus) },
      months: [],
    };
  }

  // ---- The جنرل entry (PRD 16.4).

  function lineErrors(input: GeneralPreviewInput['lines'][number], index: number): FieldError[] {
    const path = (field: string) => `lines[${String(index)}].${field}`;
    const errors: FieldError[] = [];
    const account = accountOf(input.account_id);
    if (input.account_id === null) {
      errors.push({ code: 'ERR_REQUIRED', field: path('account_id'), values: {} });
    } else if (account === null) {
      errors.push({ code: 'ERR_NOT_FOUND', field: path('account_id'), values: {} });
    } else if (account.kind === 'pukhta' && input.person_key === null) {
      errors.push({ code: 'ERR_REQUIRED', field: path('person_key'), values: {} });
    }
    const dr = rupeesOf(input.dr);
    const cr = rupeesOf(input.cr);
    for (const side of ['dr', 'cr'] as const) {
      if (!isNumber(input[side])) {
        errors.push({ code: 'ERR_NOT_A_NUMBER', field: path(side), values: {} });
      }
    }
    if (input.dr.includes('.')) {
      errors.push({ code: 'ERR_DECIMALS_NOT_ALLOWED', field: path('dr'), values: {} });
    }
    if (dr > 0 && cr > 0) {
      errors.push({ code: 'ERR_FORMAT_INVALID', field: path('cr'), values: {} });
    } else if (dr === 0 && cr === 0 && isNumber(input.dr) && isNumber(input.cr)) {
      errors.push({ code: 'ERR_REQUIRED', field: path('dr'), values: {} });
    }
    return errors;
  }

  /** What the lines do to cash in hand, in rupees: a credit on روکڑ takes it out. */
  function cashChangeOf(lines: GeneralPreviewInput['lines']): number {
    return lines.reduce((t, line) => {
      if (accountOf(line.account_id)?.kind !== 'cash') return t;
      return t + rupeesOf(line.dr) - rupeesOf(line.cr);
    }, 0);
  }

  function generalPreview(input: GeneralPreviewInput): GeneralPreview {
    const lines: GeneralLinePreview[] = input.lines.map((line, index) => {
      const account = accountOf(line.account_id);
      return {
        account:
          account === null
            ? null
            : {
                id: account.id,
                code: account.code,
                name_ur: account.name_ur,
                name_en: account.name_en,
              },
        dr_paisa: money(rupeesOf(line.dr)),
        cr_paisa: money(rupeesOf(line.cr)),
        balance_after: null,
        errors: lineErrors(line, index),
      };
    });
    const totalDr = input.lines.reduce((t, line) => t + rupeesOf(line.dr), 0);
    const totalCr = input.lines.reduce((t, line) => t + rupeesOf(line.cr), 0);
    const errors: FieldError[] = [];
    if (input.lines.length < 2) {
      errors.push({ code: 'ERR_GENERAL_TOO_FEW_LINES', field: 'lines', values: {} });
    } else if (totalDr !== totalCr) {
      errors.push({
        code: 'ERR_GENERAL_NOT_BALANCED',
        field: 'lines',
        values: { dr_paisa: totalDr * 100, cr_paisa: totalCr * 100 },
      });
    }
    // PRD FR-6.5: a line on a shop expense account is split by the shares of the day.
    const expenseTotal = input.lines.reduce((t, line) => {
      const kind = accountOf(line.account_id)?.kind;
      return kind === 'expense' ? t + rupeesOf(line.dr) - rupeesOf(line.cr) : t;
    }, 0);
    const split: ExpenseShare[] =
      input.date !== null && expenseTotal > 0
        ? splitShares(expenseTotal).map((part, index) => {
            const one = PARTNERS[index];
            if (one === undefined) throw new Error('no partner');
            return {
              partner: refOf(one),
              share_bps: one.bps,
              amount_paisa: money(part),
              working: { message_id: 'working-partner-share', args: [], rule_id: null },
            };
          })
        : [];
    // A credit on روکڑ takes cash out (PRD FR-6.7).
    const cashChange = cashChangeOf(input.lines);
    const out = cashChange < 0;
    const hasLineError = lines.some((line) => line.errors.length > 0);
    return {
      lines,
      total_dr_paisa: money(totalDr),
      total_cr_paisa: money(totalCr),
      difference_paisa: money(totalDr - totalCr),
      is_balanced: totalDr === totalCr && totalDr > 0,
      can_save: errors.length === 0 && !hasLineError && totalDr > 0,
      split,
      cash: cashCheck(out, Math.abs(cashChange)),
      errors,
      warnings: [],
    };
  }

  function generalSave(input: GeneralInput): VoucherSaved {
    const answer = generalPreview({ date: input.date, lines: input.lines });
    const errors = [...answer.errors, ...answer.lines.flatMap((line) => line.errors)];
    if (errors.length > 0) throw refusedAll(errors);
    const change = cashChangeOf(input.lines);
    if (answer.cash.needs_confirm && !input.confirm_cash_below_zero) {
      throw refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', null, {
        cash_after_paisa: (cash.store.cash + change) * 100,
      });
    }
    cash.store.cash += change;
    store.generalSaves.push(input);
    const number = nextNumber('general');
    return {
      voucher_id: 900 + store.generalSaves.length,
      voucher_type: 'general',
      number,
      entry_id: 2000 + store.generalSaves.length,
      new_balance: ZERO,
    };
  }

  return {
    store,
    capitalPreview,
    saveCapital,
    capitalHistory,
    statement,
    generalPreview,
    generalSave,
    accounts,
  };
}

export type PartnerMoneyBooks = ReturnType<typeof createPartnerMoneyBooks>;
