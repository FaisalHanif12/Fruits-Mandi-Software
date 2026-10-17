// One-time people on the voucher screen (plan 15, PRD FR-13.5, FR-13.7, FR-13.8): a receipt from the
// one-time buyer account names the person whose open lines it is for, picked from the known people;
// the open bills are his own; the save carries his key; and "make regular" gives him an account and
// moves his balance to it. In the Urdu and in the English app, against a fake backend.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { NASIR, SHAHID } from '../helpers/bikri-saved';
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
import { AKRAM, KARIM, PUKHTA_BUYER, RAHIM, fakeVouchers } from '../helpers/vouchers';

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
const pageText = () => visible(document.body.textContent);
const PEOPLE = [SHAHID, NASIR];

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  vi.mocked(pukhtaApi.searchPukhtaPeople).mockImplementation((text) => {
    const needle = text.toLowerCase();
    return Promise.resolve(
      PEOPLE.filter((person) =>
        [person.name_ur, person.name_en.toLowerCase(), person.phone].some((part) =>
          part.includes(needle),
        ),
      ),
    );
  });
  const rows = [AKRAM, PUKHTA_BUYER, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('one-time people on the voucher screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
  });

  async function atTheOneTimeBuyer() {
    const app = await renderApp('/bikri/voucher?type=receipt');
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
    await user.keyboard('03049999');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    return { ...app, user };
  }

  it('the one-time account asks for the person; without him the field tells it', async () => {
    const { user } = await atTheOneTimeBuyer();
    const person = await labelled(language, 'vouchers-field-person');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('1000');
    await waitFor(() => {
      expect(person).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-person-required'));
    // Nothing is asked of the bills until a person is chosen: his money is kept per person.
    expect(vouchersApi.assistOpenBills).not.toHaveBeenCalled();
  });

  it('a name typed and Enter takes the first match; his key goes to the bills, the preview and the save', async () => {
    const { user } = await atTheOneTimeBuyer();
    const person = await labelled(language, 'vouchers-field-person');
    await user.click(person);
    await user.keyboard(language === 'ur' ? 'شاہ' : 'shah');
    await screen.findByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(vouchersApi.assistOpenBills).toHaveBeenCalledWith({
        account_id: 23,
        person_key: 'shahid|03000000000',
      });
    });
    expect(person.value).toBe(language === 'ur' ? 'شاہد' : 'Shahid');
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('1000');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          type: 'receipt',
          account_id: 23,
          person_key: 'shahid|03000000000',
        }),
        null,
      );
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersSave).mock.calls[0]?.[0]).toMatchObject({
      type: 'receipt',
      input: { account_id: 23, person_key: 'shahid|03000000000', amount: '1000' },
    });
  });

  it('editing the text after taking a person drops him, so the key never fits other words', async () => {
    const { user } = await atTheOneTimeBuyer();
    const person = await labelled(language, 'vouchers-field-person');
    await user.click(person);
    await user.keyboard('shah');
    await screen.findByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(vouchersApi.assistOpenBills).toHaveBeenCalled();
    });
    await user.click(await labelled(language, 'vouchers-field-amount'));
    await user.keyboard('5');
    await user.click(person);
    await user.keyboard('x');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ person_key: null }),
        null,
      );
    });
  });

  it('"make regular" opens the account form with his names and phone, and the new account takes his place', async () => {
    const { user } = await atTheOneTimeBuyer();
    const person = await labelled(language, 'vouchers-field-person');
    await user.click(person);
    await user.keyboard('shah');
    await screen.findByRole('option');
    await user.keyboard('{Enter}');
    const made = account(24, 'buyer', '03040003', 'شاہد', 'Shahid');
    vi.mocked(pukhtaApi.pukhtaMakeRegular).mockResolvedValue({
      account: { id: made.id, code: made.code, name_ur: made.name_ur, name_en: made.name_en },
      moved: { amount_paisa: made.balance.amount_paisa, side: 'zero' },
      entry_id: null,
    });
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'vouchers-make-regular')),
      }),
    );
    const dialog = await screen.findByRole('dialog');
    expect(visible(dialog.textContent)).toContain(message(language, 'pukhta-make-regular-title'));
    // His names and phone are in the form already.
    const values = Array.from(dialog.querySelectorAll('input')).map((input) => input.value);
    expect(values).toContain('شاہد');
    expect(values).toContain('Shahid');
    expect(values).toContain('03000000000');
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
    );
    await waitFor(() => {
      expect(pukhtaApi.pukhtaMakeRegular).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(pukhtaApi.pukhtaMakeRegular).mock.calls[0]?.[0]).toMatchObject({
      person_key: 'shahid|03000000000',
      side: 'buyer',
      account_input: {
        account_type: 'buyer',
        name_ur: 'شاہد',
        name_en: 'Shahid',
        phone1: '03000000000',
        // No opening: the backend refuses any opening here, and the balance moves with the person.
        opening_dr: '',
        opening_cr: '',
        opening_is_advance: false,
      },
    });
    // The new account is in the buyer field, and the one-time person fields are gone.
    await waitFor(() => {
      expect(screen.queryByLabelText(named(message(language, 'vouchers-field-person')))).toBeNull();
    });
    expect(
      await screen.findByRole('combobox', {
        name: named(message(language, 'vouchers-field-buyer')),
      }),
    ).toHaveValue(language === 'ur' ? 'شاہد' : 'Shahid');
  });
});
