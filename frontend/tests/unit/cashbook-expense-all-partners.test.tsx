// The list under the خرچہ form with no partner chosen (plan 98, PRD FR-16.2, FR-19.11), in the Urdu and the English app,
// against a fake backend: type ذاتی خرچہ lists every partner's money taken (the command `partner_takes` with no
// partner), with the partner's name on each row, the reason, the amount, what was returned and what is still open, and
// the backend's totals, and not a request to choose a partner; type ذاتی خرچہ واپسی shows the same list; a partner
// chosen on the form shows his own list instead; the partner code keeps the list closed with its box. Before plan 98 the
// card said "choose a partner" and listed nothing. Every figure is the fake backend's: the screen adds nothing up.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import type { FakeCashbook } from '../helpers/cashbook';
import { choose, named, prepareCashbook } from '../helpers/cashbook-screen';
import { fail, leaveAppWindow, renderApp } from '../helpers/app';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

/** A take of one partner, saved before the screen is opened. */
function take(books: FakeCashbook, partnerId: number, amount: string, reason: string) {
  books.saveExpense({
    type: 'personal_expense',
    date: '10-10-2026',
    partner_id: partnerId,
    amount,
    reason,
    return_by: null,
    taken_by: null,
    narration: null,
    confirm_cash_below_zero: true,
  });
}

describe.each(LANGUAGES)('the خرچہ list with no partner chosen (%s)', (language) => {
  let books: FakeCashbook;

  beforeEach(() => {
    books = prepareCashbook(language);
  });

  const m = (id: string) => message(language, id);

  async function open(type: string) {
    const app = await renderApp(`/rokar/expense?type=${type}`);
    await screen.findByRole('heading', { level: 1, name: m('cashbook-expense-title') }, SLOW_MS);
    return { ...app, user: userEvent.setup() };
  }

  const listOf = () =>
    screen.findByRole('group', { name: named(m('cashbook-all-takes-title')) }, SLOW_MS);

  it('lists every partner’s money taken, with his name, his reason and the backend’s totals, not a request to choose', async () => {
    take(books, 1, '2000', 'گھر کا خرچ');
    take(books, 2, '500', 'wedding');
    await open('personal_expense');
    const list = await listOf();
    await waitFor(() => {
      const text = visible(list.textContent);
      // Both partners, each with the amount and the reason typed on his entry.
      expect(text).toContain(nameOf(language, 'الف', 'Partner A'));
      expect(text).toContain(nameOf(language, 'ب', 'Partner B'));
      expect(text).toContain('2,000');
      expect(text).toContain('500');
      expect(text).toContain('گھر کا خرچ');
      expect(text).toContain('wedding');
      // The total under the amount column is the backend's figure over every partner.
      expect(text).toContain('2,500');
    }, SLOW_MS);
    // The old request to choose a partner is gone; the command was asked for every partner, latest day first.
    expect(screen.queryByText(m('cashbook-takes-empty'))).toBeNull();
    expect(vi.mocked(cashbookApi.partnerTakes).mock.calls.at(-1)?.[0]).toMatchObject({
      partner_id: null,
      sort: { column: 'entry_date', direction: 'desc' },
    });
  });

  it('shows what was returned and what is still open for each take, with the totals', async () => {
    take(books, 1, '2000', 'home');
    books.saveExpense({
      type: 'personal_return',
      date: '10-10-2026',
      partner_id: 1,
      amount: '700',
      narration: null,
    });
    // ذاتی خرچہ واپسی shows the same list of takes, with what he gave back against them.
    await open('personal_return');
    const list = await listOf();
    await waitFor(() => {
      const text = visible(list.textContent);
      expect(text).toContain(nameOf(language, 'الف', 'Partner A'));
      expect(text).toContain('2,000');
      expect(text).toContain('700');
      expect(text).toContain('1,300');
    }, SLOW_MS);
  });

  it('says so in words when no partner took any money in the window, with what to do', async () => {
    await open('personal_expense');
    expect(
      await screen.findByText(m('cashbook-all-takes-empty'), undefined, SLOW_MS),
    ).toBeInTheDocument();
    expect(screen.getByText(m('cashbook-all-empty-help'))).toBeInTheDocument();
  });

  it('a partner chosen shows only his own list, and the list of every partner is gone', async () => {
    take(books, 1, '2000', 'home');
    take(books, 2, '500', 'wedding');
    const { user } = await open('personal_expense');
    await listOf();
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    const own = await screen.findByRole(
      'group',
      { name: named(m('cashbook-takes-label')) },
      SLOW_MS,
    );
    await waitFor(() => {
      const text = visible(own.textContent);
      expect(text).toContain('2,000');
      expect(text).toContain('home');
      // The other partner's entry is not in his list.
      expect(text).not.toContain('wedding');
    }, SLOW_MS);
    expect(vi.mocked(cashbookApi.partnerTakes).mock.calls.at(-1)?.[0]).toMatchObject({
      partner_id: 1,
    });
    expect(screen.queryByRole('group', { name: named(m('cashbook-all-takes-title')) })).toBeNull();
  });

  it('while the partner code is not entered it asks for the code in the list’s place, never nothing', async () => {
    take(books, 1, '2000', 'home');
    vi.mocked(cashbookApi.partnerTakes).mockRejectedValue(fail({ code: 'ERR_PARTNER_LOCKED' }));
    await open('personal_expense');
    expect(
      await screen.findByText(m('partners-code-box-text'), undefined, SLOW_MS),
    ).toBeInTheDocument();
    expect(screen.queryByText('2,000')).toBeNull();
  });
});
