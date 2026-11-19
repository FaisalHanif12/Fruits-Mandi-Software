// Gate lane T4 (plan 80): a language switch in the middle of a خرچہ entry loses nothing (Code
// Standards F39, PRD FR-7.33, FR-7.67, 16.2, E32, E55, E56). Owner: `robustness`.
//
// `cashbook-expense.test.tsx` switches once on a shop expense. This test switches again and again,
// on the entry where it matters most: a partner's personal spending, which has a partner, an
// amount, a reason typed in Urdu letters, a return-by date and a taker. Then the type is changed
// after the switches (E56: the shared fields stay, the others are cleared) and the entry is saved:
// the backend gets what was typed, once. The screen works out no money.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  choose,
  dropdown,
  labelled,
  named,
  prepareCashbook,
  typeAmount,
} from '../helpers/cashbook-screen';
import { leaveAppWindow, press, renderApp } from '../helpers/app';
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

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);
const other = (language: Language): Language => (language === 'ur' ? 'en' : 'ur');
const switchTo = (language: Language) => {
  act(() => {
    useLanguageStore.setState({ language });
  });
};

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a language switch in the middle of a خرچہ entry (%s)', (language) => {
  beforeEach(() => {
    prepareCashbook(language);
  });

  async function open(type: string) {
    const app = await renderApp(`/rokar/expense?type=${type}`);
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'cashbook-expense-title'),
    });
    return { ...app, user: userEvent.setup() };
  }

  const typeGroup = (shown: Language) =>
    screen.getByRole('group', { name: named(message(shown, 'cashbook-expense-type')) });

  it('five switches keep the partner, the amount, the Urdu reason and the type, and F10 sends them once', async () => {
    const { user } = await open('personal_expense');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2000');
    await user.click(await labelled(language, 'cashbook-field-reason'));
    await user.keyboard('گھر کا خرچ');
    let shown: Language = language;
    for (let round = 0; round < 5; round += 1) {
      shown = other(shown);
      switchTo(shown);
      expect((await labelled(shown, 'cashbook-field-amount')).value).toBe('2,000');
      expect((await labelled(shown, 'cashbook-field-reason')).value).toBe('گھر کا خرچ');
      expect(await dropdown(shown, 'cashbook-field-partner')).toHaveTextContent(
        nameOf(shown, 'الف', 'Partner A'),
      );
      expect(
        within(typeGroup(shown)).getByRole('button', {
          name: named(message(shown, 'cashbook-type-personal-expense')),
        }),
      ).toHaveAttribute('aria-pressed', 'true');
    }
    press('F10');
    await waitFor(() => {
      expect(cashbookApi.savePersonalExpense).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(cashbookApi.savePersonalExpense).mock.calls[0]?.[0]).toMatchObject({
      partner_id: 1,
      amount: '2000',
      reason: 'گھر کا خرچ',
    });
    // Nothing else was saved by the switches.
    expect(cashbookApi.saveShopExpense).not.toHaveBeenCalled();
    expect(cashbookApi.savePersonalReturn).not.toHaveBeenCalled();
  });

  it('a type changed after a switch keeps the date and the amount and clears the partner (E56)', async () => {
    const { user } = await open('personal_expense');
    await choose(user, language, 'cashbook-field-partner', nameOf(language, 'الف', 'Partner A'));
    await typeAmount(user, language, '2000');
    switchTo(other(language));
    const shown = other(language);
    expect((await labelled(shown, 'cashbook-field-amount')).value).toBe('2,000');
    const date = (await labelled(shown, 'cashbook-field-date')).value;
    await user.click(
      within(typeGroup(shown)).getByRole('button', {
        name: named(message(shown, 'cashbook-type-shop-expense')),
      }),
    );
    // The date and the amount are shared by the types, so they stay.
    await waitFor(() => {
      expect(
        within(typeGroup(shown)).getByRole('button', {
          name: named(message(shown, 'cashbook-type-shop-expense')),
        }),
      ).toHaveAttribute('aria-pressed', 'true');
    });
    expect((await labelled(shown, 'cashbook-field-amount')).value).toBe('2,000');
    expect((await labelled(shown, 'cashbook-field-date')).value).toBe(date);
    // The partner field is not on a shop expense; the head is, and it starts empty.
    expect(
      screen.queryByRole('combobox', { name: named(message(shown, 'cashbook-field-partner')) }),
    ).toBeNull();
    expect(
      visible((await dropdown(shown, 'cashbook-field-head')).getAttribute('value') ?? ''),
    ).toBe('');
  });
});
