// A NEW entry on a closed day, on the other entry screens (plan 85, PRD FR-21.4 v1.49, FR-6.6), in the Urdu and
// the English app. One shared piece (`ClosedDayEntryNotice`, shared/edit) serves them all: the voucher screen has
// its own save flow, and the screens of the روکڑ area (خرچہ, ادھار رقم, الگ کھاتہ, سرمایہ, پھل, جنرل) share
// `EntryFrame`. Here the voucher screen and the خرچہ screen stand for them. The backend refuses an entry whose
// date is a closed day (`ERR_DATE_CLOSED` on `date`): the screen shows the notice with "Reopen the day and save",
// the box asks the reason and the Owner's sign, the day reopens (the date goes as typed, DD-MM-YYYY), the old
// refusal is gone, the save posts, and the days close again with the token of the reopen.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as reportsApi from '../../src/features/reports/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as editApi from '../../src/shared/edit/api';
import { useFixStore } from '../../src/shared/edit/fix-store';
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
import { HEADS } from '../helpers/cashbook-books';
import { chooseRecord, named, prepareCashbook, typeAmount } from '../helpers/cashbook-screen';
import { DAY_FIX_CLOSE_E2E_45, DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import { confirmFixDay } from '../helpers/fix-day';
import { fakeMasters } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';
import { AKRAM, ARSHAD, CHUKTA_KHATA, KARIM, RAHIM, fakeVouchers } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

/** The day of the entries in these tests: today of the fake day status, as typed. */
const DAY = '10-10-2026';

/** True from the moment the double has opened the day. */
let opened = false;

/** The backend's refusal of a closed day, at the field of the date. */
const closedDay = () =>
  new CommandError(
    refusal([
      {
        code: 'ERR_DATE_CLOSED',
        field: 'date',
        values: { closed_up_to: '10-10-2026', today: '10-10-2026', today_open: 0 },
      },
    ]),
  );

beforeEach(() => {
  vi.resetAllMocks();
  opened = false;
  useFixStore.getState().end();
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
  vi.mocked(editApi.dayFixOpen).mockImplementation(() => {
    opened = true;
    return Promise.resolve(DAY_FIX_OPEN_E2E_45);
  });
  vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_E2E_45);
});

afterEach(() => {
  leaveAppWindow();
  useFixStore.getState().end();
});

const alertsText = () =>
  screen.queryAllByRole('alert').map((element) => visible(element.textContent));

describe.each(LANGUAGES)('a new voucher on a closed day (%s)', (language) => {
  beforeEach(() => {
    healthyBackend();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
    vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
    vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
    const rows = [AKRAM, ARSHAD, CHUKTA_KHATA, RAHIM, KARIM];
    fakeAccounts(rows);
    fakeAccountSearch(rows);
    fakeMasters({});
    fakeVouchers();
    // While the day is closed the backend refuses the entry: the preview finds it, and the save is refused.
    const preview = vi.mocked(vouchersApi.vouchersPreview).getMockImplementation();
    const save = vi.mocked(vouchersApi.vouchersSave).getMockImplementation();
    if (preview === undefined || save === undefined) throw new Error('no voucher fakes');
    vi.mocked(vouchersApi.vouchersPreview).mockImplementation(async (...args) => {
      const reply = await preview(...args);
      return opened
        ? reply
        : { ...reply, errors: [{ code: 'ERR_DATE_CLOSED', field: 'date', values: {} }] };
    });
    vi.mocked(vouchersApi.vouchersSave).mockImplementation((...args) =>
      opened ? save(...args) : Promise.reject(closedDay()),
    );
  });

  const closedWords = () => message(language, 'edit-closed-entry', { date: DAY });
  const fixButton = () =>
    screen.findByRole('button', { name: named(message(language, 'edit-action-fix-save')) });

  /** Opens the receipt screen and types a receipt from اکرم of 10,000. */
  async function typeReceipt() {
    await renderApp('/bikri/voucher?type=receipt');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'vouchers-receipt-title'),
    });
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      }),
    );
    await user.keyboard('0');
    await waitFor(() => {
      const options = screen.getAllByRole('option');
      expect(options).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    await user.click(
      await screen.findByLabelText<HTMLInputElement>(
        named(message(language, 'vouchers-field-amount')),
      ),
    );
    await user.keyboard('10000');
    return user;
  }

  it('shows the notice and the button, reopens with the reason and the sign, and the save closes the days again', async () => {
    const user = await typeReceipt();
    // The preview says the day is closed: the notice with the button.
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(true);
    });
    await user.click(await fixButton());
    await confirmFixDay(language, user, DAY);
    // The days are open: the red notice is gone, and the screen asked its answer again.
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(false);
    });
    expect(vouchersApi.vouchersSave).not.toHaveBeenCalled();
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0].input).toMatchObject({
      account_id: 21,
      amount: '10000',
      date: DAY,
    });
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
    expect(editApi.dayFixClose).toHaveBeenCalledWith(DAY_FIX_OPEN_E2E_45.token);
  });

  it('a save tried on the closed day is refused, shows the same notice, and nothing is saved', async () => {
    await typeReceipt();
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(true);
    });
    expect(await fixButton()).toBeInTheDocument();
    expect(editApi.dayFixOpen).not.toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)('a new خرچہ on a closed day (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
    const preview = vi.mocked(cashbookApi.expensePreview).getMockImplementation();
    const save = vi.mocked(cashbookApi.saveShopExpense).getMockImplementation();
    if (preview === undefined || save === undefined) throw new Error('no cashbook fakes');
    vi.mocked(cashbookApi.expensePreview).mockImplementation(async (...args) => {
      const reply = await preview(...args);
      return opened
        ? reply
        : { ...reply, errors: [{ code: 'ERR_DATE_CLOSED', field: 'date', values: {} }] };
    });
    vi.mocked(cashbookApi.saveShopExpense).mockImplementation((...args) =>
      opened ? save(...args) : Promise.reject(closedDay()),
    );
  });

  const closedWords = () => message(language, 'edit-closed-entry', { date: DAY });
  const head = HEADS[0];

  it('the shared frame shows the notice; the reopen and the save close the days again', async () => {
    if (head === undefined) throw new Error('no head');
    await renderApp('/rokar/expense?type=shop_expense');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    const user = userEvent.setup();
    await chooseRecord(
      user,
      language,
      'cashbook-field-head',
      language === 'ur' ? head.name_ur : head.name_en,
    );
    await typeAmount(user, language, '3000');
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(true);
    });
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'edit-action-fix-save')) }),
    );
    await confirmFixDay(language, user, DAY);
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(false);
    });
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveShopExpense).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
    expect(editApi.dayFixClose).toHaveBeenCalledWith(DAY_FIX_OPEN_E2E_45.token);
  });
});
