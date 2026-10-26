// Plan 32, R7 in a grid: a mouse click on a picker cell of the sale lines opens the picker with the
// full list of that kind, like a click on the field of a form. Arriving on the cell by keyboard
// opens nothing (PRD 7.2): Enter moved the cursor there in typeHeaderAndLot, and no list was open.
// Both languages, against the fake backend of Event 1.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  cellOf,
  focusedIn,
  names,
  openBook,
  salesGrid,
  typeHeaderAndLot,
} from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a click on a picker cell of the sale lines (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const buyerListsCalls = () =>
    vi
      .mocked(accountApi.searchAccounts)
      .mock.calls.filter(([input]) => input.scope === 'buyer' && input.text === '');

  it('Enter and the arrow keys never open the list; a click that only selects the cell opens nothing; a click on the selected cell lists every buyer', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    // The cursor is on the buyer cell of the first line, put there by Enter: nothing is open.
    expect(screen.queryAllByRole('option')).toHaveLength(0);
    expect(buyerListsCalls()).toHaveLength(0);

    // Moving between cells by keys, in every direction, opens nothing (PRD 7.2).
    await user.keyboard('{ArrowRight}{ArrowDown}{ArrowLeft}{ArrowUp}');
    await waitFor(() => {
      expect(focusedIn(sales)).not.toBeNull();
    });
    expect(screen.queryAllByRole('option')).toHaveLength(0);
    expect(buyerListsCalls()).toHaveLength(0);

    // A first click on another cell only selects it.
    await user.click(cellOf(sales, 0, 'qty'));
    await user.click(cellOf(sales, 0, 'buyer'));
    expect(screen.queryAllByRole('option')).toHaveLength(0);
    expect(buyerListsCalls()).toHaveLength(0);

    // The click on the cell that already has the focus opens the picker with the full list.
    await user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(screen.getAllByRole('option').length).toBeGreaterThan(1);
    });
    expect(buyerListsCalls().length).toBeGreaterThan(0);
    // Both regular buyers of Event 1 are in the list, in the language of the screen.
    const shown = screen.getAllByRole('option').map((option) => visible(option.textContent));
    expect(shown.some((row) => row.includes(names(language).akram))).toBe(true);
    expect(shown.some((row) => row.includes(names(language).arshad))).toBe(true);
  });
});
