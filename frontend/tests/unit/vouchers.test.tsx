// The voucher screen (plans 10 and 15, PRD 15.1 to 15.4, FR-7.58 to FR-7.61, FR-7.67, FR-7.68,
// FR-6.7), in the Urdu and in the English app, by keyboard, against a fake backend: ONE screen with
// a type list for وصولی, ادائیگی and پیشگی, the helper panels, the notes under the form, the save
// with F10, the cash box, and the language switch in the middle of an entry. The screen works out
// no money: every figure shown is the fake backend's.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts, money } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { FORMER_PAYMENT_LABELS, labelIs } from '../helpers/former-payment-labels';
import { fakeMasters, record } from '../helpers/masters';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import {
  AKRAM,
  ARSHAD,
  KARIM,
  PUKHTA_BUYER,
  RAHIM,
  TRANSPORT,
  fakeVouchers,
  type FakeVouchers,
} from '../helpers/vouchers';
import { EMPTY_PAIRS, sliceOf } from '../helpers/paged';
import { voucherRow } from '../helpers/voucher-books';

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

const STAFF = record('staff', 81, '0001', 'اسلم', 'Aslam');

const named = (text: string) => (name: string) => visible(name) === text;

const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });

const saveButton = (language: Language) =>
  screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) });

const pageText = () => visible(document.body.textContent);

/** Types in an account picker and takes the only match. */
async function pick(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  fieldId: string,
  typed: string,
  shown: string,
) {
  await user.click(await dropdown(language, fieldId));
  await user.keyboard(typed);
  // The recent list is open before the search answers: wait until the matches are the only rows.
  await waitFor(() => {
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(1);
    expect(visible(options[0]?.textContent)).toContain(shown);
  });
  await user.keyboard('{Enter}');
}

/** The text of a panel, by the heading of its card. */
async function panelText(language: Language, titleId: string): Promise<string> {
  const title = await screen.findByRole('heading', { name: named(message(language, titleId)) });
  return visible(title.closest('section')?.textContent);
}

