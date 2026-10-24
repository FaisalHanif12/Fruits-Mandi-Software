// The waiting list of the voucher screen (PRD FR-15.1): several vouchers entered one after another
// and saved together. F2 puts the form in the list and starts the next, F10 saves the list one
// voucher at a time (each its own save), a voucher the backend refuses stays with its reason, and
// cash below zero asks for each voucher that needs it and never sends the confirmation unasked.
// In the Urdu and in the English app, by keyboard, against a fake backend. The screen adds no
// money: the amounts, the balances and the total are the fake backend's.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import {
  AKRAM,
  ARSHAD,
  KARIM,
  PUKHTA_BUYER,
  RAHIM,
  fakeVouchers,
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
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));
const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });
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
  await waitFor(() => {
    const options = screen.getAllByRole('option');
    expect(options).toHaveLength(1);
    expect(visible(options[0]?.textContent)).toContain(shown);
  });
  await user.keyboard('{Enter}');
}

/** A voucher typed on the form: the account (what is typed, and what the picker then shows), then the amount. */
async function typeVoucher(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  field: string,
  typed: string,
  amount: string,
  shown = typed,
) {
  await pick(user, language, field, typed, shown);
  await user.click(await labelled(language, 'vouchers-field-amount'));
  await user.keyboard(amount);
}

/** The waiting list card, found by its title. */
async function waitingCard(language: Language): Promise<HTMLElement> {
  const title = await screen.findByRole('heading', {
    name: named(message(language, 'vouchers-waiting-title')),
  });
  const card = title.closest('section');
  if (card === null) throw new Error('the waiting list is not a card');
  return card;
}

const noWaitingList = (language: Language) =>
  screen.queryByRole('heading', { name: named(message(language, 'vouchers-waiting-title')) });

/** The amounts the save commands were called with, in order. */
const savedAmounts = () =>
  vi.mocked(vouchersApi.vouchersSave).mock.calls.map((call) => call[0].input.amount);

