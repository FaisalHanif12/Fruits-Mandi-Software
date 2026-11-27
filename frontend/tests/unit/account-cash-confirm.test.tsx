// The account form when an opening in روکڑ would take cash below zero (PRD FR-6.7, plan 80 R4-4, and plan 99:
// a party's opening goes through روکڑ, FR-10.9 v1.60), and the "from روکڑ" note, in the Urdu and the English app. `accounts_save` answers ERR_CASH_BELOW_ZERO_NOT_CONFIRMED with the value
// `cash_after_paisa`; the form asks the question of FR-6.7 with that figure, No sends nothing, and the yes sends the
// same account again with `confirm_cash_below_zero`.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccounts, money } from '../helpers/accounts';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** A partner's capital account whose opening is in روکڑ (PRD 17.1 v1.43). */
const CAPITAL = account(61, 'other', '06020001', 'سرمایہ الف', 'Capital A', {
  opening_date: '2026-08-31',
  opening_cr_paisa: money(50_000),
});

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the account form and the cash question (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** Opens the account, edits its note, and makes the first save answer with the cash refusal. */
  const openAndSave = async () => {
    fakeAccounts([CAPITAL]);
    const saved = vi.mocked(accountApi.accountsSave).getMockImplementation();
    if (saved === undefined) throw new Error('no fake save');
    vi.mocked(accountApi.accountsSave).mockImplementation((draft, words, confirm) =>
      confirm === true
        ? saved(draft, words, confirm)
        : Promise.reject(
            fail({
              code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED',
              field: 'opening_cr',
              values: { cash_after_paisa: -500_000 },
            }),
          ),
    );
    await renderApp('/index/accounts/61');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-opening-cr'));
    await user.keyboard('{Control>}a{/Control}100');
    press('F10');
    return user;
  };

  it('asks with the cash after, with No focused; the yes sends the same account again confirmed', async () => {
    const user = await openAndSave();
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain(message(language, 'common-cash-confirm-title'));
    expect(visible(box.textContent)).toContain('-5,000');
    expect(
      within(box).getByRole('button', {
        name: (label) => visible(label).startsWith(message(language, 'common-cash-confirm-no')),
      }),
    ).toHaveFocus();
    expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(1);
    await user.click(
      within(box).getByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(2);
    });
    const calls = vi.mocked(accountApi.accountsSave).mock.calls;
    expect(calls.map(([, , confirm]) => confirm)).toEqual([false, true]);
    // The same account both times: the second is the yes to the first.
    expect(calls[1]?.[0]).toMatchObject({ id: 61, opening_cr: '100' });
    expect(
      (await screen.findAllByText(message(language, 'accounts-saved'))).length,
    ).toBeGreaterThan(0);
  });

  it('No sends nothing again and keeps what was typed', async () => {
    const user = await openAndSave();
    await screen.findByRole('alertdialog');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(1);
    expect((await labelled(language, 'accounts-field-opening-cr')).value).toBe('100');
  });
});

describe.each(LANGUAGES)('a party opening goes through روکڑ (%s)', (language) => {
  const m = (id: string) => message(language, id);

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const noteText = () => visible(document.body.textContent);

  it('says so under the opening of a new account, with the way the money moves as it is typed', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const debit = await labelled(language, 'accounts-field-opening-dr');
    expect(noteText()).toContain(m('accounts-opening-cash'));
    await user.click(debit);
    await user.keyboard('5000');
    expect(noteText()).toContain(m('accounts-opening-cash-out'));
    await user.clear(debit);
    await user.click(await labelled(language, 'accounts-field-opening-cr'));
    await user.keyboard('700');
    expect(noteText()).toContain(m('accounts-opening-cash-in'));
    expect(noteText()).not.toContain(m('accounts-opening-cash-out'));
  });

  it('says nothing about روکڑ for an account whose opening goes to the opening difference', async () => {
    fakeAccounts([CAPITAL]);
    await renderApp('/index/accounts/61');
    await labelled(language, 'accounts-field-opening-cr');
    expect(noteText()).not.toContain(m('accounts-opening-cash'));
    expect(noteText()).not.toContain(m('accounts-opening-cash-in'));
  });

  it('a new opening that takes cash below zero asks the cash question, and the yes saves it confirmed', async () => {
    fakeAccounts([]);
    const saved = vi.mocked(accountApi.accountsSave).getMockImplementation();
    if (saved === undefined) throw new Error('no fake save');
    vi.mocked(accountApi.accountsSave).mockImplementation((draft, words, confirm) =>
      confirm === true
        ? saved(draft, words, confirm)
        : Promise.reject(
            fail({
              code: 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED',
              field: 'opening_dr',
              values: { cash_after_paisa: -216_000 },
            }),
          ),
    );
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Mani');
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard('مانی');
    await user.click(await labelled(language, 'accounts-field-opening-dr'));
    await user.keyboard('30000');
    press('F10');
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain('-2,160');
    await user.click(
      within(box).getByRole('button', { name: message(language, 'common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(vi.mocked(accountApi.accountsSave)).toHaveBeenCalledTimes(2);
    });
    const calls = vi.mocked(accountApi.accountsSave).mock.calls;
    expect(calls.map(([, , confirm]) => confirm)).toEqual([false, true]);
    expect(calls[1]?.[0]).toMatchObject({ id: null, opening_dr: '30000' });
  });
});
