// The الگ کھاتہ screen (plan 17, PRD 16.6, FR-16.6, FR-7.67, FR-6.7, E2E-38), in the Urdu and in the
// English app, by keyboard, against a fake backend: ONE screen with a type list for money paid out
// (minus) and received (plus), the khata picker with F4 to add a khata, the entries with the signed
// running balance under the form, the cash box, and the language switch in the middle of an entry.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  dropdown,
  labelled,
  named,
  pageText,
  pickAccount,
  prepareCashbook,
  typeAmount,
  startsWith,
} from '../helpers/cashbook-screen';
import { KHATA_ROW, type FakeCashbook } from '../helpers/cashbook';
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

describe.each(LANGUAGES)('the الگ کھاتہ screen (%s)', (language) => {
  let books: FakeCashbook;
  const family = nameOf(language, 'خاندانی کھاتہ', 'Family Khata');

  beforeEach(() => {
    books = prepareCashbook(language, [KHATA_ROW]);
  });

  async function open(type = 'khata_out') {
    const app = await renderApp(`/rokar/khata?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-khata-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  async function entry(user: ReturnType<typeof userEvent.setup>, amount: string, reason: string) {
    await typeAmount(user, language, amount);
    const field = await labelled(language, 'cashbook-field-reason');
    await user.click(field);
    await user.clear(field);
    await user.keyboard(reason);
    press('F10');
  }

  it('starts as paid out, with the two types and the same fields for both', async () => {
    await open();
    const group = screen.getByRole('group', {
      name: named(message(language, 'cashbook-khata-type')),
    });
    expect(
      within(group)
        .getAllByRole('button')
        .map((button) => visible(button.textContent)),
    ).toEqual([
      message(language, 'cashbook-type-khata-out'),
      message(language, 'cashbook-type-khata-in'),
    ]);
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
    expect(await dropdown(language, 'cashbook-field-khata')).toBeInTheDocument();
    expect(await labelled(language, 'cashbook-field-reason')).toBeInTheDocument();
    expect(screen.queryByLabelText(named(message(language, 'vouchers-field-mode')))).toBeNull();
    expect(await screen.findByText(message(language, 'cashbook-khata-pick'))).toBeInTheDocument();
  });

  it('E2E-38: paid out 10,000 goes minus, 4,000 received brings it to minus 6,000, cash follows', async () => {
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
    await entry(user, '10000', 'family');
    await waitFor(() => {
      expect(cashbookApi.saveKhataOut).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveKhataOut).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      khata_account_id: 71,
      amount: '10000',
      reason: 'family',
      confirm_cash_below_zero: false,
    });
    const list = await screen.findByRole('group', {
      name: named(message(language, 'cashbook-khata-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('-10,000');
    });
    // Received: the type changes, the khata stays, and the balance goes to minus 6,000.
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'cashbook-type-khata-in')) }),
    );
    expect(await dropdown(language, 'cashbook-field-khata')).toHaveValue(family);
    await entry(user, '4000', 'back');
    await waitFor(() => {
      expect(cashbookApi.saveKhataIn).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('-6,000');
    });
    expect(books.store.cash).toBe(53484 - 10000 + 4000);
    expect(pageText()).toContain(message(language, 'cashbook-khata-sign-hint'));
  });

  it('a missing khata or reason is told at its field only after the user tries to save', async () => {
    const { user } = await open();
    await typeAmount(user, language, '1000');
    const field = await dropdown(language, 'cashbook-field-khata');
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenCalled();
    });
    expect(field).not.toHaveAttribute('aria-invalid', 'true');
    press('F10');
    await waitFor(() => {
      expect(field).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-khata-required'));
    expect(pageText()).toContain(message(language, 'err-reason-required'));
    // The backend refused it at the fields, and nothing was saved.
    expect(books.store.saves).toHaveLength(0);
  });

  it('cash below zero asks before paying out; receiving never asks', async () => {
    books.store.cash = 500;
    const { user } = await open();
    await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
    await typeAmount(user, language, '1000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('family');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-500' }));
    });
    // The answer must be for the last thing typed (the reason); on a busy machine it can still be
    // on its way, and then the save is asked for without the question first.
    await waitFor(() => {
      expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ reason: 'family' }),
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
    await user.click(no);
    expect(cashbookApi.saveKhataOut).not.toHaveBeenCalled();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'cashbook-type-khata-in')) }),
    );
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveKhataIn).toHaveBeenCalledTimes(1);
    });
    expect(
      screen.queryByRole('button', {
        name: startsWith(message(language, 'common-cash-confirm-no')),
      }),
    ).toBeNull();
  });

  it('F4 in the khata field adds a khata and it comes back into the field', async () => {
    const { user } = await open();
    await user.click(await dropdown(language, 'cashbook-field-khata'));
    await user.keyboard('Garden');
    await screen.findByText(message(language, 'search-no-match'));
    press('F4');
    await screen.findByRole('heading', {
      name: named(message(language, 'cashbook-add-khata-title')),
    });
    const urdu = await labelled(language, 'cashbook-add-name-ur');
    await user.click(urdu);
    await user.keyboard('باغ');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.khataCreate).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.khataCreate).mock.calls[0]?.[0]).toMatchObject({
      name_en: 'Garden',
      note: null,
    });
    expect(await dropdown(language, 'cashbook-field-khata')).toHaveValue(
      nameOf(language, 'باغ', 'Garden'),
    );
  });

  it('the entries failing to load say so with a retry that works', async () => {
    const { user } = await open();
    vi.mocked(cashbookApi.khataStatement).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
    const title = await screen.findByText(message(language, 'cashbook-khata-failed'));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    await user.click(
      within(alert).getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
    );
    expect(await screen.findByText(message(language, 'cashbook-khata-empty'))).toBeInTheDocument();
  });

  it('switching the language in the middle of an entry loses nothing, the type included', async () => {
    const { user } = await open('khata_in');
    await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
    await typeAmount(user, language, '700');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('r');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'cashbook-field-amount')).value).toBe('700');
    expect((await labelled(other, 'cashbook-field-reason')).value).toBe('r');
    expect(await dropdown(other, 'cashbook-field-khata')).toHaveValue(
      nameOf(other, 'خاندانی کھاتہ', 'Family Khata'),
    );
    const group = screen.getByRole('group', { name: named(message(other, 'cashbook-khata-type')) });
    expect(
      within(group).getByRole('button', {
        name: named(message(other, 'cashbook-type-khata-in')),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('the keyboard alone does it: Enter walks date, khata, amount and reason; F10 saves (F30)', async () => {
    const { user } = await open();
    const date = await labelled(language, 'cashbook-field-date');
    await waitFor(() => {
      expect(date).toHaveFocus();
    });
    await user.keyboard('{Enter}');
    const khata = await dropdown(language, 'cashbook-field-khata');
    await waitFor(() => {
      expect(khata).toHaveFocus();
    });
    await user.keyboard('family');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    const amount = await labelled(language, 'cashbook-field-amount');
    await waitFor(() => {
      expect(amount).toHaveFocus();
    });
    await user.keyboard('10000{Enter}');
    const reason = await labelled(language, 'cashbook-field-reason');
    await waitFor(() => {
      expect(reason).toHaveFocus();
    });
    await user.keyboard('family');
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.saveKhataOut).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.saveKhataOut).mock.calls[0]?.[0]).toMatchObject({
      khata_account_id: 71,
      amount: '10000',
      reason: 'family',
    });
    // After the save the khata stays chosen; the next entry is a receipt of money for it.
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'cashbook-type-khata-in')) }),
    );
    expect(await dropdown(language, 'cashbook-field-khata')).toHaveValue(family);
  });
});
