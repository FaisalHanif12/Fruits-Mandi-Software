// The screen and the backend agree on the voucher commands (plan 15, T4). Each wrapper sends the
// command name the app registers and the one `{ input }` object the command takes. The expected
// inputs are typed with the generated bindings, so the compiler fails this file when a field of the
// backend's input is renamed, added or removed. The answers are checked against the backend's own
// files in vouchers-real-answers.test.tsx.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { AdvanceInput } from '../../src/bindings/AdvanceInput';
import type { CollectionListInput } from '../../src/bindings/CollectionListInput';
import type { OpenBillsInput } from '../../src/bindings/OpenBillsInput';
import type { OpenPayablesInput } from '../../src/bindings/OpenPayablesInput';
import type { PaymentInput } from '../../src/bindings/PaymentInput';
import type { PrintVoucherInput } from '../../src/bindings/PrintVoucherInput';
import type { ReceiptInput } from '../../src/bindings/ReceiptInput';
import type { VoucherCancelInput } from '../../src/bindings/VoucherCancelInput';
import type { VoucherEditInput } from '../../src/bindings/VoucherEditInput';
import type { VoucherPreviewInput } from '../../src/bindings/VoucherPreviewInput';
import type { VoucherPrintDataInput } from '../../src/bindings/VoucherPrintDataInput';
import type { VouchersListInput } from '../../src/bindings/VouchersListInput';
import { reportsPrintVoucher, reportsVoucherPrintData } from '../../src/features/reports/api';
import * as api from '../../src/features/vouchers/api';
import { searchGlobal } from '../../src/shared/accounts/api';
import * as real from '../helpers/vouchers-real';
import * as invoke from '../../src/shared/api/invoke';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

const RECEIPT: ReceiptInput = {
  date: '10-10-2026',
  account_id: 21,
  amount: '10000',
  collector_staff_id: null,
  narration: '',
};
const PAYMENT: PaymentInput = {
  date: '10-10-2026',
  account_id: 31,
  amount: '4316',
  reason: '',
  narration: 'x',
  confirm_cash_below_zero: false,
};
const ADVANCE: AdvanceInput = {
  date: '10-10-2026',
  seller_account_id: 31,
  amount: '15000',
  expected_goods: 'mango',
  narration: '',
  confirm_cash_below_zero: true,
};

beforeEach(() => {
  vi.resetAllMocks();
  call.mockResolvedValue({});
});

describe('each wrapper sends the command and the input of the backend', () => {
  it('vouchers_preview: the entry with its type, and the voucher an edit takes back', async () => {
    const input: VoucherPreviewInput = {
      entry: { type: 'receipt', ...RECEIPT },
      replacing_voucher_id: null,
    };
    await api.vouchersPreview(input.entry, null);
    expect(call).toHaveBeenLastCalledWith('vouchers_preview', { input });
    await api.vouchersPreview({ type: 'payment', ...PAYMENT }, 502);
    expect(call).toHaveBeenLastCalledWith('vouchers_preview', {
      input: { entry: { type: 'payment', ...PAYMENT }, replacing_voucher_id: 502 },
    });
  });

  it('the three saves each have their own command and send the input as it is', async () => {
    await api.vouchersSave({ type: 'receipt', input: RECEIPT });
    expect(call).toHaveBeenLastCalledWith('vouchers_save_receipt', { input: RECEIPT });
    await api.vouchersSave({ type: 'payment', input: PAYMENT });
    expect(call).toHaveBeenLastCalledWith('vouchers_save_payment', { input: PAYMENT });
    await api.vouchersSave({ type: 'advance', input: ADVANCE });
    expect(call).toHaveBeenLastCalledWith('vouchers_save_advance', { input: ADVANCE });
  });

  it('vouchers_edit and vouchers_cancel: the version read with the voucher, the reason and the cash answer', async () => {
    const edit: VoucherEditInput = {
      id: 501,
      version: 2,
      entry: { type: 'receipt', ...RECEIPT },
      reason: 'wrong amount',
      confirm_cash_below_zero: true,
    };
    await api.vouchersEdit({
      id: edit.id,
      version: edit.version,
      entry: edit.entry,
      reason: edit.reason,
      confirm: edit.confirm_cash_below_zero,
    });
    expect(call).toHaveBeenLastCalledWith('vouchers_edit', { input: edit });
    const cancel: VoucherCancelInput = {
      id: 501,
      version: 2,
      reason: 'typed twice',
      confirm_cash_below_zero: false,
    };
    await api.vouchersCancel({
      id: cancel.id,
      version: cancel.version,
      reason: cancel.reason,
      confirm: cancel.confirm_cash_below_zero,
    });
    expect(call).toHaveBeenLastCalledWith('vouchers_cancel', { input: cancel });
  });

  it('vouchers_get and vouchers_list', async () => {
    await api.vouchersGet(501);
    expect(call).toHaveBeenLastCalledWith('vouchers_get', { input: { id: 501 } });
    const list: VouchersListInput = {
      voucher_type: 'receipt',
      from: '10-10-2026',
      to: '10-10-2026',
      account_id: null,
      offset: 0,
      limit: 0,
      show_finished: false,
      search: null,
      sort: null,
      group_by_month: false,
    };
    await api.vouchersList(list);
    expect(call).toHaveBeenLastCalledWith('vouchers_list', { input: list });
  });

  it('assist_open_bills, assist_open_payables, assist_advances', async () => {
    const bills: OpenBillsInput = { account_id: 21 };
    await api.assistOpenBills(bills);
    expect(call).toHaveBeenLastCalledWith('assist_open_bills', { input: bills });
    const payables: OpenPayablesInput = { seller_id: 31, include_settled: false };
    await api.assistOpenPayables(payables);
    expect(call).toHaveBeenLastCalledWith('assist_open_payables', { input: payables });
    await api.assistAdvances(31);
    expect(call).toHaveBeenLastCalledWith('assist_advances', { input: { seller_id: 31 } });
  });

  it('assist_collection_list and vouchers_save_receipts_batch', async () => {
    const list: CollectionListInput = {
      collector_staff_id: null,
      city_id: null,
      search: null,
      offset: 0,
      limit: 100,
    };
    await api.assistCollectionList(list);
    expect(call).toHaveBeenLastCalledWith('assist_collection_list', { input: list });
    await api.vouchersSaveReceiptsBatch([RECEIPT, { ...RECEIPT, account_id: 22, amount: '' }]);
    expect(call).toHaveBeenLastCalledWith('vouchers_save_receipts_batch', {
      input: { rows: [RECEIPT, { ...RECEIPT, account_id: 22, amount: '' }] },
    });
  });

  it('the voucher slip: the voucher id and the language', async () => {
    const data: VoucherPrintDataInput = { voucher_id: 501, language: 'ur' };
    await reportsVoucherPrintData(data);
    expect(call).toHaveBeenLastCalledWith('reports_voucher_print_data', { input: data });
    const print: PrintVoucherInput = { voucher_id: 501, language: 'en', copies: null, to: 'pdf' };
    await reportsPrintVoucher(print);
    expect(call).toHaveBeenLastCalledWith('reports_print_voucher', { input: print });
  });
});