let store: FakeVouchers;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, PUKHTA_BUYER, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the waiting list of vouchers (%s)', (language) => {
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

  it('F2 puts the form in the list and starts the next: the type and date stay, the rest starts again', async () => {
    const { user } = await open('receipt');
    expect(noWaitingList(language)).toBeNull();
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenCalled();
    });
    press('F2');
    const card = await waitingCard(language);
    // The row shows the account, and the amount and balance after come from the backend's preview.
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain('03040001');
      expect(text).toContain('10,000');
      expect(text).toContain('2,880');
      expect(text).toContain(message(language, 'vouchers-state-waiting'));
    });
    // The count is at the top, by the buttons, so it is seen even when the list is below the fold.
    expect(pageText()).toContain(message(language, 'vouchers-waiting-count', { count: 1 }));
    // Nothing was saved. The form is ready for the next voucher, with the cursor on the account.
    expect(vouchersApi.vouchersSave).not.toHaveBeenCalled();
    expect((await labelled(language, 'vouchers-field-amount')).value).toBe('');
    expect((await labelled(language, 'vouchers-field-date')).value).toBe('10-10-2026');
    expect(await dropdown(language, 'vouchers-field-buyer')).toHaveValue('');
    await waitFor(async () => {
      expect(await dropdown(language, 'vouchers-field-buyer')).toHaveFocus();
    });
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-type') });
    expect(
      within(group).getByRole('button', { name: message(language, 'vouchers-type-receipt') }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('F10 saves the list in order, each voucher with its own save; the form joins it; saved ones leave and the day total shows', async () => {
    const { user } = await open('receipt');
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    press('F2');
    await typeVoucher(user, language, 'vouchers-field-buyer', '2', '5000');
    press('F2');
    // The third is still on the form when F10 is pressed: it joins the list.
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '2880');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(3);
    });
    expect(savedAmounts()).toEqual(['10000', '5000', '2880']);
    expect(
      vi.mocked(vouchersApi.vouchersSave).mock.calls.map((call) => call[0].input),
    ).toMatchObject([{ account_id: 21 }, { account_id: 22 }, { account_id: 21 }]);
    expect(store.rows).toHaveLength(3);
    // One message for the whole run, and nothing is left waiting.
    const toast = message(language, 'vouchers-saved-many', { count: 3 });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    await waitFor(() => {
      expect(noWaitingList(language)).toBeNull();
    });
    // The vouchers are in the day's list under the form, with the backend's total.
    const list = await screen.findByRole('group', {
      name: message(language, 'vouchers-list-label'),
    });
    await waitFor(() => {
      const text = visible(list.textContent);
      expect(text).toContain(message(language, 'vouchers-number-receipt', { number: 3 }));
      expect(text).toContain('17,880');
    });
  });

  it('a voucher the backend refuses stays in the list with its reason while the others are saved', async () => {
    const { user } = await open('receipt');
    const original = vi.mocked(vouchersApi.vouchersSave).getMockImplementation();
    vi.mocked(vouchersApi.vouchersSave).mockImplementation((built) =>
      built.input.amount === '5000'
        ? Promise.reject(fail({ code: 'ERR_VOUCHER_CANCELLED', field: 'amount' }))
        : (original?.(built) ?? Promise.reject(new Error('no fake'))),
    );
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    press('F2');
    await typeVoucher(user, language, 'vouchers-field-buyer', '2', '5000');
    press('F2');
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '2880');
    press('F10');
    await waitFor(() => {
      expect(store.rows).toHaveLength(2);
    });
    // The refused one is the only row left, with the reason in words and no code on screen.
    const card = await waitingCard(language);
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain(message(language, 'err-voucher-cancelled'));
      expect(text).toContain('03040002');
      expect(text).not.toContain('ERR_');
    });
    expect(within(card).getAllByRole('row')).toHaveLength(2);
    const refusedToast = message(language, 'vouchers-some-refused', { count: 1 });
    expect(
      (await screen.findAllByText((content) => visible(content) === refusedToast)).length,
    ).toBeGreaterThan(0);
    // Saving again tries only what is left, and this time the backend accepts it.
    vi.mocked(vouchersApi.vouchersSave).mockImplementation(
      original ?? (() => Promise.reject(new Error('x'))),
    );
    press('F10');
    await waitFor(() => {
      expect(store.rows).toHaveLength(3);
    });
    await waitFor(() => {
      expect(noWaitingList(language)).toBeNull();
    });
  });

  it('cash below zero asks for the voucher that needs it, with the cash after that entry: No keeps it and the run goes on, Yes saves it', async () => {
    store = fakeVouchers({ cash: 20000 });
    const { user } = await open('advance');
    await typeVoucher(user, language, 'vouchers-field-seller', 'rahim', '15000', '030110001');
    press('F2');
    await typeVoucher(user, language, 'vouchers-field-seller', 'karim', '10000', '030110002');
    press('F2');
    await typeVoucher(user, language, 'vouchers-field-seller', 'rahim', '1000', '030110001');
    press('F10');
    // The first voucher is saved on its own (cash 5,000). The second would take cash to -5,000:
    // the run stops there and asks, with focus on "No".
    const no = await screen.findByRole('button', {
      name: new RegExp(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    expect(pageText()).toContain('-5,000');
    expect(store.rows).toHaveLength(1);
    await user.click(no);
    // No keeps that voucher in the list, marked, and the run goes on with the third.
    await waitFor(() => {
      expect(store.rows).toHaveLength(2);
    });
    const card = await waitingCard(language);
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain('030110002');
      expect(text).toContain(message(language, 'vouchers-state-declined'));
    });
    expect(within(card).getAllByRole('row')).toHaveLength(2);
    // The confirmation was never sent: nobody said yes to it.
    expect(
      vi
        .mocked(vouchersApi.vouchersSave)
        .mock.calls.some(
          (call) => call[0].type === 'advance' && call[0].input.confirm_cash_below_zero,
        ),
    ).toBe(false);
    // Saving again asks again; Yes saves it, with the confirmation this time.
    press('F10');
    await user.click(
      await screen.findByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(store.rows).toHaveLength(3);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls.at(-1)?.[0]).toMatchObject({
      type: 'advance',
      input: { seller_account_id: 32, amount: '10000', confirm_cash_below_zero: true },
    });
    await waitFor(() => {
      expect(noWaitingList(language)).toBeNull();
    });
  });

  it('vouchers of different types wait in one list, each saved as its own type', async () => {
    const { user } = await open('receipt');
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    press('F2');
    const group = screen.getByRole('group', { name: message(language, 'vouchers-field-type') });
    await user.click(
      within(group).getByRole('button', { name: message(language, 'vouchers-type-payment') }),
    );
    await typeVoucher(user, language, 'vouchers-field-paid-to', 'rahim', '4316', '030110001');
    press('F2');
    // The list keeps both, with the type of each.
    const card = await waitingCard(language);
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain(message(language, 'vouchers-type-receipt'));
      expect(text).toContain(message(language, 'vouchers-type-payment'));
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(2);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls.map((call) => call[0].type)).toEqual([
      'receipt',
      'payment',
    ]);
  });

  it('an amount with no account, or an account with no amount, is told at its field and not listed', async () => {
    const { user } = await open('receipt');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('10000');
    press('F2');
    expect(await screen.findByText(message(language, 'err-required'))).toBeInTheDocument();
    expect(noWaitingList(language)).toBeNull();
    await user.clear(await labelled(language, 'vouchers-field-amount'));
    await pick(user, language, 'vouchers-field-buyer', '1', '03040001');
    press('F2');
    expect(await screen.findByText(message(language, 'err-required'))).toBeInTheDocument();
    expect(noWaitingList(language)).toBeNull();
    expect(vouchersApi.vouchersSave).not.toHaveBeenCalled();
  });

  it('a voucher can be removed from the list before it is saved', async () => {
    const { user } = await open('receipt');
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    press('F2');
    const card = await waitingCard(language);
    await user.click(
      within(card).getByRole('button', { name: message(language, 'vouchers-remove') }),
    );
    await waitFor(() => {
      expect(noWaitingList(language)).toBeNull();
    });
    expect(vouchersApi.vouchersSave).not.toHaveBeenCalled();
  });

  it('switching the language keeps the list and its vouchers; the names follow the language', async () => {
    const { user } = await open('receipt');
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    press('F2');
    await typeVoucher(user, language, 'vouchers-field-buyer', '2', '5000');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const card = await waitingCard(other);
    await waitFor(() => {
      expect(visible(card.textContent)).toContain(
        other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
      );
    });
    // What was typed on the form stays too, and F10 still saves both.
    expect((await labelled(other, 'vouchers-field-amount')).value).toBe('5,000');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(2);
    });
    expect(savedAmounts()).toEqual(['10000', '5000']);
  });

  it('leaving with vouchers waiting asks first', async () => {
    const { user } = await open('receipt');
    await typeVoucher(user, language, 'vouchers-field-buyer', '1', '10000');
    press('F2');
    await waitingCard(language);
    press('Escape');
    expect(
      await screen.findByRole('button', {
        name: new RegExp(message(language, 'layout-save-leave')),
      }),
    ).toBeInTheDocument();
    expect(user).toBeDefined();
  });

  it('while a saved voucher is changed there is no list, and F2 starts a new voucher', async () => {
    store.rows.push({
      id: 501,
      voucher_type: 'receipt',
      number: 1,
      entry_date: '2026-10-10',
      account: { id: 21, code: '03040001', name_ur: AKRAM.name_ur, name_en: AKRAM.name_en },
      amount_paisa: store.rows[0]?.amount_paisa ?? (1_000_000 as never),
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
    });
    await renderApp('/bikri/voucher?type=receipt&id=501');
    await screen.findByRole('heading', { level: 1 });
    const newEntry = await screen.findByRole('button', {
      name: new RegExp(message(language, 'vouchers-new')),
    });
    expect(newEntry).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'vouchers-add-to-list')) }),
    ).toBeNull();
    expect(noWaitingList(language)).toBeNull();
  });
});
