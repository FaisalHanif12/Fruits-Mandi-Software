// Printing a voucher from the screen (plan 15, PRD FR-15.2, FR-7.36, FR-7.57): the print box of a
// saved voucher (language, printer or PDF, copies, preview), F9, the Print button of the list, and
// Ctrl+Enter printing the slip when it is ticked. The slip itself is in print-layout.test.tsx and
// the hidden window in print-route.test.tsx.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { fakePrinting, type FakePrinting } from '../helpers/print';
import { voucherPrintData } from '../helpers/print-data';
import { message, visible } from '../helpers/render';
import {
  AKRAM,
  KARIM,
  RAHIM,
  fakeVouchers,
  voucherRow,
  type FakeVouchers,
} from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
const textOf = (element: HTMLElement) => visible(element.textContent);

/** The box reads the Printing settings first; it shows its form when they have arrived. */
async function theBox() {
  const box = await screen.findByRole('dialog');
  await within(box).findByRole('checkbox');
  return box;
}

describe.each(LANGUAGES)('printing a voucher from the screen (%s)', (language) => {
  let store: FakeVouchers;
  let printing: FakePrinting;

  beforeEach(() => {
    vi.resetAllMocks();
    healthyBackend();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
    vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
    vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
    const rows = [AKRAM, RAHIM, KARIM];
    fakeAccounts(rows);
    fakeAccountSearch(rows);
    fakeMasters({});
    store = fakeVouchers();
    printing = fakePrinting();
    vi.mocked(reportsApi.reportsVoucherPrintData).mockImplementation((input) =>
      Promise.resolve(voucherPrintData('receipt', { language: input.language })),
    );
    vi.mocked(reportsApi.reportsPrintVoucher).mockResolvedValue({
      printed: ['receipt_slip'],
      pdf_paths: null,
    });
  });
  afterEach(() => {
    leaveAppWindow();
  });

  async function open() {
    await renderApp('/bikri/voucher?type=receipt');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-receipt-title')),
    });
    return userEvent.setup();
  }

  async function saveOne(user: ReturnType<typeof userEvent.setup>, andNext = false) {
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      }),
    );
    await user.keyboard('0020');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    await user.click(
      await screen.findByLabelText(named(message(language, 'vouchers-field-amount'))),
    );
    await user.keyboard('10000');
    if (andNext) press('Enter', { ctrlKey: true });
    else press('F10');
    await waitFor(() => {
      expect(store.rows).toHaveLength(1);
    });
  }

  it('F9 with nothing saved says to save first, and opens no box', async () => {
    await open();
    press('F9');
    expect(
      await screen.findByText(named(message(language, 'vouchers-print-save-first'))),
    ).toBeInTheDocument();
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('F9 after a save opens the box for that voucher; Print sends the language, the copies and the target', async () => {
    const user = await open();
    await saveOne(user);
    press('F9');
    const box = await theBox();
    expect(textOf(box)).toContain(
      message(language, 'vouchers-print-title', {
        number: message(language, 'vouchers-number-receipt', { number: 1 }),
      }),
    );
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-to-pdf')) }),
    );
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'common-language-en')) }),
    );
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-save-pdf')) }),
    );
    await waitFor(() => {
      expect(reportsApi.reportsPrintVoucher).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintVoucher).mock.calls[0]?.[0]).toEqual({
      voucher_id: 501,
      language: 'en',
      copies: null,
      to: 'pdf',
    });
  });

  it('the preview shows the slip in the print language, and the app keeps its own', async () => {
    const user = await open();
    await saveOne(user);
    press('F9');
    const box = await theBox();
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'common-language-en')) }),
    );
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-preview')) }),
    );
    await waitFor(() => {
      expect(textOf(box)).toContain('Ten thousand rupees only');
    });
    expect(vi.mocked(reportsApi.reportsVoucherPrintData)).toHaveBeenCalledWith({
      voucher_id: 501,
      language: 'en',
    });
    expect(useLanguageStore.getState().language).toBe(language);
  });

  it('a print that fails shows its reason in the box and keeps the voucher saved', async () => {
    vi.mocked(reportsApi.reportsPrintVoucher).mockRejectedValueOnce(
      (await import('../helpers/app')).fail({ code: 'ERR_PRINT_WINDOWS_ONLY' }),
    );
    const user = await open();
    await saveOne(user);
    press('F9');
    const box = await theBox();
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-print')) }),
    );
    await waitFor(() => {
      expect(textOf(box)).toContain(message(language, 'err-print-windows-only'));
    });
    expect(store.rows).toHaveLength(1);
  });

  it('the tick in the box is remembered as the print-after-save choice, next to the other documents', async () => {
    const user = await open();
    await saveOne(user);
    press('F9');
    const box = await theBox();
    await user.click(
      within(box).getByRole('checkbox', {
        name: named(message(language, 'print-doc-receipt-slip')),
      }),
    );
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-print')) }),
    );
    await waitFor(() => {
      expect(printing.saves).toHaveLength(1);
    });
    expect(printing.saves[0]?.print_after_save).toEqual([
      'seller_bill',
      'buyer_slips',
      'pukhta_slip',
      'receipt_slip',
    ]);
  });

  it('Ctrl+Enter saves and prints the slip when it is ticked, and the next voucher starts', async () => {
    printing = fakePrinting({
      settings: { print_after_save: ['receipt_slip'] },
    });
    const user = await open();
    await saveOne(user, true);
    await waitFor(() => {
      expect(reportsApi.reportsPrintVoucher).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintVoucher).mock.calls[0]?.[0]).toEqual({
      voucher_id: 501,
      language,
      copies: null,
      to: 'printer',
    });
    expect(
      (await screen.findAllByText(named(message(language, 'print-toast-printed')))).length,
    ).toBeGreaterThan(0);
  });

  it('Ctrl+Enter prints nothing when the slip is not ticked, and the voucher is saved all the same', async () => {
    const user = await open();
    await saveOne(user, true);
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(reportsApi.reportsPrintVoucher).not.toHaveBeenCalled();
    expect(store.rows).toHaveLength(1);
    expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
  });

  it('a row of the list has a Print button that opens the box for it', async () => {
    store.rows.push(voucherRow({ id: 700, number: 7 }));
    const user = await open();
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="700"]')).not.toBeNull();
    });
    const cellOf = document.querySelector<HTMLElement>('.ag-row[row-id="700"] [col-id="number"]');
    if (cellOf === null) throw new Error('no cell');
    await user.click(cellOf);
    // The Print button of the list, not the one at the top of the screen.
    const list = (
      await screen.findByRole('heading', {
        name: named(message(language, 'vouchers-list-title-receipt')),
      })
    ).closest('section');
    if (list === null) throw new Error('no list card');
    await user.click(
      within(list).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'vouchers-print')),
      }),
    );
    const box = await theBox();
    expect(textOf(box)).toContain(
      message(language, 'vouchers-print-title', {
        number: message(language, 'vouchers-number-receipt', { number: 7 }),
      }),
    );
  });
});
