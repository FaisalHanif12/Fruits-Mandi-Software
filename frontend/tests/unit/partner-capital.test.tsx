// The سرمایہ screen (plan 17 F2, PRD 17.4, FR-17.7, FR-17.8, FR-7.67, FR-6.7, E2E-35), in the Urdu
// and in the English app, by keyboard, against a fake backend: ONE screen with a type list for
// capital put in and taken back, the partner drop-down, the history and balance under the form, the
// warning when more than the balance is taken back (it still saves), the cash box, and the language
// switch in the middle of an entry.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  choose,
  labelled,
  named,
  pageText,
  startsWith,
  typeAmount,
} from '../helpers/cashbook-screen';
import { dropdown } from '../helpers/cashbook-screen';
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

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the سرمایہ screen (%s)', (language) => {
  let books: PartnerMoney;
  const partnerB = nameOf(language, 'ب', 'Partner B');

  beforeEach(() => {
    books = preparePartnerMoney(language);
  });

  async function open(search = '?type=capital_in') {
    const app = await renderApp(`/partners/capital${search}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'partners-capital-screen-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  it('starts as capital put in, with the two types, today, and no bank, cheque or count field', async () => {
    await open();
    const group = screen.getByRole('group', {
      name: named(message(language, 'partners-capital-type')),
    });
    expect(
      within(group)
        .getAllByRole('button')
        .map((button) => visible(button.textContent)),
    ).toEqual([
      message(language, 'cashbook-type-capital-in'),
      message(language, 'cashbook-type-capital-out'),
    ]);
    expect((await labelled(language, 'cashbook-field-date')).value).toBe('10-10-2026');
    expect(await dropdown(language, 'cashbook-field-partner')).toBeInTheDocument();
    expect(screen.queryByLabelText(named(message(language, 'vouchers-field-mode')))).toBeNull();
    // Without a partner the history says to choose one.
    expect(
      await screen.findByText(named(message(language, 'partners-capital-pick'))),
    ).toBeInTheDocument();
  });

  it('E2E-35: partner ب puts in 50,000: the text goes as typed, cash comes in, his history shows it', async () => {
    const { user } = await open();
    await choose(user, language, 'cashbook-field-partner', partnerB);
    await typeAmount(user, language, '50000');
    // The balance now and after come from the backend's preview.
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'cashbook-balance-after'));
    });
    press('F10');
    await waitFor(() => {
      expect(partnersApi.saveCapitalIn).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(partnersApi.saveCapitalIn).mock.calls[0]?.[0]).toMatchObject({
      date: '10-10-2026',
      partner_id: 2,
      amount: '50000',
      confirm_cash_below_zero: false,
    });
    const toast = message(language, 'cashbook-saved', {
      kind: 'capital_in',
      number: message(language, 'vouchers-number-capital-in', { number: 1 }),
    });
    expect(
      (await screen.findAllByText((content) => visible(content) === toast)).length,
    ).toBeGreaterThan(0);
    expect(books.cash.store.cash).toBe(53484 + 50000);
    // The partner stays chosen, so his history is in view with the balance.
    const list = await screen.findByRole('group', {
      name: named(message(language, 'partners-capital-label')),
    });
    await waitFor(() => {
      expect(visible(list.textContent)).toContain('50,000');
    });
    expect(pageText()).toContain(message(language, 'partners-capital-balance'));
    expect(pageText()).toContain(message(language, 'partners-capital-kind-invested'));
  });

  it('taking back more than the balance warns and still saves (FR-17.8)', async () => {
    const { user } = await open('?type=capital_out&partner=2');
    await typeAmount(user, language, '10000');
    // His capital balance is 0, so the warning names the balance and the part above it.
    await waitFor(() => {
      expect(pageText()).toContain(
        message(language, 'err-capital-exceeds-balance', { balance: '0', over: '10,000' }),
      );
    });
    press('F10');
    await waitFor(() => {
      expect(partnersApi.saveCapitalOut).toHaveBeenCalledTimes(1);
    });
    expect(books.cash.store.cash).toBe(53484 - 10000);
  });

  it('cash below zero asks before capital is taken back; putting it in never asks', async () => {
    books.cash.store.cash = 500;
    const { user } = await open('?type=capital_out&partner=2');
    await typeAmount(user, language, '1000');
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'common-cash-after', { amount: '-500' }));
    });
    press('F10');
    const no = await screen.findByRole('button', {
      name: startsWith(message(language, 'common-cash-confirm-no')),
    });
    await user.click(no);
    expect(partnersApi.saveCapitalOut).not.toHaveBeenCalled();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'cashbook-type-capital-in')) }),
    );
    press('F10');
    await waitFor(() => {
      expect(partnersApi.saveCapitalIn).toHaveBeenCalledTimes(1);
    });
    expect(
      screen.queryByRole('button', {
        name: startsWith(message(language, 'common-cash-confirm-no')),
      }),
    ).toBeNull();
  });

  it('a missing partner is told at its field only after the user tries to save', async () => {
    const { user } = await open();
    await typeAmount(user, language, '1000');
    const field = await dropdown(language, 'cashbook-field-partner');
    await waitFor(() => {
      expect(
        screen.getByLabelText(named(message(language, 'cashbook-field-amount'))),
      ).toBeInTheDocument();
    });
    expect(field).not.toHaveAttribute('aria-invalid', 'true');
    press('F10');
    await waitFor(() => {
      expect(field).toHaveAttribute('aria-invalid', 'true');
    });
    expect(pageText()).toContain(message(language, 'err-partner-required'));
    expect(books.money.store.capitalSaves).toHaveLength(0);
  });

  it('the history failing to load says so with a retry that works', async () => {
    vi.mocked(partnersApi.capitalHistory).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    await open('?type=capital_in&partner=2');
    const title = await screen.findByText(named(message(language, 'partners-capital-failed')));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    const user = userEvent.setup();
    await user.click(
      within(alert).getByRole('button', { name: startsWith(message(language, 'common-retry')) }),
    );
    expect(
      await screen.findByText(named(message(language, 'partners-capital-empty'))),
    ).toBeInTheDocument();
  });

  it('switching the language in the middle of an entry loses nothing, the type included', async () => {
    const { user } = await open('?type=capital_out&partner=2');
    await typeAmount(user, language, '700');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'cashbook-field-amount')).value).toBe('700');
    expect(await dropdown(other, 'cashbook-field-partner')).toHaveTextContent(
      nameOf(other, 'ب', 'Partner B'),
    );
    const group = screen.getByRole('group', {
      name: named(message(other, 'partners-capital-type')),
    });
    expect(
      within(group).getByRole('button', {
        name: named(message(other, 'cashbook-type-capital-out')),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('the keyboard alone does it: Enter walks date, partner and amount; F10 saves (F30)', async () => {
    const { user } = await open();
    const date = await labelled(language, 'cashbook-field-date');
    await waitFor(() => {
      expect(date).toHaveFocus();
    });
    await user.keyboard('{Enter}');
    const partner = await dropdown(language, 'cashbook-field-partner');
    await waitFor(() => {
      expect(partner).toHaveFocus();
    });
    await user.keyboard('{Enter}');
    await user.click(await screen.findByRole('option', { name: named(partnerB) }));
    const amount = await labelled(language, 'cashbook-field-amount');
    await user.click(amount);
    await user.keyboard('50000');
    press('F10');
    await waitFor(() => {
      expect(partnersApi.saveCapitalIn).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(partnersApi.saveCapitalIn).mock.calls[0]?.[0]).toMatchObject({
      partner_id: 2,
      amount: '50000',
    });
  });
});
