// The canned data of the component tests has the shape of the backend's real answers (plan 15, T4).
// The backend wrote each real answer in its own test (tests/fixtures/*.json, copied, never edited).
// This compares the keys and kinds at every level, so a test that passes on canned data cannot hide
// a field the real backend does not send, or one it sends and the canned data forgot. Then it checks
// that the screen builds exactly the request the backend's own test sent for the same entry.

import { describe, expect, it } from 'vitest';
import type { GlobalHit } from '../../src/bindings/GlobalHit';
import { build, entryOf, newDraft } from '../../src/features/vouchers/draft';
import type { VoucherDraft } from '../../src/features/vouchers/types';
import { createBooks, collectionList, voucherRow } from '../helpers/voucher-books';
import { voucherPrintData } from '../helpers/print-data';
import * as real from '../helpers/vouchers-real';

type Shape = string | null | Shape[] | { [key: string]: Shape };

/** The keys and kinds of a value at every level; a list is shown by its first element. */
function shapeOf(value: unknown): Shape {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value)) return value.length === 0 ? [] : [shapeOf(value[0])];
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, item]) => [key, shapeOf(item)]),
    );
  }
  return typeof value;
}

/** The first difference between two shapes, or null. A null or an empty list can hold anything. */
function firstDifference(a: Shape, b: Shape, path = ''): string | null {
  if (a === null || b === null) return null;
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b)) return `${path}: list against not a list`;
    const [x, y] = [a[0], b[0]];
    return x === undefined || y === undefined ? null : firstDifference(x, y, `${path}[]`);
  }
  if (typeof a === 'string' || typeof b === 'string') {
    return a === b ? null : `${path}: ${JSON.stringify(a)} against ${JSON.stringify(b)}`;
  }
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const key of [...keys].sort()) {
    const [x, y] = [a[key], b[key]];
    if (x === undefined) return `${path}.${key}: the real answer has no such field`;
    if (y === undefined) return `${path}.${key}: the canned data has no such field`;
    const inner = firstDifference(x, y, `${path}.${key}`);
    if (inner !== null) return inner;
  }
  return null;
}

const same = (realValue: unknown, canned: unknown) =>
  firstDifference(shapeOf(realValue), shapeOf(canned));

const RECEIPT_ENTRY = (accountId: number, amount: string) => ({
  type: 'receipt' as const,
  date: '10-10-2026',
  account_id: accountId,
  person_key: null,
  amount,
  mode: 'cash' as const,
  bank_account_id: null,
  cheque_no: '',
  cheque_date: '',
  collector_staff_id: null,
  narration: '',
});

describe('the comparison itself', () => {
  it('sees a missing field, an extra field and a different kind', () => {
    expect(same({ a: 1, b: 2 }, { a: 1 })).toContain('.b');
    expect(same({ a: 1 }, { a: 1, c: 2 })).toContain('.c');
    expect(same({ a: 1 }, { a: 'x' })).toContain('.a');
    expect(same({ a: [{ k: 1 }] }, { a: [{ j: 1 }] })).not.toBeNull();
    expect(same({ a: null }, { a: 1 })).toBeNull();
  });
});

