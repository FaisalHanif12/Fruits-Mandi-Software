// The خرچہ screen (plan 17, PRD 16.2, 16.3, FR-7.67, FR-7.68, FR-7.62, FR-16.8, FR-6.7, E2E-20,
// E2E-39), in the Urdu and in the English app, by keyboard, against a fake backend: ONE screen with
// a type list for دکان خرچہ, ذاتی خرچہ and ذاتی خرچہ واپسی, the split or the partner total shown
// before the save, the lists below the form, repeat last month, the right-type note, the cash box,
// and the language switch in the middle of an entry. The screen works out no money: every figure
// shown is the fake backend's.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { HEADS } from '../helpers/cashbook-books';
import {
  choose,
  dropdown,
  labelled,
  named,
  pageText,
  prepareCashbook,
  typeAmount,
  startsWith,
} from '../helpers/cashbook-screen';
import type { FakeCashbook } from '../helpers/cashbook';
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

describe.each(LANGUAGES)('the خرچہ screen (%s)', (language) => {
  let books: FakeCashbook;

  beforeEach(() => {
    books = prepareCashbook(language);
  });

  const headName = (index: number) => {
    const head = HEADS[index];
    if (head === undefined) throw new Error('no head');
    return nameOf(language, head.name_ur, head.name_en);
  };

  async function open(type = 'shop_expense') {
    const app = await renderApp(`/rokar/expense?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  const saveButton = () =>
    screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) });

  it('Alt+K opens it as دکان خرچہ with the three types, today, and no mode, bank or cheque', async () => {
    await renderApp('/');
    // The app is up when its main menu is.
    await screen.findByRole('navigation', { name: named(message(language, 'layout-nav-label')) });
    press('KeyK', { altKey: true });
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    const group = screen.getByRole('group', {
      name: named(message(language, 'cashbook-expense-type')),
    });
    const buttons = within(group).getAllByRole('button');
    expect(buttons.map((button) => visible(button.textContent))).toEqual([
      message(language, 'cashbook-type-shop-expense'),
      message(language, 'cashbook-type-personal-expense'),
      message(language, 'cashbook-type-personal-return'),
    ]);
    expect(buttons[0]).toHaveAttribute('aria-pressed', 'true');
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
    expect(saveButton()).toBeDisabled();
    for (const gone of ['vouchers-field-mode', 'vouchers-field-bank', 'vouchers-field-cheque-no']) {
      expect(screen.queryByLabelText(named(message(language, gone)))).toBeNull();
    }
    // The side menu of the روکڑ area is next to the main menu.
    expect(
      screen.getByRole('navigation', { name: named(message(language, 'layout-rokar-label')) }),
    ).toBeInTheDocument();
  });

  it('the heads come most used first, as the backend sent them', async () => {
    const { user } = await open();
    await user.click(await dropdown(language, 'cashbook-field-head'));
    const options = await screen.findAllByRole('option');
    expect(options.map((option) => visible(option.textContent))).toEqual(
      HEADS.map((_, index) => headName(index)),
    );
  });

  it("shows each partner's share before the save, with the backend's workings (E2E-39, E2E-20)", async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    await waitFor(() => {
      expect(cashbookApi.expensePreview).toHaveBeenLastCalledWith(
        expect.objectContaining({
          type: 'shop_expense',
          expense_head_id: 1,
          amount: '3000',
          date: '10-10-2026',
        }),
        null,
      );
    });
    const card = (
      await screen.findByRole('heading', { name: message(language, 'cashbook-shares-title') })
    ).closest('section');
    if (card === null) throw new Error('no shares card');
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain('1,200');
      expect(text).toContain('900');
    });
    expect(
      within(card).getAllByRole('button', {
        name: startsWith(message(language, 'calc-info-label')),
      }),
    ).toHaveLength(3);
    // Nothing was saved by looking.
    expect(cashbookApi.saveShopExpense).not.toHaveBeenCalled();
  });

  it('F10 saves it as typed, says so, keeps the date, clears the rest and lists it with the total', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    await user.click(await labelled(language, 'cashbook-field-narration'));
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveShopExpense).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveShopExpense).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      expense_head_id: 1,
      amount: '3000',
      paid_to: null,
      narration: 'x',
      confirm_cash_below_zero: false,
    });
    const toast = message(language, 'cashbook-saved', {
      kind: 'shop_expense',
      number: message(language, 'vouchers-number-shop-expense', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'cashbook-field-amount')))
          .value,
      ).toBe('');
    });
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
    // The day's list under the form has it, with the backend's total.
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-day-label')),
    });
    await waitFor(() => {
      const text = visible(list.textContent);
      expect(text).toContain(message(language, 'vouchers-number-shop-expense', { number: 1 }));
      expect(text).toContain('3,000');
    });
  });

  it('changing the type keeps the date, amount and narration and clears the rest (FR-7.67)', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    await user.click(await labelled(language, 'cashbook-field-narration'));
    await user.keyboard('n');
    await user.click(
      screen.getByRole('button', {
        name: named(message(language, 'cashbook-type-personal-expense')),
      }),
    );
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('3,000');
    expect((await labelled(language, 'cashbook-field-narration')).value).toBe('n');
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
    // The head is gone; the partner, reason, return-by and taken-by fields are there, empty.
    expect(
      screen.queryByRole('combobox', { name: named(message(language, 'cashbook-field-head')) }),
    ).toBeNull();
    expect((await labelled(language, 'cashbook-field-reason')).value).toBe('');
    expect(await dropdown(language, 'cashbook-field-partner')).toBeInTheDocument();
    expect(await labelled(language, 'cashbook-field-return-by')).toBeInTheDocument();
    expect(await labelled(language, 'cashbook-field-taken-by')).toBeInTheDocument();
  });

  it("a partner's own spending shows his total now and after, and the others not affected (E2E-39)", async () => {
    const { user } = await open('personal_expense');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('home');
    const card = (
      await screen.findByRole('heading', { name: message(language, 'cashbook-effect-title') })
    ).closest('section');
    if (card === null) throw new Error('no effect card');
    // الف's total is 2,196 in these books; after taking 2,000 it is 196.
    await waitFor(() => {
      const text = visible(card.textContent);
      expect(text).toContain('2,196');
      expect(text).toContain('196');
    });
    expect(visible(card.textContent)).toContain(message(language, 'cashbook-others-not-affected'));
    // His money taken is listed under the form (nothing yet).
    expect(await screen.findByText(message(language, 'cashbook-takes-empty'))).toBeInTheDocument();
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.savePersonalExpense).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.savePersonalExpense).mock.calls[0]?.[0]).toMatchObject({
      partner_id: 1,
      amount: '2000',
      reason: 'home',
      return_by: null,
      taken_by: null,
    });
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-takes-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('2,000');
    });
  });

  it('a return above what is open is told at the amount; one within it settles the take', async () => {
    const { user } = await open('personal_expense');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('home');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.savePersonalExpense).toHaveBeenCalledTimes(1);
    });
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'cashbook-type-personal-return')),
      }),
    );
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2500');
    press('F10');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-return-exceeds-open', { open: '2,000' }));
    // The backend refused it at the amount; only the take was saved.
    expect(books.store.saves).toHaveLength(1);
    await user.clear(amount);
    await user.keyboard('2000');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.savePersonalReturn).toHaveBeenCalledTimes(1);
    });
    expect(books.store.totals.get(1)).toBe(2196);
  });

  it('a name of a partner in "paid to" offers ذاتی خرچہ with a button; only the button switches (FR-7.68)', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    await user.click(await labelled(language, 'cashbook-field-paid-to'));
    await user.keyboard('x');
    expect(
      screen.queryByRole('button', {
        name: startsWith(message(language, 'cashbook-personal-make')),
      }),
    ).toBeNull();
    const paid = await labelled(language, 'cashbook-field-paid-to');
    await user.clear(paid);
    // Typed in English letters, with a space to end the word: the Urdu app keeps them as typed here
    // because the fake conversion offers nothing; the match is the backend's (either script).
    await user.keyboard(nameOf(language, 'الف', 'Partner A'));
    const make = await screen.findByRole('button', {
      name: startsWith(message(language, 'cashbook-personal-make')),
    });
    // Still a دکان خرچہ: nothing switched by itself.
    expect(
      screen.getByRole('button', {
        name: named(message(language, 'cashbook-type-shop-expense')),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
    await user.click(make);
    expect(
      screen.getByRole('button', {
        name: named(message(language, 'cashbook-type-personal-expense')),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('3,000');
    const partner = await dropdown(language, 'cashbook-field-partner');
    // The partners arrive a moment after the form opens; the value shows once they are there.
    await waitFor(() => {
      expect(partner).toHaveTextContent(nameOf(language, 'الف', 'Partner A'));
    });
  });

  it('cash below zero shows the amber note, F10 asks with No focused, No saves nothing, Yes saves', async () => {
    books.store.cash = 1000;
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-confirm-title'));
    });
    expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-2,000' }));
    press('F10');
    const no = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-no')),
    });
    await waitFor(() => {
      expect(no).toHaveFocus();
    });
    await user.click(no);
    expect(cashbookApi.saveShopExpense).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    press('F10');
    await user.click(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'common-cash-confirm-yes')),
      }),
    );
    await waitFor(() => {
      expect(cashbookApi.saveShopExpense).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveShopExpense).mock.calls[0]?.[0]).toMatchObject({
      confirm_cash_below_zero: true,
    });
  });

  it('repeat last month fills one regular expense into the form for the user to check', async () => {
    const rent = HEADS[1];
    if (rent === undefined) throw new Error('no head');
    books.store.repeat.push({
      head: rent,
      amount: 25000,
      paid_to: 'x',
      narration: '',
      last_date: '2026-09-05',
    });
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'cashbook-repeat-open')),
      }),
    );
    const panel = (
      await screen.findByRole('heading', { name: message(language, 'cashbook-repeat-title') })
    ).closest('section');
    if (panel === null) throw new Error('no panel');
    expect(visible(panel.textContent)).toContain('25,000');
    await user.click(
      within(panel).getByRole('button', {
        name: startsWith(message(language, 'cashbook-repeat-fill')),
      }),
    );
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('25,000');
    expect(await dropdown(language, 'cashbook-field-head')).toHaveTextContent(headName(1));
    // It fills the form only: nothing was saved, and the panel closed.
    expect(cashbookApi.saveShopExpense).not.toHaveBeenCalled();
    expect(
      screen.queryByRole('heading', { name: message(language, 'cashbook-repeat-title') }),
    ).toBeNull();
  });

  it('repeat last month with nothing to repeat says so', async () => {
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'cashbook-repeat-open')),
      }),
    );
    expect(await screen.findByText(message(language, 'cashbook-repeat-empty'))).toBeInTheDocument();
  });

  it('a refused field shows at the field and nothing typed is lost', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    vi.mocked(cashbookApi.saveShopExpense).mockRejectedValueOnce(
      fail({ code: 'ERR_DATE_CLOSED', field: 'date', values: { closed_up_to: '09-10-2026' } }),
    );
    press('F10');
    const date = await labelled(language, 'cashbook-field-date');
    await waitFor(() => {
      expect(date).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(
      message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }),
    );
    expect((await labelled(language, 'cashbook-field-amount')).value).toBe('3,000');
  });

  it('an empty head is told at the head only after the user tries to save', async () => {
    const { user } = await open();
    await typeAmount(user, language, '3000');
    const head = await dropdown(language, 'cashbook-field-head');
    await waitFor(() => {
      expect(cashbookApi.expensePreview).toHaveBeenCalled();
    });
    expect(head).not.toHaveAttribute('aria-invalid', 'true');
    press('F10');
    await waitFor(() => {
      expect(head).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-expense-head-required'));
    // The backend refused it at the head, and nothing was saved.
    expect(books.store.saves).toHaveLength(0);
  });

  it('switching the language in the middle of an entry loses nothing', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    await typeAmount(user, language, '3000');
    await user.click(await labelled(language, 'cashbook-field-narration'));
    await user.keyboard('n');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'cashbook-field-amount')).value).toBe('3,000');
    expect((await labelled(other, 'cashbook-field-narration')).value).toBe('n');
    const head = HEADS[0];
    if (head === undefined) throw new Error('no head');
    expect(await dropdown(other, 'cashbook-field-head')).toHaveTextContent(
      nameOf(other, head.name_ur, head.name_en),
    );
    const group = screen.getByRole('group', {
      name: named(message(other, 'cashbook-expense-type')),
    });
    expect(
      within(group).getByRole('button', {
        name: named(message(other, 'cashbook-type-shop-expense')),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('the heads failing to load says so with a retry that works', async () => {
    vi.mocked(cashbookApi.expenseHeads).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await open();
    const title = await screen.findByText(message(language, 'cashbook-heads-failed'));
    const section = title.closest('[role="alert"]');
    if (!(section instanceof HTMLElement)) throw new Error('no alert');
    expect(visible(section.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent.setup().click(
      within(section).getByRole('button', {
        name: startsWith(message(language, 'common-retry')),
      }),
    );
    expect(await dropdown(language, 'cashbook-field-head')).toBeInTheDocument();
  });

  it('a failing preview says so with a retry, and nothing is made up', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-head', headName(0));
    vi.mocked(cashbookApi.expensePreview).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await typeAmount(user, language, '3000');
    expect(
      await screen.findByText(message(language, 'cashbook-preview-failed')),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(screen.queryByText(message(language, 'cashbook-preview-failed'))).toBeNull();
    });
  });

  it('the day with no shop expense says why and what to do', async () => {
    await open();
    expect(await screen.findByText(message(language, 'cashbook-day-empty'))).toBeInTheDocument();
    expect(screen.getByText(message(language, 'cashbook-day-empty-help'))).toBeInTheDocument();
  });

  it('the keyboard alone does it: type the amount, choose the head, F10, then F2 for the next', async () => {
    const { user } = await open();
    const date = await labelled(language, 'cashbook-field-date');
    await waitFor(() => {
      expect(date).toHaveFocus();
    });
    // Enter walks the fields; the head is a drop-down the keyboard opens.
    await user.keyboard('{Enter}');
    const head = await dropdown(language, 'cashbook-field-head');
    await waitFor(() => {
      expect(head).toHaveFocus();
    });
    await user.keyboard('{Enter}');
    await user.click(await screen.findByRole('option', { name: named(headName(0)) }));
    await typeAmount(user, language, '3000');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveShopExpense).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'cashbook-field-amount')))
          .value,
      ).toBe('');
    });
    press('F2');
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
  });
});
