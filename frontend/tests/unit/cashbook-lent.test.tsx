// The ادھار رقم screen (plan 17, PRD 16.5, FR-16.4, FR-16.5, FR-16.9, FR-7.67, FR-6.7, E2E-30), in
// the Urdu and in the English app, by keyboard, against a fake backend: ONE screen with a type list
// for money lent and money returned, the person picker with F4 to add a person, the return in cash
// or from the salary (the only mode of this area), the person's loans under the form, the cash box,
// and the language switch in the middle of an entry.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  choose,
  dropdown,
  labelled,
  named,
  pageText,
  pickAccount,
  prepareCashbook,
  typeAmount,
  startsWith,
} from '../helpers/cashbook-screen';
import { FORMER_PAYMENT_LABELS, labelIs } from '../helpers/former-payment-labels';
import { ASLAM_ROW, type FakeCashbook } from '../helpers/cashbook';
import { fail, leaveAppWindow, press, renderApp } from '../helpers/app';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the ادھار رقم screen (%s)', (language) => {
  let books: FakeCashbook;

  beforeEach(() => {
    books = prepareCashbook(language, [ASLAM_ROW]);
  });

  async function open(type = 'money_lent') {
    const app = await renderApp(`/rokar/lent?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-lent-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  const aslam = nameOf(language, 'اسلم', 'Aslam');

  /** Lends 5,000 to اسلم with the reason, through the screen, and waits for the save. */
  async function lend(user: ReturnType<typeof userEvent.setup>, amount = '5000') {
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    await typeAmount(user, language, amount);
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('home');
    await user.click(await labelled(language, 'cashbook-field-return-by'));
    await user.keyboard('15-10-2026');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveMoneyLent).toHaveBeenCalledTimes(1);
    });
  }

  it('starts as money lent, with the types, today, and no bank, cheque or count field', async () => {
    await open();
    const group = screen.getByRole('group', {
      name: named(message(language, 'cashbook-lent-type')),
    });
    expect(
      within(group)
        .getAllByRole('button')
        .map((button) => visible(button.textContent)),
    ).toEqual([
      message(language, 'cashbook-type-money-lent'),
      message(language, 'cashbook-type-money-lent-return'),
    ]);
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
    expect(await dropdown(language, 'cashbook-field-person')).toBeInTheDocument();
    expect(await labelled(language, 'cashbook-field-reason')).toBeInTheDocument();
    expect(screen.queryByLabelText(named(message(language, 'cashbook-field-mode')))).toBeNull();
    for (const gone of FORMER_PAYMENT_LABELS[language]) {
      expect(screen.queryByLabelText(labelIs(gone))).toBeNull();
    }
    // Without a person the list says to choose one.
    expect(await screen.findByText(message(language, 'cashbook-lent-pick'))).toBeInTheDocument();
  });

  it("lends cash: the text goes as typed, the loan is listed, and the balance is the backend's", async () => {
    const { user } = await open();
    await lend(user);
    expect(vi.mocked(cashbookApi.saveMoneyLent).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      person_account_id: 61,
      amount: '5000',
      reason: 'home',
      return_by: '15-10-2026',
      confirm_cash_below_zero: false,
    });
    const toast = message(language, 'cashbook-saved', {
      kind: 'money_lent',
      number: message(language, 'vouchers-number-money-lent', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    // The person stays chosen for the next entry, so his list stays in view.
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-lent-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('5,000');
    });
    expect(pageText()).toContain(message(language, 'cashbook-lent-balance'));
  });

  it('a return-by date before the entry date is told at that field and nothing is saved (FR-16.9)', async () => {
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    await typeAmount(user, language, '5000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('home');
    await user.click(await labelled(language, 'cashbook-field-return-by'));
    await user.keyboard('09-10-2026');
    press('F10');
    const field = await labelled(language, 'cashbook-field-return-by');
    await waitFor(() => {
      expect(field).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-return-by-before-entry'));
    expect(books.store.saves).toHaveLength(0);
  });

  it('a return in cash and one from the salary: the type keeps the person, the mode is the only choice (E2E-30)', async () => {
    const { user } = await open();
    await lend(user);
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'cashbook-type-money-lent-return')),
      }),
    );
    // The date, the person and the narration stay; the reason and return-by fields are gone.
    expect(await dropdown(language, 'cashbook-field-person')).toHaveValue(aslam);
    expect(screen.queryByLabelText(named(message(language, 'cashbook-field-reason')))).toBeNull();
    await typeAmount(user, language, '3000');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveMoneyLentReturn).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveMoneyLentReturn).mock.calls[0]?.[0]).toMatchObject({
      person_account_id: 61,
      amount: '3000',
      mode: 'cash',
      salary_head_id: null,
    });
    // Now 2,000 from the salary: no cash moves, and the screen says what happens to the split.
    await choose(user, language, 'cashbook-field-mode', message(language, 'cashbook-mode-salary'));
    expect(await screen.findByText(message(language, 'cashbook-salary-note'))).toBeInTheDocument();
    await typeAmount(user, language, '2000');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveMoneyLentReturn).toHaveBeenCalledTimes(2);
    });
    expect(vi.mocked(cashbookApi.saveMoneyLentReturn).mock.calls[1]?.[0]).toMatchObject({
      amount: '2000',
      mode: 'salary',
      salary_head_id: null,
    });
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-lent-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('5,000');
    });
    // Cash: 53,484 - 5,000 + 3,000 (the salary part moves none).
    expect(books.store.cash).toBe(51484);
  });

  it('a return above what is open is told at the amount', async () => {
    const { user } = await open();
    await lend(user);
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'cashbook-type-money-lent-return')),
      }),
    );
    await typeAmount(user, language, '9000');
    press('F10');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-return-exceeds-open', { open: '5,000' }));
  });

  it('cash below zero shows the amber note and F10 asks first; No saves nothing', async () => {
    books.store.cash = 1000;
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    await typeAmount(user, language, '5000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('home');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-4,000' }));
    });
    // The answer must be for the last thing typed (the reason), or the save is asked without the
    // question first. On a busy machine the preview can still be on its way.
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ reason: 'home' }),
        null,
      );
    });
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    await user.click(no);
    expect(cashbookApi.saveMoneyLent).not.toHaveBeenCalled();
  });

  it('F4 in the person field adds a person without leaving the screen, and he comes back into the field', async () => {
    const { user } = await open();
    await user.click(await dropdown(language, 'cashbook-field-person'));
    await user.keyboard('Salman');
    await screen.findByText(message(language, 'search-no-match'));
    press('F4');
    await screen.findByRole('heading', { name: message(language, 'cashbook-add-person-title') });
    // What was typed became the English name; the Urdu name is typed by hand.
    const english = await labelled(language, 'cashbook-add-name-en');
    expect(english.value).toBe('Salman');
    const urdu = await labelled(language, 'cashbook-add-name-ur');
    await user.click(urdu);
    await user.keyboard('سلمان');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.lentPersonCreate).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.lentPersonCreate).mock.calls[0]?.[0]).toMatchObject({
      name_en: 'Salman',
      phone: null,
      staff_id: null,
    });
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: message(language, 'cashbook-add-person-title') }),
      ).toBeNull();
    });
    expect(await dropdown(language, 'cashbook-field-person')).toHaveValue(
      nameOf(language, 'سلمان', 'Salman'),
    );
  });

  it("the person's loans failing to load say so with a retry that works", async () => {
    const { user } = await open();
    vi.mocked(cashbookApi.lentList).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    const title = await screen.findByText(message(language, 'cashbook-lent-failed'));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    await user.click(
      within(alert).getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
    );
    expect(await screen.findByText(message(language, 'cashbook-lent-empty'))).toBeInTheDocument();
  });

  it('switching the language in the middle of an entry loses nothing', async () => {
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
    await typeAmount(user, language, '300');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('r');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'cashbook-field-amount')).value).toBe('300');
    expect((await labelled(other, 'cashbook-field-reason')).value).toBe('r');
    expect(await dropdown(other, 'cashbook-field-person')).toHaveValue(
      nameOf(other, 'اسلم', 'Aslam'),
    );
  });

  it('the preview failing says so with a retry, and nothing is made up', async () => {
    const { user } = await open();
    vi.mocked(vouchersApi.vouchersPreview).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await typeAmount(user, language, '300');
    expect(
      await screen.findByText(message(language, 'cashbook-preview-failed')),
    ).toBeInTheDocument();
  });

  it('the keyboard alone does it: Enter walks date, person, amount, reason and return-by; F10 saves (F30)', async () => {
    const { user } = await open();
    const date = await labelled(language, 'cashbook-field-date');
    await waitFor(() => {
      expect(date).toHaveFocus();
    });
    await user.keyboard('{Enter}');
    const person = await dropdown(language, 'cashbook-field-person');
    await waitFor(() => {
      expect(person).toHaveFocus();
    });
    await user.keyboard('aslam');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    // Enter picks him and moves on to the amount.
    await user.keyboard('{Enter}');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveFocus();
    });
    await user.keyboard('5000{Enter}');
    const reason = await labelled(language, 'cashbook-field-reason');
    await waitFor(() => {
      expect(reason).toHaveFocus();
    });
    await user.keyboard('home{Enter}');
    const returnBy = await labelled(language, 'cashbook-field-return-by');
    await waitFor(() => {
      expect(returnBy).toHaveFocus();
    });
    await user.keyboard('15-10-2026');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveMoneyLent).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveMoneyLent).mock.calls[0]?.[0]).toMatchObject({
      person_account_id: 61,
      amount: '5000',
      reason: 'home',
      return_by: '15-10-2026',
    });
    // After the save the person stays chosen for the next entry; F2 starts a clean form.
    expect(await dropdown(language, 'cashbook-field-person')).toHaveValue(aslam);
    press('F2');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(await dropdown(language, 'cashbook-field-person')).toHaveValue('');
  });
});