describe('each wrapper sends exactly the request the backend’s own test sent', () => {
  it('the saves, the previews, the edit and the cancel', async () => {
    await api.vouchersSave({ type: 'receipt', input: real.REQUEST_SAVE_RECEIPT });
    expect(call).toHaveBeenLastCalledWith('vouchers_save_receipt', {
      input: real.REQUEST_SAVE_RECEIPT,
    });
    await api.vouchersSave({ type: 'payment', input: real.REQUEST_SAVE_CASH_LENT });
    expect(call).toHaveBeenLastCalledWith('vouchers_save_payment', {
      input: real.REQUEST_SAVE_CASH_LENT,
    });
    await api.vouchersSave({ type: 'advance', input: real.REQUEST_SAVE_ADVANCE });
    expect(call).toHaveBeenLastCalledWith('vouchers_save_advance', {
      input: real.REQUEST_SAVE_ADVANCE,
    });
    const preview = real.REQUEST_PREVIEW_RECEIPT_EDIT;
    await api.vouchersPreview(preview.entry, preview.replacing_voucher_id);
    expect(call).toHaveBeenLastCalledWith('vouchers_preview', { input: preview });
    const edit = real.REQUEST_EDIT_RECEIPT;
    await api.vouchersEdit({
      id: edit.id,
      version: edit.version,
      entry: edit.entry,
      reason: edit.reason,
      confirm: edit.confirm_cash_below_zero,
    });
    expect(call).toHaveBeenLastCalledWith('vouchers_edit', { input: edit });
    const cancel = real.REQUEST_CANCEL_CASH_LENT;
    await api.vouchersCancel({
      id: cancel.id,
      version: cancel.version,
      reason: cancel.reason,
      confirm: cancel.confirm_cash_below_zero,
    });
    expect(call).toHaveBeenLastCalledWith('vouchers_cancel', { input: cancel });
  });

  it('the reads, the lists and the collection round', async () => {
    await api.vouchersList(real.REQUEST_LIST_PAYMENTS);
    expect(call).toHaveBeenLastCalledWith('vouchers_list', { input: real.REQUEST_LIST_PAYMENTS });
    await api.vouchersList(real.REQUEST_LIST_ALL);
    expect(call).toHaveBeenLastCalledWith('vouchers_list', { input: real.REQUEST_LIST_ALL });
    await api.assistOpenBills(real.REQUEST_OPEN_BILLS_AKRAM);
    expect(call).toHaveBeenLastCalledWith('assist_open_bills', {
      input: real.REQUEST_OPEN_BILLS_AKRAM,
    });
    await api.assistOpenPayables(real.REQUEST_OPEN_PAYABLES_RAHIM);
    expect(call).toHaveBeenLastCalledWith('assist_open_payables', {
      input: real.REQUEST_OPEN_PAYABLES_RAHIM,
    });
    await api.assistCollectionList(real.REQUEST_COLLECTION_LIST);
    expect(call).toHaveBeenLastCalledWith('assist_collection_list', {
      input: real.REQUEST_COLLECTION_LIST,
    });
    await api.vouchersSaveReceiptsBatch(real.REQUEST_RECEIPTS_BATCH.rows);
    expect(call).toHaveBeenLastCalledWith('vouchers_save_receipts_batch', {
      input: real.REQUEST_RECEIPTS_BATCH,
    });
  });
});

describe('search_global keeps the voucher hits of the backend’s answer', () => {
  it('the type, number, day, account, amount and state of each voucher', async () => {
    call.mockResolvedValue(real.GLOBAL_VOUCHER_NUMBER);
    const found = await searchGlobal('1');
    const vouchers = found.filter((hit) => hit.kind === 'voucher');
    expect(vouchers.map((hit) => [hit.voucher_type, hit.number])).toEqual([
      ['receipt', 1],
      ['payment', 1],
      ['advance', 1],
    ]);
    expect(vouchers[0]).toMatchObject({
      id: 2,
      entry_date: '2026-10-10',
      account: { code: '0020' },
      status: 'saved',
    });
    // A voucher hit and an account hit with the same id are two rows.
    expect(found.filter((hit) => hit.kind === 'account').length).toBeGreaterThan(0);
  });
});
