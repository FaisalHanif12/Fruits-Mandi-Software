// The جنرل screen (plan 17 F2, PRD 16.4, FR-6.5, FR-13.5, FR-6.7, entry-grid skill), in the Urdu and
// in the English app, by keyboard, against a fake backend: a grid of lines (account, بنام, جمع,
// note), the totals, the difference and the split from the backend, Save off until the backend says
// the entry is balanced, the person of a پختہ کھاتہ line, problems on the right cell, the cash box,
// and the language switch in the middle of an entry.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { cellOf, focusedIn, gridOf, textOf, typeCodeAndEnter } from '../helpers/bikri-screen';
import { named, pageText, startsWith } from '../helpers/cashbook-screen';
import { fail, leaveAppWindow, press, renderApp } from '../helpers/app';
import { preparePartnerMoney, type PartnerMoney } from '../helpers/partner-money';
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

const ELECTRICITY = '05010001';
const CASH = '01010001';
const PUKHTA = '03049999';

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the جنرل screen (%s)', (language) => {
  let books: PartnerMoney;

  beforeEach(() => {
    books = preparePartnerMoney(language);
  });

  const grid = () => gridOf(language, 'cashbook-general-grid-label');
  const saveButton = () =>
    screen.getByRole('button', { name: startsWith(message(language, 'layout-save-save')) });

  async function open() {
    const app = await renderApp('/rokar/general');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'cashbook-general-screen-title')),
    });
    // The grid draws its rows a moment after the screen.
    await waitFor(() => {
      expect(cellOf(grid(), 1, 'cr')).toBeInTheDocument();
    });
    return { ...app, user: userEvent.setup() };
  }

  /** Types an account by its code in the cell the cursor is on and goes on to its بنام cell. */
  async function account(user: ReturnType<typeof userEvent.setup>, row: number, code: string) {
    await user.click(cellOf(grid(), row, 'account'));
    await typeCodeAndEnter(user, code);
    await waitFor(() => {
      expect(focusedIn(grid())).toBe(`${String(row)}:dr`);
    });
  }

  /** Electricity 3,000 on the بنام side and cash 3,000 on the جمع side: the correction of Event 5. */
  async function typeBalancedEntry(user: ReturnType<typeof userEvent.setup>) {
    await account(user, 0, ELECTRICITY);
    await user.keyboard('3000{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('0:cr');
    });
    await user.keyboard('{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('1:account');
    });
    await account(user, 1, CASH);
    await user.keyboard('{Enter}3000{Enter}');
  }

  it('starts with two empty lines, today, no totals yet and Save switched off', async () => {
    await open();
    expect(
      (
        await screen.findByLabelText<HTMLInputElement>(
          named(message(language, 'cashbook-field-date')),
        )
      ).value,
    ).toBe('10-10-2026');
    expect(grid().querySelectorAll('.ag-row[row-index]')).toHaveLength(2);
    expect(pageText()).toContain(message(language, 'cashbook-general-totals-hint'));
    expect(saveButton()).toBeDisabled();
  });

  it("a balanced entry: the totals and the split are the backend's, Save opens, F10 saves the text as typed", async () => {
    const { user } = await open();
    await typeBalancedEntry(user);
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'cashbook-general-balanced'));
    });
    // The backend's totals: 3,000 on each side, and the split of the expense line (E2E-39 shares).
    const totals = visible(
      screen.getByText(named(message(language, 'cashbook-general-total-dr'))).closest('dl')
        ?.textContent,
    );
    expect(totals).toContain('3,000');
    expect(pageText()).toContain(message(language, 'cashbook-general-split-title'));
    expect(pageText()).toContain('1,200');
    expect(saveButton()).toBeEnabled();
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.generalSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.generalSave).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      lines: [
        { account_id: 501, dr: '3000', cr: '', person_key: null },
        { account_id: 1, dr: '', cr: '3000', person_key: null },
      ],
      confirm_cash_below_zero: false,
    });
    const toast = message(language, 'cashbook-saved', {
      kind: 'general',
      number: message(language, 'vouchers-number-general', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    // A new, empty entry starts on the same day.
    await waitFor(() => {
      expect(grid().querySelectorAll('.ag-row[row-index]')).toHaveLength(2);
      expect(textOf(grid(), 0, 'dr')).toBe('');
    });
  });

  it('an entry that does not balance cannot be saved: Save stays off and the difference shows', async () => {
    const { user } = await open();
    await account(user, 0, ELECTRICITY);
    await user.keyboard('3000{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('0:cr');
    });
    await user.keyboard('{Enter}{Enter}');
    await account(user, 1, CASH);
    await user.keyboard('{Enter}2500{Enter}');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'cashbook-general-not-balanced'));
    });
    expect(saveButton()).toBeDisabled();
    press('F10');
    expect(cashbookApi.generalSave).not.toHaveBeenCalled();
    // After the try, the backend's words for the entry show.
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-general-not-balanced', { dr: '3,000', cr: '2,500' }),
      );
    });
    expect(books.money.store.generalSaves).toHaveLength(0);
  });

  it('a line with both sides is told at its cell, with the words of the backend', async () => {
    const { user } = await open();
    await account(user, 0, ELECTRICITY);
    await user.keyboard('3000{Enter}500{Enter}');
    await waitFor(() => {
      expect(cellOf(grid(), 0, 'cr')).toHaveClass('cell-error');
    });
    expect(cellOf(grid(), 0, 'dr')).not.toHaveClass('cell-error');
  });

  it('letters in a money cell are told at that cell with the backend’s words, and nothing breaks', async () => {
    const { user } = await open();
    await account(user, 0, ELECTRICITY);
    await user.keyboard('abc{Enter}');
    await waitFor(() => {
      expect(cellOf(grid(), 0, 'dr')).toHaveClass('cell-error');
    });
    // The grid and its totals are still there; the entry cannot be saved.
    expect(cellOf(grid(), 1, 'cr')).toBeInTheDocument();
    expect(saveButton()).toBeDisabled();
  });

  it('a line on a پختہ کھاتہ account asks for the person, then goes on to its بنام cell (FR-13.5)', async () => {
    const { user } = await open();
    await user.click(cellOf(grid(), 0, 'account'));
    await typeCodeAndEnter(user, PUKHTA);
    // Enter went to the person field below the grid, not to the amount.
    const person = await screen.findByRole('combobox', {
      name: (name) =>
        visible(name).includes(message(language, 'cashbook-general-people-title')) ||
        visible(name).startsWith(nameOf(language, 'سطر', 'Person for line')),
    });
    await waitFor(() => {
      expect(person).toHaveFocus();
    });
    await user.keyboard('sha');
    await screen.findAllByRole('option');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('0:dr');
    });
    expect(textOf(grid(), 0, 'person')).toContain('03000000000');
    await user.keyboard('1000{Enter}');
    // The preview is asked with the person's key and the amount as typed.
    await waitFor(() => {
      const lastCall = vi.mocked(cashbookApi.generalPreview).mock.calls.at(-1)?.[0];
      expect(lastCall?.lines[0]).toMatchObject({
        account_id: 23,
        person_key: 'shahid|03000000000',
        dr: '1000',
      });
    });
  });

  it('cash below zero asks before a line credits روکڑ, and No saves nothing', async () => {
    books.cash.store.cash = 500;
    const { user } = await open();
    await typeBalancedEntry(user);
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-2,500' }));
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-no')),
    });
    await user.click(no);
    expect(cashbookApi.generalSave).not.toHaveBeenCalled();
    press('F10');
    const yes = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-yes')),
    });
    await user.click(yes);
    await waitFor(() => {
      expect(cashbookApi.generalSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.generalSave).mock.calls[0]?.[0]).toMatchObject({
      confirm_cash_below_zero: true,
    });
  });

  it('the totals failing to load say so with a retry that works', async () => {
    const { user } = await open();
    vi.mocked(cashbookApi.generalPreview).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await account(user, 0, ELECTRICITY);
    const title = await screen.findByText(named(message(language, 'cashbook-preview-failed')));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    await user.click(
      within(alert).getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
    );
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'cashbook-general-total-dr'));
    });
  });

  it('switching the language in the middle of an entry loses nothing', async () => {
    const { user } = await open();
    await account(user, 0, ELECTRICITY);
    await user.keyboard('3000{Enter}');
    await waitFor(() => {
      expect(focusedIn(grid())).toBe('0:cr');
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const rebuilt = gridOf(other, 'cashbook-general-grid-label');
    await waitFor(() => {
      expect(textOf(rebuilt, 0, 'dr')).toBe('3,000');
    });
    expect(textOf(rebuilt, 0, 'account')).toBe(nameOf(other, 'بجلی', 'Electricity'));
    // The focused cell is focused again in the rebuilt grid.
    await waitFor(() => {
      expect(focusedIn(rebuilt)).toBe('0:cr');
    });
  });

  it('the keyboard alone does it: Enter walks the cells of both lines and F10 saves (F30)', async () => {
    const { user } = await open();
    await typeBalancedEntry(user);
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.generalSave).toHaveBeenCalledTimes(1);
    });
    expect(books.cash.store.cash).toBe(53484 - 3000);
  });
});
