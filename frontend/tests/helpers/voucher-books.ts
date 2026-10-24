// The books behind the voucher fakes (plans 10 and 15), in plain code with no test framework, so the
// component tests (vouchers.ts) and the development page /dev/vouchers (dev-bridge.ts) answer the
// voucher commands from the same numbers. It keeps the PRD worked day after Events 1 and 2 in
// rupees: Akram owes 12,880 (one open bill, Event 1), Arshad owes 18,684, Rahim is owed 4,316
// (24,316 less the 5,000 cash and the 15,000 advance, already adjusted), Karim is owed 10,160, and
// the cash in hand is 50,000. The sums are a test double's: the screen only shows what comes back
// and never works out an amount (CLAUDE.md rule 1).

import type { Balance } from '../../src/bindings/Balance';
import type { CashCheck } from '../../src/bindings/CashCheck';
import type { ErrorCode } from '../../src/bindings/ErrorCode';
import type { FieldError } from '../../src/bindings/FieldError';
import type { OpenAdvance } from '../../src/bindings/OpenAdvance';
import type { OpenBill } from '../../src/bindings/OpenBill';
import type { OpenBills } from '../../src/bindings/OpenBills';
import type { OpenPayables } from '../../src/bindings/OpenPayables';
import type { PayablePosting } from '../../src/bindings/PayablePosting';
import type { RecordRef } from '../../src/bindings/RecordRef';
import type { VoucherCancelled } from '../../src/bindings/VoucherCancelled';
import type { VouchersListInput } from '../../src/bindings/VouchersListInput';
import { CommandError } from '../../src/shared/api';
import { formatPaisa } from '../../src/shared/format';
import type { BuiltVoucher } from '../../src/features/vouchers/draft';
import type { ScreenType } from '../../src/features/vouchers/screen-type';
import type {
  AdvancesInfo,
  VoucherEntry,
  ReceiptRowResult,
  VoucherPreview,
  VoucherRow,
  VoucherSaved,
  VouchersList,
} from '../../src/features/vouchers/types';
import { paisaForTest } from './paisa';
import { refusal } from './ui-error';
import { sliceOf } from './paged';

export const TODAY_ISO = '2026-10-10';

const money = (rupees: number) => paisaForTest(rupees * 100);
const ZERO: Balance = { amount_paisa: money(0), side: 'zero' };
const balance = (rupees: number, side: Balance['side']): Balance => ({
  amount_paisa: money(rupees),
  side,
});

const rupeesOf = (text: string): number => Number(text.replaceAll(',', '') || '0');

/** `DD-MM-YYYY` as `YYYY-MM-DD`. */
const isoOf = (text: string): string => text.split('-').reverse().join('-');

/** The parties of the worked day, by id. */
export interface Party extends RecordRef {
  type: 'buyer' | 'seller' | 'other';
}

