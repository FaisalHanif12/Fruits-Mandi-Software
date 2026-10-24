// The main task of each entry screen, done with the keyboard only (Code Standards F30, NFR-USE-7;
// D1+D2 review, finding 34). Nothing here clicks: focus starts where the screen puts it, Tab
// moves between fields, and the text, Enter and F10 do the rest. In the Urdu and in the English
// app, against a fake backend. Screens covered: a new account, a new rule, a receipt, several receipts in a
// list, a payment, an advance, the firm in Settings and the opening balances grid. The login, the wizard, the day close, the master
// lists and change password have the same kind of test in their own files.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as rulesApi from '../../src/features/rules/api';
import * as settingsApi from '../../src/features/settings/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts, fakeOpenings, money } from '../helpers/accounts';
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
import { message, visible } from '../helpers/render';
import { fakeRules } from '../helpers/rules';
import { suggestionsFor } from '../helpers/translit';
import { fakeAdvances } from '../helpers/vouchers';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/rules/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  opening_date: null,
});
const ARSHAD = account(22, 'buyer', '03040002', 'ارشد ملتانی', 'Arshad Multani', {
  opening_date: null,
});
const CASH = account(51, 'other', '01010001', 'روکڑ', 'Cash in Hand', {
  opening_date: '2026-08-31',
  opening_dr_paisa: money(65000),
});
const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim');

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.getByLabelText<HTMLInputElement>(named(message(language, id)));
const saveButton = (language: Language) =>
  screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) });

/** Tab until `target()` has focus. The key a person would press, nothing else. */
async function tabTo(
  user: ReturnType<typeof userEvent.setup>,
  target: () => Element,
  what: string,
) {
  const LIMIT = 60;
  for (let presses = 0; presses < LIMIT; presses += 1) {
    if (document.activeElement === target()) return;
    await user.tab();
  }
  throw new Error(`Tab did not reach ${what} in ${String(LIMIT)} presses`);
}