describe('the canned data of the component tests has the real shape', () => {
  it('a preview: a receipt, a payment to a seller, cash lent, an advance, a seller return, an edit', () => {
    const books = createBooks();
    expect(same(real.PREVIEW_RECEIPT_BUYER, books.preview(RECEIPT_ENTRY(21, '10000')))).toBeNull();
    const payment = (accountId: number) => ({
      type: 'payment' as const,
      date: '10-10-2026',
      account_id: accountId,
      person_key: null,
      amount: '4316',
      mode: 'cash' as const,
      bank_account_id: null,
      cheque_no: '',
      cheque_date: '',
      reason: accountId === 21 ? 'x' : '',
      narration: '',
      confirm_cash_below_zero: false,
    });
    expect(same(real.PREVIEW_PAYMENT_SELLER, books.preview(payment(31)))).toBeNull();
    expect(same(real.PREVIEW_PAYMENT_BUYER, books.preview(payment(21)))).toBeNull();
    expect(
      same(
        real.PREVIEW_ADVANCE,
        books.preview({
          type: 'advance',
          date: '10-10-2026',
          seller_account_id: 31,
          amount: '15000',
          mode: 'cash',
          bank_account_id: null,
          cheque_no: '',
          cheque_date: '',
          expected_goods: '',
          narration: '',
          confirm_cash_below_zero: false,
        }),
      ),
    ).toBeNull();
    // Every real preview has the same keys as every other.
    for (const one of [
      real.PREVIEW_PAYMENT_SELLER,
      real.PREVIEW_PAYMENT_BUYER,
      real.PREVIEW_PAYMENT_NOTHING_PAYABLE,
      real.PREVIEW_PAYMENT_CASH_BELOW_ZERO,
      real.PREVIEW_ADVANCE,
      real.PREVIEW_RECEIPT_SELLER_RETURN,
      real.PREVIEW_RECEIPT_EDIT,
    ]) {
      expect(same(real.PREVIEW_RECEIPT_BUYER, one)).toBeNull();
    }
  });

  it('the bills a receipt settles, and the advances a seller return adjusts', () => {
    const books = createBooks();
    const preview = books.preview(RECEIPT_ENTRY(21, '10000'));
    expect(same(real.PREVIEW_RECEIPT_BUYER.bill_effects[0], preview.bill_effects[0])).toBeNull();
    expect(real.PREVIEW_RECEIPT_SELLER_RETURN.advance_effects[0]).toBeDefined();
    expect(real.PREVIEW_RECEIPT_SELLER_RETURN.return_limit_paisa).not.toBeNull();
  });

  it('the result of a save, of an edit and of a cancel', () => {
    const books = createBooks();
    const saved = books.save({ type: 'receipt', input: stripType(RECEIPT_ENTRY(21, '10000')) });
    expect(same(real.SAVED_RECEIPT, saved)).toBeNull();
    for (const one of [
      real.SAVED_PAYMENT,
      real.SAVED_ADVANCE,
      real.SAVED_CASH_LENT,
      real.SAVED_CHEQUE,
      real.EDITED_RECEIPT,
    ]) {
      expect(same(real.SAVED_RECEIPT, one)).toBeNull();
    }
    expect(Object.keys(real.CANCELLED_CASH_LENT).sort()).toEqual(
      Object.keys(
        books.cancel({ id: saved.voucher_id, version: 1, reason: 'x', confirm: false }),
      ).sort(),
    );
  });

  it('a voucher row and the list with its total', () => {
    expect(same(real.GOT_RECEIPT, voucherRow())).toBeNull();
    const books = createBooks();
    books.save({ type: 'receipt', input: stripType(RECEIPT_ENTRY(21, '10000')) });
    const list = books.list({
      voucher_type: 'receipt',
      from: '',
      to: '',
      account_id: null,
      person_key: null,
    });
    expect(same(real.LIST_PAYMENTS, list)).toBeNull();
    expect(same(real.LIST_ALL, { rows: list.rows, total_paisa: list.total_paisa })).toBeNull();
  });

  it('the open bills, the payables and the advances of a seller', () => {
    const books = createBooks();
    expect(same(real.OPEN_BILLS_AKRAM, books.openBills({ account_id: 21 }))).toBeNull();
    expect(same(real.OPEN_BILLS_AKRAM, real.OPEN_BILLS_TRUNCATED)).toBeNull();
    expect(same(real.OPEN_PAYABLES_RAHIM, books.openPayables({ seller_id: 31 }))).toBeNull();
    expect(same(real.OPEN_PAYABLES_RAHIM, real.OPEN_PAYABLES_TRUNCATED)).toBeNull();
    expect(same(real.ADVANCES_KARIM, books.advances(32))).toBeNull();
  });

  it('the collection list and the result of the batch of receipts', () => {
    expect(same(real.COLLECTION_SATURDAY, collectionList('two'))).toBeNull();
    // A batch answer has a result for each row: saved, skipped, or refused with its problem.
    const [saved, skipped, refused] = real.RECEIPTS_BATCH.rows;
    expect(saved?.saved).not.toBeNull();
    expect(skipped).toMatchObject({ saved: null, skipped: true, errors: [] });
    expect(refused?.errors[0]?.field).toBe('rows[2].amount');
  });

  it('a one-time person made regular', () => {
    const canned = {
      account: { id: 24, code: '03040003', name_ur: 'شاہد', name_en: 'Shahid' },
      moved: { amount_paisa: 0, side: 'zero' },
      entry_id: null,
    };
    expect(same(real.MADE_REGULAR, canned)).toBeNull();
  });

  it('the receipt slip: a receipt, a payment, an advance, a cheque', () => {
    expect(same(real.SLIP_RECEIPT_UR, voucherPrintData('receipt', { language: 'ur' }))).toBeNull();
    expect(same(real.SLIP_PAYMENT_EN, voucherPrintData('payment', { language: 'en' }))).toBeNull();
    expect(same(real.SLIP_ADVANCE_UR, voucherPrintData('advance', { language: 'ur' }))).toBeNull();
    expect(
      same(real.SLIP_CHEQUE_EN, voucherPrintData('receipt', { language: 'en', mode: 'cheque' })),
    ).toBeNull();
  });

  it('a voucher found by global search', () => {
    const voucher: GlobalHit | undefined = real.GLOBAL_VOUCHER_NUMBER.find(
      (hit) => hit.kind === 'voucher',
    );
    expect(voucher).toBeDefined();
  });
});

