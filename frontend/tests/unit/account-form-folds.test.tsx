// The parts of the account form that fold and its Save bar (plan 38, G1, G5): the daily parts
// (account, names, contact, opening) are open, the credit and the "more" parts start shut, a shut
// part says how many of its fields hold a value, a part with a problem opens itself, a heading is a
// button reached by the keyboard, and Save is always in view under the scrolling form. Urdu and
// English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
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

const named = (text: string) => (name: string) => visible(name) === text;
const startsWith = (text: string) => (name: string) => visible(name).startsWith(text);

const NASIR = account(23, 'buyer', '0022', 'ناصر', 'Nasir', {
  opening_date: '2026-08-31',
  opening_cr_paisa: money(2500),
  credit_limit_paisa: money(50000),
  note: 'old customer',
});

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

const section = (language: Language, id: string) =>
  screen.findByRole('button', { name: startsWith(message(language, id)) });

describe.each(LANGUAGES)('the account form folds (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('opens with the daily parts open and the credit and "more" parts shut', async () => {
    fakeAccounts([NASIR]);
    await renderApp('/index/accounts/23');
    for (const id of ['accounts-section-contact', 'accounts-section-opening']) {
      expect(await section(language, id)).toHaveAttribute('aria-expanded', 'true');
    }
    for (const id of ['accounts-section-credit', 'accounts-section-more']) {
      expect(await section(language, id)).toHaveAttribute('aria-expanded', 'false');
    }
    expect(
      screen.queryByLabelText(named(message(language, 'accounts-field-note'))),
    ).not.toBeInTheDocument();
  });

  it('a shut part says how many of its fields hold a value, and opens with Enter on its heading', async () => {
    fakeAccounts([NASIR]);
    await renderApp('/index/accounts/23');
    const user = userEvent.setup();
    const more = await section(language, 'accounts-section-more');
    // The note is typed: one field behind the heading.
    expect(visible(more.textContent)).toContain('1');
    more.focus();
    await user.keyboard('{Enter}');
    expect(more).toHaveAttribute('aria-expanded', 'true');
    expect(
      await screen.findByLabelText<HTMLInputElement>(
        named(message(language, 'accounts-field-note')),
      ),
    ).toHaveValue('old customer');
  });

  it('a problem inside a shut part opens it, and the part cannot be shut while it is there', async () => {
    fakeAccounts([NASIR]);
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(
      fail({ code: 'ERR_REQUIRED', field: 'credit_limit', values: {} }),
    );
    await renderApp('/index/accounts/23');
    const user = userEvent.setup();
    expect(await section(language, 'accounts-section-credit')).toHaveAttribute(
      'aria-expanded',
      'false',
    );
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    const credit = await section(language, 'accounts-section-credit');
    await waitFor(() => {
      expect(credit).toHaveAttribute('aria-expanded', 'true');
    });
    await user.click(credit);
    expect(credit).toHaveAttribute('aria-expanded', 'true');
  });

  it('Save is under the scrolling form, not inside it, so it is always in view', async () => {
    fakeAccounts([NASIR]);
    await renderApp('/index/accounts/23');
    const save = await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'layout-save-save')),
    });
    expect(save.closest('form')).toBeNull();
    const scroller = document.querySelector('form')?.parentElement;
    expect(scroller?.className).toContain('overflow-auto');
    expect(scroller?.contains(save)).toBe(false);
  });
});
