// Robustness: F10 pressed twice on a new account gives one account (NFR-REL-1; the double click
// and double Enter of the robustness list). Owner: `robustness`.
//
// A person who is not sure the first F10 was taken presses it again. If the save is still on its
// way, the second press must do nothing. Two saves of a new account make two accounts with two
// codes and the same names: the backend only warns about a duplicate name (PRD FR-10.7). The
// backend has no request number, so the screen is the only guard.
//
// A slow fake backend holds the save open for 400 ms, as a busy disk would; the second press
// comes 60 ms after the first, as in a real double tap.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, fakeAccounts, money } from '../helpers/accounts';
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
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2880, 'dr'),
  credit_limit_paisa: money(50000),
});

const named = (text: string) => (name: string) => visible(name) === text;
const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { tariq: ['طارق'], mehmood: ['محمود'] },
          toEnglish: { طارق: 'Tariq', محمود: 'Mehmood' },
          rulesOnly: [],
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
  fakeMasters({});
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(['ur', 'en'] as const)('a new account when F10 is pressed twice (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('sends one save to the backend and makes one account', async () => {
    const store = fakeAccounts([AKRAM]);
    const real = vi.mocked(accountApi.accountsSave).getMockImplementation();
    vi.mocked(accountApi.accountsSave).mockImplementation(async (...args) => {
      await wait(400);
      if (real === undefined) throw new Error('no fake save');
      return real(...args);
    });
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const nameField = await screen.findByLabelText<HTMLInputElement>(
      named(
        message(language, language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en'),
      ),
    );
    await user.click(nameField);
    await user.keyboard(language === 'ur' ? 'tariq mehmood{Enter}' : 'Tariq Mehmood');
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'accounts-field-name-en')))
          .value,
      ).toBe('Tariq Mehmood');
    });
    press('F10');
    await wait(60);
    press('F10');
    await wait(60);
    press('F10');
    await wait(900);
    expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    expect(store.rows).toHaveLength(2);
  });
});