/** The input of a save is the entry without its type tag. */
function stripType<T extends { type: string }>(entry: T): Omit<T, 'type'> {
  return Object.fromEntries(Object.entries(entry).filter(([key]) => key !== 'type')) as Omit<
    T,
    'type'
  >;
}

describe('the screen builds the request the backend’s own test sent', () => {
  const buyer = { id: 20, code: '03040001', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' };
  const seller = { id: 18, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim' };
  const bank = { id: 1, code: '0001', name_ur: 'بینک', name_en: 'Bank' };
  const draft = (type: VoucherDraft['type'], changes: Partial<VoucherDraft>): VoucherDraft => ({
    ...newDraft(type, '2026-10-10'),
    ...changes,
  });

  it('a receipt from a buyer, as typed with the thousands comma', () => {
    const built = build(
      draft('receipt', { account: buyer, accountType: 'buyer', amount: '10,000' }),
      false,
    );
    expect(built).toEqual({ type: 'receipt', input: real.REQUEST_SAVE_RECEIPT });
    if (built === null) throw new Error('no voucher');
    expect(entryOf(built)).toEqual(real.REQUEST_PREVIEW_RECEIPT_BUYER.entry);
  });

  it('a receipt by cheque into a bank account', () => {
    const built = build(
      draft('receipt', {
        account: { ...buyer, id: 21, code: '03040002' },
        accountType: 'buyer',
        amount: '5,000',
        mode: 'cheque',
        bank,
        cheque_no: '000123',
        cheque_date: '12-10-2026',
      }),
      false,
    );
    expect(built).toEqual({ type: 'receipt', input: real.REQUEST_SAVE_RECEIPT_CHEQUE });
  });

  it('cash lent to a buyer carries the reason', () => {
    const built = build(
      draft('payment', {
        account: buyer,
        accountType: 'buyer',
        amount: '3,000',
        reason: 'نقد ادھار',
      }),
      false,
    );
    expect(built).toEqual({ type: 'payment', input: real.REQUEST_SAVE_CASH_LENT });
  });

  it('a payment to a seller has no reason, and an advance names its seller', () => {
    const payment = build(
      draft('payment', { account: seller, accountType: 'seller', amount: '4,316', reason: 'left' }),
      false,
    );
    expect(payment).toEqual({ type: 'payment', input: real.REQUEST_SAVE_PAYMENT });
    const advance = build(
      draft('advance', { account: seller, accountType: 'seller', amount: '15,000' }),
      false,
    );
    expect(advance).toEqual({ type: 'advance', input: real.REQUEST_SAVE_ADVANCE });
  });

  it('an edit takes back the voucher it replaces, and a cancel sends the version read', () => {
    expect(real.REQUEST_PREVIEW_RECEIPT_EDIT.replacing_voucher_id).toBe(2);
    expect(real.REQUEST_EDIT_RECEIPT).toMatchObject({
      id: 2,
      version: 2,
      confirm_cash_below_zero: false,
    });
    expect(real.REQUEST_CANCEL_CASH_LENT).toMatchObject({ id: 4, version: 2 });
  });
});