export const PARTIES: readonly Party[] = [
  { id: 21, code: '03040001', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax', type: 'buyer' },
  { id: 22, code: '03040002', name_ur: 'ارشد ملتانی', name_en: 'Arshad Multani', type: 'buyer' },
  {
    id: 23,
    code: '03049999',
    name_ur: 'پختہ کھاتہ (خریدار)',
    name_en: 'Pukhta Khata (Buyer)',
    type: 'buyer',
  },
  { id: 31, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim', type: 'seller' },
  { id: 32, code: '030110002', name_ur: 'کریم', name_en: 'Karim', type: 'seller' },
  { id: 41, code: '02010002', name_ur: 'کرایہ کھاتہ', name_en: 'Freight Payable', type: 'other' },
];

const partyOf = (id: number): Party | undefined => PARTIES.find((party) => party.id === id);
export const refOf = (id: number): RecordRef | null => {
  const party = partyOf(id);
  return party === undefined
    ? null
    : { id: party.id, code: party.code, name_ur: party.name_ur, name_en: party.name_en };
};

const bill = (lineId: number, date: string, number: number, amount: number): OpenBill => ({
  line_id: lineId,
  entry_date: date,
  source: 'bikri',
  number,
  amount_paisa: money(amount),
  open_paisa: money(amount),
  days: 0,
});

/** What the books keep: the books in rupees, and every voucher saved. */
export interface VoucherBooks {
  /** Cash in hand in rupees. */
  cash: number;
  /** Open bills of a buyer, oldest first. */
  bills: Map<
    number,
    { lineId: number; date: string; number: number; amount: number; open: number }[]
  >;
  /** Payable of a seller in rupees. */
  payable: Map<number, number>;
  /** Open advances of a seller in rupees. */
  advances: Map<number, number>;
  /** The vouchers saved so far, in order. */
  rows: VoucherRow[];
  /** What was sent to each command. */
  saves: BuiltVoucher[];
  edits: { id: number; version: number; entry: BookEntry; reason: string; confirm: boolean }[];
  cancels: { id: number; version: number; reason: string; confirm: boolean }[];
  /** The advances saved so far: the shape the first advance tests read. */
  saved: { seller: number; amount: string }[];
}

export interface BooksOptions {
  cash?: number;
  /** Karim has nothing payable (for the FR-7.68 note). */
  karimPayable?: number;
}

function cashCheck(cash: number, change: number): CashCheck {
  const after = cash + change;
  return {
    cash_before_paisa: money(cash),
    change_paisa: money(change),
    cash_after_paisa: money(after),
    below_zero: after < 0,
    lowers_cash: change < 0,
    needs_confirm: after < 0 && change < 0,
  };
}

/** The three kinds these books know. A kind of the روکڑ area (plan 17) has its own books. */
type BookEntry = Extract<VoucherEntry, { type: 'receipt' | 'payment' | 'advance' }>;

function bookEntry(entry: VoucherEntry): BookEntry {
  if (entry.type === 'receipt' || entry.type === 'payment' || entry.type === 'advance') {
    return entry;
  }
  throw refused('ERR_NOT_IMPLEMENTED', null);
}

function refused(code: string, field: string | null, values: Record<string, string | number> = {}) {
  return new CommandError(refusal([{ code, field, values }]));
}

export function createBooks(options: BooksOptions = {}) {
  const store: VoucherBooks = {
    cash: options.cash ?? 50000,
    bills: new Map([
      [21, [{ lineId: 101, date: TODAY_ISO, number: 1, amount: 12880, open: 12880 }]],
      [22, [{ lineId: 102, date: TODAY_ISO, number: 1, amount: 18684, open: 18684 }]],
    ]),
    payable: new Map([
      [31, 4316],
      [32, options.karimPayable ?? 10160],
    ]),
    advances: new Map([[31, 15000]]),
    rows: [],
    saves: [],
    edits: [],
    cancels: [],
    saved: [],
  };
  const counters: Record<ScreenType, number> = { receipt: 0, payment: 0, advance: 0 };
  const openBillsOf = (id: number) => store.bills.get(id) ?? [];
  const dueOf = (id: number) => openBillsOf(id).reduce((total, one) => total + one.open, 0);
  const accountOf = (entry: BookEntry | BuiltVoucher['input']) =>
    'seller_account_id' in entry ? entry.seller_account_id : entry.account_id;

  function preview(asked: VoucherEntry): VoucherPreview {
    const entry = bookEntry(asked);
    const id = accountOf(entry);
    const party = partyOf(id);
    const amountText = entry.amount.replaceAll(',', '');
    const decimals = amountText.includes('.');
    const amount = decimals ? 0 : rupeesOf(entry.amount);
    const errors: FieldError[] = [];
    const warnings: FieldError[] = [];
    if (decimals) errors.push({ code: 'ERR_DECIMALS_NOT_ALLOWED', field: 'amount', values: {} });
    if (
      entry.type !== 'advance' &&
      party?.code === '03049999' &&
      (entry.person_key ?? null) === null
    ) {
      errors.push({ code: 'ERR_PERSON_REQUIRED', field: 'person_key', values: {} });
    }
    if (entry.type === 'payment' && party?.type === 'buyer' && entry.reason.trim() === '') {
      errors.push({ code: 'ERR_PAYMENT_TO_BUYER_NEEDS_REASON', field: 'reason', values: {} });
    }
    const buyer = party?.type === 'buyer';
    const seller = party?.type === 'seller';
    const due = dueOf(id);
    // Receipt from a buyer: oldest bills first, the rest stays as credit.
    let left = entry.type === 'receipt' && buyer ? amount : 0;
    const billEffects = (entry.type === 'receipt' && buyer ? openBillsOf(id) : []).map((one) => {
      const settled = Math.min(left, one.open);
      left -= settled;
      return {
        bill: {
          ...bill(one.lineId, one.date, one.number, one.amount),
          open_paisa: money(one.open),
        },
        settled_paisa: money(settled),
        still_open_paisa: money(one.open - settled),
      };
    });
    const payable = store.payable.get(id) ?? 0;
    if (entry.type === 'payment' && seller && amount > payable && payable > 0) {
      warnings.push({
        code: 'ERR_PAYMENT_ABOVE_PAYABLE',
        field: 'amount',
        values: { payable_paisa: payable * 100, over_paisa: (amount - payable) * 100 },
      });
    }
    const change = entry.type === 'receipt' ? amount : -amount;
    const current = buyer ? balance(due, 'dr') : seller ? balance(payable, 'cr') : ZERO;
    const after = buyer
      ? entry.type === 'payment'
        ? balance(due + amount, 'dr')
        : balance(
            Math.max(due - amount, 0),
            due - amount < 0 ? 'cr' : due - amount === 0 ? 'zero' : 'dr',
          )
      : seller
        ? balance(Math.max(payable - amount, 0), payable - amount > 0 ? 'cr' : 'zero')
        : ZERO;
    return {
      account: refOf(id),
      amount_paisa: money(amount),
      current_balance: current,
      new_balance: after,
      bill_effects: billEffects,
      extra_credit_paisa: money(left),
      advance_effects: [],
      owes_settled_paisa: money(0),
      return_limit_paisa:
        entry.type === 'receipt' && seller ? money(store.advances.get(id) ?? 0) : null,
      payable_open_paisa: entry.type === 'payment' && seller ? money(payable) : null,
      payable_is_zero_for_seller: entry.type === 'payment' && seller && payable === 0,
      cash: cashCheck(store.cash, change),
      errors,
      warnings,
    };
  }

  function save(built: BuiltVoucher): VoucherSaved {
    store.saves.push(built);
    const input = built.input;
    const id = accountOf(input);
    const amount = rupeesOf(input.amount);
    const change = built.type === 'receipt' ? amount : -amount;
    if (
      built.type !== 'receipt' &&
      !('confirm_cash_below_zero' in input && input.confirm_cash_below_zero) &&
      store.cash + change < 0
    ) {
      throw refused('ERR_CASH_BELOW_ZERO_NOT_CONFIRMED', null, {
        cash_after_paisa: (store.cash + change) * 100,
      });
    }
    store.cash += change;
    if (built.type === 'receipt') {
      let left = amount;
      for (const one of openBillsOf(id)) {
        const used = Math.min(left, one.open);
        one.open -= used;
        left -= used;
      }
    } else if (built.type === 'payment') {
      store.payable.set(id, Math.max((store.payable.get(id) ?? 0) - amount, 0));
    } else {
      store.advances.set(id, (store.advances.get(id) ?? 0) + amount);
      store.saved.push({ seller: id, amount: input.amount });
    }
    counters[built.type] += 1;
    const number = counters[built.type];
    const voucherId = 500 + store.rows.length + 1;
    store.rows.push({
      id: voucherId,
      voucher_type: built.type,
      number,
      entry_date: isoOf(input.date),
      account: refOf(id),
      amount_paisa: money(amount),
      // The columns of an old row stay on the row; a new voucher never writes them (PRD 15.1).
      mode: 'cash',
      bank_account_id: null,
      cheque_no: '',
      cheque_date: null,
      expected_goods: built.type === 'advance' ? built.input.expected_goods : '',
      narration: input.narration,
      status: 'saved',
      version: 1,
      person: null,
      collector_staff_id: built.type === 'receipt' ? built.input.collector_staff_id : null,
      reason: built.type === 'payment' ? built.input.reason : '',
      cancel_reason: '',
      entry_id: 1000 + voucherId,
      editable: true,
    });
    const isBuyer = partyOf(id)?.type === 'buyer';
    return {
      voucher_id: voucherId,
      voucher_type: built.type,
      number,
      entry_id: 1000 + voucherId,
      new_balance: isBuyer ? balance(dueOf(id), 'dr') : balance(store.payable.get(id) ?? 0, 'cr'),
    };
  }

  function edit(
    input: Omit<VoucherBooks['edits'][number], 'entry'> & { entry: VoucherEntry },
  ): VoucherSaved {
    const entry = bookEntry(input.entry);
    store.edits.push({ ...input, entry });
    const row = store.rows.find((one) => one.id === input.id);
    if (row === undefined) throw refused('ERR_NOT_FOUND', null);
    row.version += 1;
    row.amount_paisa = money(rupeesOf(entry.amount));
    return {
      voucher_id: row.id,
      voucher_type: row.voucher_type,
      number: row.number,
      entry_id: row.entry_id ?? 0,
      new_balance: ZERO,
    };
  }

  function cancel(input: VoucherBooks['cancels'][number]): VoucherCancelled {
    store.cancels.push(input);
    const row = store.rows.find((one) => one.id === input.id);
    if (row === undefined) throw refused('ERR_NOT_FOUND', null);
    row.status = 'cancelled';
    row.editable = false;
    row.cancel_reason = input.reason;
    return {
      voucher_id: row.id,
      voucher_type: row.voucher_type,
      number: row.number,
      reversal_entry_id: 9000 + row.id,
      new_balance: ZERO,
    };
  }

  function get(id: number): VoucherRow {
    const row = store.rows.find((one) => one.id === id);
    if (row === undefined) throw refused('ERR_NOT_FOUND', null);
    // A copy, as the real answer is: a row changed later must not change an answer already read.
    return { ...row };
  }

  // Like the backend (plan 15): the rows, and the sum of the saved ones. A cancelled voucher is in
  // the rows (marked by its status) and not in the total. The sum is a test double's.
  function list(input: VouchersListInput): VouchersList {
    const rows = store.rows
      .filter(
        (row) =>
          (input.voucher_type === null || row.voucher_type === input.voucher_type) &&
          (input.from === '' || row.entry_date >= isoOf(input.from)) &&
          (input.to === '' || row.entry_date <= isoOf(input.to)) &&
          (input.account_id === null || row.account?.id === input.account_id),
      )
      .map((row) => ({ ...row }));
    const total = rows
      .filter((row) => row.status === 'saved')
      // The amount as the screen would show it, read back as rupees: a test double's sum.
      .reduce(
        (sum, row) => sum + rupeesOf(formatPaisa(row.amount_paisa, { grouping: 'international' })),
        0,
      );
    // As the backend: a list of all types has no total (null). A page, with the count and the
    // totals of the whole filter (PRD FR-19.11).
    return {
      rows: sliceOf(rows, input.offset, input.limit),
      total_count: rows.length,
      totals: {
        cancelled_count: rows.filter((row) => row.status === 'cancelled').length,
        total_paisa: input.voucher_type === null ? null : money(total),
      },
    };
  }

  function openBills(input: { account_id: number }): OpenBills {
    const bills = openBillsOf(input.account_id).filter((one) => one.open > 0);
    return {
      account: refOf(input.account_id) ?? {
        id: input.account_id,
        code: '',
        name_ur: '',
        name_en: '',
      },
      bills: bills.map((one) => ({
        ...bill(one.lineId, one.date, one.number, one.amount),
        open_paisa: money(one.open),
      })),
      // As the backend (plan 15 T4): the count of all open bills, and whether the list holds only
      // the oldest 200 of them. The test double never has more than a few.
      total_count: bills.length,
      truncated: false,
      open_paisa: money(dueOf(input.account_id)),
      unmatched_credit_paisa: money(0),
      balance: dueOf(input.account_id) === 0 ? ZERO : balance(dueOf(input.account_id), 'dr'),
      oldest_days: bills.length === 0 ? null : 0,
    };
  }

  function openPayables(input: { seller_id: number }): OpenPayables {
    const payable = store.payable.get(input.seller_id) ?? 0;
    const postings: PayablePosting[] =
      payable === 0
        ? []
        : [
            {
              entry_date: TODAY_ISO,
              source: 'bikri',
              number: 1,
              credit_paisa: money(24316),
              advance_adjusted_paisa: money(15000),
              paid_paisa: money(24316 - 15000 - payable),
              still_open_paisa: money(payable),
              days: 0,
            },
          ];
    return {
      account: refOf(input.seller_id) ?? {
        id: input.seller_id,
        code: '',
        name_ur: '',
        name_en: '',
      },
      postings,
      total_count: postings.length,
      truncated: false,
      open_paisa: money(payable),
      advances_open_paisa: money(store.advances.get(input.seller_id) ?? 0),
      owes_firm_paisa: money(0),
      balance: payable === 0 ? ZERO : balance(payable, 'cr'),
    };
  }

  function advances(sellerId: number): AdvancesInfo {
    const open = store.advances.get(sellerId) ?? 0;
    const rows: OpenAdvance[] =
      open === 0
        ? []
        : [
            {
              source: 'voucher',
              voucher_number: 1,
              entry_date: '2026-09-01',
              given_paisa: money(open),
              adjusted_paisa: money(0),
              open_paisa: money(open),
            },
          ];
    return {
      open: rows,
      open_paisa: money(open),
      last:
        open === 0
          ? null
          : { entry_date: '2026-09-01', amount_paisa: money(open), voucher_number: 1 },
    };
  }

  return { store, preview, save, edit, cancel, get, list, openBills, openPayables, advances };
}

/** The collection list of the worked day: two buyers of one collector in one city. */
export function collectionList(rows: 'two' | 'none' = 'two') {
  const group = {
    collector: { id: 81, code: '0001', name_ur: 'اسلم', name_en: 'Aslam' },
    city: { id: 91, code: '0001', name_ur: 'کوٹ رادھا کشن', name_en: 'Kot Radha Kishan' },
    buyers: [
      {
        account: refOf(21) ?? { id: 21, code: '', name_ur: '', name_en: '' },
        phone: '03001234567',
        dues_paisa: money(2880),
        oldest_bill_date: TODAY_ISO,
        oldest_days: 3,
      },
      {
        account: refOf(22) ?? { id: 22, code: '', name_ur: '', name_en: '' },
        phone: '',
        dues_paisa: money(18684),
        oldest_bill_date: TODAY_ISO,
        oldest_days: 0,
      },
    ],
    total_paisa: money(21564),
  };
  return {
    date: TODAY_ISO,
    weekday: 5,
    groups: rows === 'none' ? [] : [group],
    total_paisa: money(rows === 'none' ? 0 : 21564),
  };
}

/** A saved voucher as the backend lists it: a receipt of 10,000 from Akram on today, still open. */
export function voucherRow(overrides: Partial<VoucherRow> = {}): VoucherRow {
  return {
    id: 501,
    voucher_type: 'receipt',
    number: 1,
    entry_date: TODAY_ISO,
    account: refOf(21),
    amount_paisa: money(10000),
    mode: 'cash',
    bank_account_id: null,
    cheque_no: '',
    cheque_date: null,
    expected_goods: '',
    narration: '',
    status: 'saved',
    version: 1,
    person: null,
    collector_staff_id: null,
    reason: '',
    cancel_reason: '',
    entry_id: 1501,
    editable: true,
    ...overrides,
  };
}

/** A row of the receipts batch that was saved (PRD FR-7.61): its voucher and no problem. */
export function savedReceiptRow(index: number): ReceiptRowResult {
  return {
    saved: {
      voucher_id: 600 + index,
      voucher_type: 'receipt',
      number: index + 1,
      entry_id: 7000 + index,
      new_balance: { amount_paisa: money(0), side: 'zero' },
    },
    skipped: false,
    errors: [],
  };
}

/** A blank row of the batch: the backend skips it and says so (no amount, no problem). */
export function skippedReceiptRow(): ReceiptRowResult {
  return { saved: null, skipped: true, errors: [] };
}

/** A row the backend refused: its problem is at `rows[index].<field>`. */
export function refusedReceiptRow(
  index: number,
  code: ErrorCode,
  field = 'amount',
): ReceiptRowResult {
  return {
    saved: null,
    skipped: false,
    errors: [{ code, field: `rows[${String(index)}].${field}`, values: {} }],
  };
}