function fakeTranslit() {
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { tariq: ['طارق'], mehmood: ['محمود'] },
          toEnglish: { طارق: 'Tariq', محمود: 'Mehmood' },
          rulesOnly: [],
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeTranslit();
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('main tasks by keyboard only (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('a new account: Tab to the name, type it, F10 saves, and the list is back', async () => {
    fakeAccounts([AKRAM, ARSHAD]);
    const { router } = await renderApp('/index/accounts/new');
    await waitFor(() => {
      expect(labelled(language, 'accounts-field-code').value).toBe('03040003');
    });
    const user = userEvent.setup();
    const nameId = language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en';
    await tabTo(user, () => labelled(language, nameId), 'the name');
    await user.keyboard(language === 'ur' ? 'tariq mehmood{Enter}' : 'Tariq Mehmood');
    await waitFor(() => {
      expect(labelled(language, 'accounts-field-name-ur').value).toBe('طارق محمود');
      expect(labelled(language, 'accounts-field-name-en').value).toBe('Tariq Mehmood');
    });
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      code: '03040003',
      names: { urdu: 'طارق محمود', english: 'Tariq Mehmood' },
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/accounts');
    });
  });

  it('a new rule: Tab to the value, type it, F10 saves, and the list is back', async () => {
    fakeMasters();
    fakeAccounts([AKRAM, RAHIM]);
    fakeAccountSearch([AKRAM, RAHIM]);
    fakeRules();
    const { router } = await renderApp('/index/rules/new');
    await screen.findByRole('heading', { level: 1, name: message(language, 'rules-new-title') });
    const user = userEvent.setup();
    await tabTo(user, () => labelled(language, 'rules-field-value'), 'the value');
    await user.keyboard('2.5');
    press('F10');
    await waitFor(() => {
      expect(rulesApi.rulesSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(rulesApi.rulesSave).mock.calls[0]?.[0]).toMatchObject({
      side: 'seller',
      charge_type: 'commission',
      method: 'percent',
      value: '2.5',
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules');
    });
  });

  it('an advance: Tab to the seller, type and Enter, then the amount, F10 saves', async () => {
    fakeAccounts([RAHIM]);
    fakeAccountSearch([RAHIM]);
    fakeMasters({});
    const store = fakeAdvances();
    await renderApp('/bikri/advance');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-advance-title'),
    });
    const user = userEvent.setup();
    const seller = () =>
      screen.getByRole('combobox', { name: named(message(language, 'vouchers-field-seller')) });
    await tabTo(user, seller, 'the seller');
    await user.keyboard('rahim');
    await waitFor(() => {
      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(1);
      expect(visible(options[0]?.textContent)).toContain('030110001');
    });
    await user.keyboard('{Enter}');
    await tabTo(user, () => labelled(language, 'vouchers-field-amount'), 'the amount');
    await user.keyboard('5000');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'advance',
      input: { amount: '5000' },
    });
    expect(store.saved).toEqual([{ seller: 31, amount: '5000' }]);
  });

  it('a receipt: Tab to the buyer, type and Enter, then the amount, F10 saves; the list gets its row', async () => {
    fakeAccounts([AKRAM, RAHIM]);
    fakeAccountSearch([AKRAM, RAHIM]);
    fakeMasters({});
    const store = fakeAdvances();
    await renderApp('/bikri/receipt');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-receipt-title')),
    });
    const user = userEvent.setup();
    const buyer = () =>
      screen.getByRole('combobox', { name: named(message(language, 'vouchers-field-buyer')) });
    await tabTo(user, buyer, 'the buyer');
    await user.keyboard('akram');
    await waitFor(() => {
      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(1);
      expect(visible(options[0]?.textContent)).toContain('03040001');
    });
    await user.keyboard('{Enter}');
    await tabTo(user, () => labelled(language, 'vouchers-field-amount'), 'the amount');
    await user.keyboard('10000');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'receipt',
      input: { account_id: 21, amount: '10000' },
    });
    expect(store.rows).toHaveLength(1);
    // The next voucher starts at the buyer: the cursor is in the field that always changes.
    await waitFor(() => {
      expect(document.activeElement).toBe(buyer());
    });
  });

  it('several receipts: type one, F2 puts it in the list, type the next, F10 saves them all', async () => {
    fakeAccounts([AKRAM, ARSHAD, RAHIM]);
    fakeAccountSearch([AKRAM, ARSHAD, RAHIM]);
    fakeMasters({});
    const store = fakeAdvances();
    await renderApp('/bikri/receipt');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-receipt-title')),
    });
    const user = userEvent.setup();
    const buyer = () =>
      screen.getByRole('combobox', { name: named(message(language, 'vouchers-field-buyer')) });
    const typeBuyer = async (typed: string, code: string) => {
      await user.keyboard(typed);
      await waitFor(() => {
        const options = screen.getAllByRole('option');
        expect(options).toHaveLength(1);
        expect(visible(options[0]?.textContent)).toContain(code);
      });
      await user.keyboard('{Enter}');
    };
    await tabTo(user, buyer, 'the buyer');
    await typeBuyer('akram', '03040001');
    await tabTo(user, () => labelled(language, 'vouchers-field-amount'), 'the amount');
    await user.keyboard('10000');
    press('F2');
    // The cursor is back on the buyer of the next voucher: no mouse and no extra Tab.
    await waitFor(() => {
      expect(document.activeElement).toBe(buyer());
    });
    await typeBuyer('arshad', '03040002');
    await tabTo(user, () => labelled(language, 'vouchers-field-amount'), 'the amount');
    await user.keyboard('5000');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(2);
    });
    const sent = vi.mocked(vouchersApi.vouchersSave).mock.calls.map((call) => call[0]);
    expect(sent).toMatchObject([
      { type: 'receipt', input: { account_id: 21, amount: '10000' } },
      { type: 'receipt', input: { account_id: 22, amount: '5000' } },
    ]);
    expect(store.rows).toHaveLength(2);
  });

  it('a payment: Tab to the account, type and Enter, then the amount, F10 saves', async () => {
    fakeAccounts([AKRAM, RAHIM]);
    fakeAccountSearch([AKRAM, RAHIM]);
    fakeMasters({});
    fakeAdvances();
    await renderApp('/bikri/payment');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'vouchers-payment-title')),
    });
    const user = userEvent.setup();
    const paidTo = () =>
      screen.getByRole('combobox', { name: named(message(language, 'vouchers-field-paid-to')) });
    await tabTo(user, paidTo, 'the account');
    await user.keyboard('rahim');
    await waitFor(() => {
      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(1);
      expect(visible(options[0]?.textContent)).toContain('030110001');
    });
    await user.keyboard('{Enter}');
    await tabTo(user, () => labelled(language, 'vouchers-field-amount'), 'the amount');
    await user.keyboard('4316');
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'payment',
      input: { account_id: 31, amount: '4316' },
    });
  });

  it('the firm in Settings: Tab to the address, type, F10 saves', async () => {
    vi.mocked(settingsApi.settingsSetFirm).mockImplementation((input) => Promise.resolve(input));
    await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    const user = userEvent.setup();
    await tabTo(user, () => labelled(language, 'settings-firm-address'), 'the address');
    // Entering a field selects its whole value, so typing replaces it (PRD FR-7.15).
    await user.keyboard('Shop 2');
    expect(saveButton(language)).toBeEnabled();
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledWith(
        expect.objectContaining({ address: 'Shop 2' }),
      );
    });
    expect(await screen.findByText(message(language, 'settings-saved'))).toBeInTheDocument();
  });

  it('the opening balances: Tab into the grid, arrows to the row, Enter and the amount, F10 saves', async () => {
    const store = fakeAccounts([CASH, AKRAM, ARSHAD]);
    fakeOpenings(store);
    await renderApp('/index/openings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'openings-title') });
    await waitFor(() => {
      expect(document.querySelectorAll('.ag-row[row-index]')).toHaveLength(3);
    });
    const user = userEvent.setup();
    const inGrid = () => document.activeElement?.closest('.ag-cell') ?? null;
    for (let presses = 0; presses < 60 && inGrid() === null; presses += 1) await user.tab();
    expect(inGrid()).not.toBeNull();
    // Focus is on the code of the first account. Down to Akram, then along the row to the date,
    // in the direction the user sees: left in the Urdu layout, right in the English one.
    const colOf = () => inGrid()?.getAttribute('col-id');
    const rowOf = () => inGrid()?.closest('.ag-row')?.getAttribute('row-index');
    const along = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
    await user.keyboard(`{ArrowDown}${along}${along}`);
    expect(rowOf()).toBe('1');
    expect(colOf()).toBe('opening_date');
    await user.keyboard('{Enter}');
    expect(colOf()).toBe('opening_dr');
    await user.keyboard('5000{Enter}');
    await waitFor(() => {
      expect(saveButton(language)).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsOpeningsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsOpeningsSave).mock.calls[0]?.[0].lines).toHaveLength(1);
  });
});
