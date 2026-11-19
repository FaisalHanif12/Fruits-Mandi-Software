// Plan 32, R7 in a grid, changed by plan 46 (PRD FR-7.16 of v1.14): a picker cell of the sale lines
// opens the picker with the full list of that kind as soon as it gets the focus, by a click and by
// a key (Enter moved the cursor there in typeHeaderAndLot). Before plan 46 a key opened nothing and
// a click had to be on the cell that already had the focus. The keyboard paths are in
// picker-opens-on-focus-grid.test.tsx. Both languages, against the fake backend of Event 1.

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

  it('Enter and the arrow keys put the cursor on a picker cell and its list opens; a click on another cell closes it; a click on the picker cell lists every buyer again', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    // The cursor is on the buyer cell of the first line, put there by Enter: its list is open.
    await waitFor(() => {
      expect(screen.getAllByRole('option').length).toBeGreaterThan(1);
    });
    expect(buyerListsCalls().length).toBeGreaterThan(0);

    // A click on another cell takes the cursor, and the list goes with the cell it belonged to.
    await user.click(cellOf(sales, 0, 'qty'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    await waitFor(() => {
      expect(screen.queryAllByRole('option')).toHaveLength(0);
    });
    vi.mocked(accountApi.searchAccounts).mockClear();

    // One click on the picker cell, which had no focus, opens the picker with the full list.
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