let store: FakeVouchers;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, PUKHTA_BUYER, RAHIM, KARIM, TRANSPORT];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({ staff: [STAFF] });
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the voucher screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    store = fakeVouchers();
  });

  async function open(type: 'receipt' | 'payment' | 'advance' = 'receipt') {
    const app = await renderApp(`/bikri/voucher?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, `vouchers-${type}-title`),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('starts with today as the date and nothing to save, and has no mode, bank or cheque field', async () => {
    await open();
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('');
    expect(saveButton(language)).toBeDisabled();
    // All money is روکڑ: the form never asks how it moves (PRD 15.1).
    for (const gone of FORMER_PAYMENT_LABELS[language]) {
      expect(screen.queryByLabelText(labelIs(gone))).toBeNull();
    }
  });

  it('the type list has the three types, and the old addresses and menu items lead to it', async () => {
    const { router } = await open('payment');
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-type') });
    const buttons = within(group).getAllByRole('button');
    expect(buttons.map((button) => visible(button.textContent))).toEqual([
      message(language, 'vouchers-type-receipt'),
      message(language, 'vouchers-type-payment'),
      message(language, 'vouchers-type-advance'),
    ]);
    expect(buttons[1]).toHaveAttribute('aria-pressed', 'true');
    act(() => {
      void router.navigate({ to: '/bikri/advance' });
    });
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-advance-title'),
      }),
    ).toBeInTheDocument();
    expect(router.state.location.pathname).toBe('/bikri/voucher');
    expect(router.state.location.search).toMatchObject({ type: 'advance' });
  });

  it('a receipt from a buyer shows his party card and open bills, and the backend marks what the amount settles', async () => {
    const { user } = await open('receipt');
    await pick(user, language, 'vouchers-field-buyer', '0', '0020');
    const bills = await panelText(language, 'vouchers-bills-title');
    expect(bills).toContain('12,880');
    expect(bills).toContain(message(language, 'vouchers-source-bikri', { number: 1 }));
    expect(await screen.findByText(message(language, 'assist-card-balance'))).toBeInTheDocument();
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('10000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          type: 'receipt',
          account_id: 21,
          amount: '10000',
          date: '10-10-2026',
        }),
        null,
      );
    });
    // The bill the amount settles is marked in words and in the backend's figure.
    await waitFor(async () => {
      const text = await panelText(language, 'vouchers-bills-title');
      expect(text).toContain(message(language, 'vouchers-col-settles'));
      expect(text).toContain('10,000');
      expect(text).toContain(message(language, 'vouchers-effect-part'));
    });
    // The balances under the form are the backend's.
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'vouchers-new-balance'));
    });
    expect(pageText()).toContain('2,880');
  });

  it('"full balance" and "oldest bill" put the backend\'s amount into the field', async () => {
    const { user } = await open('receipt');
    await pick(user, language, 'vouchers-field-buyer', '0', '0020');
    const full = await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'vouchers-fill-full')),
    });
    await user.click(full);
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('12,880');
    await user.clear(await labelled(language, 'vouchers-field-amount'));
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'vouchers-fill-oldest')),
      }),
    );
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('12,880');
  });

  it('more than all the bills: the extra stays as credit and the screen says so', async () => {
    const { user } = await open('receipt');
    await pick(user, language, 'vouchers-field-buyer', '0', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('15000');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'vouchers-extra-credit-note'));
    });
    expect(pageText()).toContain('2,120');
  });

  it('a receipt is saved with F10: the text sent, a message, the form ready for the next, the voucher listed', async () => {
    const { user } = await open('receipt');
    await pick(user, language, 'vouchers-field-buyer', '0', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('10000');
    await user.click(await labelled(language, 'vouchers-field-narration'));
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0];
    expect(sent?.type).toBe('receipt');
    expect(sent?.input).toMatchObject({
      account_id: 21,
      person_key: null,
      amount: '10000',
      date: '10-10-2026',
      narration: 'x',
      collector_staff_id: null,
    });
    // Nothing about how it was paid goes to the backend.
    for (const gone of ['mode', 'bank_account_id', 'cheque_no', 'cheque_date']) {
      expect(sent?.input).not.toHaveProperty(gone);
    }
    const toast = message(language, 'vouchers-saved', {
      kind: 'receipt',
      number: message(language, 'vouchers-number-receipt', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    // The form is ready for the next: account and amount empty, the date kept.
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'vouchers-field-amount')))
          .value,
      ).toBe('');
    });
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    expect(saveButton(language)).toBeDisabled();
    // The saved voucher is in the list under the form, with its number and amount.
    const list = await screen.findByRole('group', {
      name: message(language, 'vouchers-list-label'),
    });
    await waitFor(() => {
      const text = visible(list.textContent);
      expect(text).toContain(message(language, 'vouchers-number-receipt', { number: 1 }));
      expect(text).toContain('10,000');
    });
  });

  it('150 vouchers come one page at a time: 25 at once, the count and the total are the backend’s, Next page loads the next 25', async () => {
    const all = Array.from({ length: 150 }, (_, index) =>
      voucherRow({ id: 700 + index, number: index + 1 }),
    );
    vi.mocked(vouchersApi.vouchersList).mockImplementation((input) =>
      Promise.resolve({
        rows: sliceOf(all, input.offset, input.limit),
        total_count: all.length,
        // The total of the whole filter, not the sum of the rows on screen.
        totals: { cancelled_count: 0, total_paisa: money(1_500_000) },
        months: [],
      }),
    );
    const { user } = await open();
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 1, 25, 150));
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.ag-row-pinned [col-id="amount"]')?.textContent)).toBe(
        '1,500,000',
      );
    });
    expect(vouchersApi.vouchersList).toHaveBeenLastCalledWith(
      expect.objectContaining({ voucher_type: 'receipt', offset: 0, limit: PAGE_ROWS }),
    );
    await user.click(await pagerButton(language, 'next'));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(rangeText(language, 26, 50, 150));
    });
    expect(vouchersApi.vouchersList).toHaveBeenLastCalledWith(
      expect.objectContaining({ offset: PAGE_ROWS, limit: PAGE_ROWS }),
    );
  });

  it('a payment to a seller shows what is payable with the advance already adjusted', async () => {
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', 'rahim', '0010');
    const text = await panelText(language, 'vouchers-payables-title');
    expect(text).toContain('24,316');
    expect(text).toContain('15,000');
    expect(text).toContain('4,316');
    const full = await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'vouchers-fill-full')),
    });
    await user.click(full);
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('4,316');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'payment',
      input: { account_id: 31, amount: '4,316', reason: '', confirm_cash_below_zero: false },
    });
  });

  it('more than is payable only warns: the extra shows in words and the save still goes', async () => {
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', 'rahim', '0010');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-payment-above-payable', { payable: '4,316', over: '684' }),
      );
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
  });

  it('a payment to a seller with nothing payable offers a پیشگی; only the button switches', async () => {
    store = fakeVouchers({ karimPayable: 0 });
    const { user, router } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', 'karim', '0011');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    const hint = message(language, 'vouchers-advance-hint');
    await waitFor(() => {
      expect(pageText()).toContain(hint);
    });
    // Nothing switched by itself.
    expect(router.state.location.search).toMatchObject({ type: 'payment' });
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'vouchers-make-advance')) }),
    );
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-advance-title'),
    });
    // The seller, the amount and the date stay.
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('3,000');
    expect((await dropdown(language, 'vouchers-field-seller')) as HTMLInputElement).toHaveValue(
      language === 'ur' ? 'کریم' : 'Karim',
    );
    expect(pageText()).not.toContain(hint);
  });

  it('a payment to a buyer is cash lent: the reason is asked, told at its field, and sent', async () => {
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', '0020', '0020');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    const reason = await labelled(language, 'vouchers-field-reason-lent');
    await waitFor(() => {
      expect(reason).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-payment-to-buyer-needs-reason'));
    await user.click(reason);
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'payment',
      input: { account_id: 21, amount: '3000', reason: 'x' },
    });
  });

  it('a seller returning advance money: the receipt takes the seller side and shows his advances', async () => {
    const { user } = await open('receipt');
    await user.click(
      screen.getByRole('button', { name: message(language, 'vouchers-from-seller') }),
    );
    await pick(user, language, 'vouchers-field-seller', 'rahim', '0010');
    const text = await panelText(language, 'vouchers-advances-title');
    expect(text).toContain('15,000');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('10000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ type: 'receipt', account_id: 31, amount: '10000' }),
        null,
      );
    });
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'vouchers-return-limit'));
    });
  });

  it('an advance goes with the expected goods; cash below zero shows the note and asks before saving', async () => {
    store = fakeVouchers({ cash: 20000 });
    const { user } = await open('advance');
    await pick(user, language, 'vouchers-field-seller', 'rahim', '0010');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('25000');
    await user.click(await labelled(language, 'vouchers-field-expected'));
    await user.keyboard('x');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-5,000' }));
    });
    press('F10');
    // The box asks, with focus on "No": nothing is saved.
    const no = await screen.findByRole('button', {
      name: new RegExp(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    expect(store.rows).toHaveLength(0);
    await userEvent.setup().click(no);
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(store.rows).toHaveLength(0);
    // Yes saves with the confirmation.
    press('F10');
    await userEvent
      .setup()
      .click(
        await screen.findByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
      );
    await waitFor(() => {
      expect(store.rows).toHaveLength(1);
    });
    // The save that went through carried the confirmation.
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls.at(-1)?.[0]).toMatchObject({
      type: 'advance',
      input: {
        seller_account_id: 31,
        amount: '25000',
        expected_goods: 'x',
        confirm_cash_below_zero: true,
      },
    });
  });

  it('changing the type keeps the date, amount and narration and clears the rest', async () => {
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', 'rahim', '0010');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('3000');
    await user.click(await labelled(language, 'vouchers-field-narration'));
    await user.keyboard('n');
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-type') });
    await user.click(
      within(group).getByRole('button', { name: message(language, 'vouchers-type-receipt') }),
    );
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-receipt-title'),
    });
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('3,000');
    expect((await labelled(language, 'vouchers-field-narration')).value).toBe('n');
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    // The account of the other type is gone.
    expect(await dropdown(language, 'vouchers-field-buyer')).toHaveValue('');
  });

  it('an advance goes to the backend with its date, amount and seller only, and Enter walks the fields', async () => {
    const { user } = await open('advance');
    await pick(user, language, 'vouchers-field-seller', 'rahim', '0010');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('100');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0];
    expect(sent).toMatchObject({
      type: 'advance',
      input: { seller_account_id: RAHIM.id, amount: '100', date: '10-10-2026' },
    });
    for (const gone of ['mode', 'bank_account_id', 'cheque_no', 'cheque_date']) {
      expect(sent?.input).not.toHaveProperty(gone);
    }
  });

  it('a refused field shows at the field, and nothing typed is lost', async () => {
    const { user } = await open('advance');
    await pick(user, language, 'vouchers-field-seller', 'rahim', '0010');
    const amount = await labelled(language, 'vouchers-field-amount');
    await user.click(amount);
    await user.keyboard('5000');
    vi.mocked(vouchersApi.vouchersSave).mockRejectedValueOnce(
      (await import('../helpers/app')).fail({
        code: 'ERR_DATE_CLOSED',
        field: 'date',
        values: { closed_up_to: '09-10-2026' },
      }),
    );
    press('F10');
    const date = await labelled(language, 'vouchers-field-date');
    await waitFor(() => {
      expect(date).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(
      message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }),
    );
    expect(amount.value).toBe('5,000');
  });

  it('an amount with no account is told at the account field, and the backend is not asked to save', async () => {
    const { user } = await open('receipt');
    const buyer = await dropdown(language, 'vouchers-field-buyer');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5000');
    press('F10');
    await waitFor(() => {
      expect(buyer).toHaveAttribute('aria-invalid', 'true');
    });
    expect(vouchersApi.vouchersSave).not.toHaveBeenCalled();
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('5,000');
  });

  it('switching the language in the middle of an entry keeps what was typed', async () => {
    const { user } = await open('payment');
    await pick(user, language, 'vouchers-field-paid-to', 'rahim', '0010');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('300');
    await user.click(await labelled(language, 'vouchers-field-narration'));
    await user.keyboard('n');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'vouchers-field-amount')).value).toBe('300');
    expect((await labelled(other, 'vouchers-field-narration')).value).toBe('n');
    expect(await dropdown(other, 'vouchers-field-paid-to')).toHaveValue(
      other === 'ur' ? 'رحیم' : 'Rahim',
    );
    const group = screen.getByRole('group', { name: message(other, 'vouchers-field-type') });
    expect(
      within(group).getByRole('button', { name: message(other, 'vouchers-type-payment') }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('a failed load of today shows the message with a retry that works', async () => {
    const appApi = await import('../../src/app/api');
    const { fail } = await import('../helpers/app');
    vi.mocked(appApi.dayStatus).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp('/bikri/voucher?type=advance');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'vouchers-advance-title'),
      }),
    ).toBeInTheDocument();
  });

  it('an open-bills panel that fails says so with a retry', async () => {
    const { fail } = await import('../helpers/app');
    const { user } = await open('receipt');
    vi.mocked(vouchersApi.assistOpenBills).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await pick(user, language, 'vouchers-field-buyer', '0', '0020');
    await screen.findByText(message(language, 'vouchers-bills-failed'));
    const failedPanel = (
      await screen.findByText(message(language, 'vouchers-bills-failed'))
    ).closest('section');
    if (failedPanel === null) throw new Error('the failed panel is not a section');
    await user.click(
      within(failedPanel).getByRole('button', { name: message(language, 'common-retry') }),
    );
    expect(await panelText(language, 'vouchers-bills-title')).toContain('12,880');
  });
});
