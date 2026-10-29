// A lot whose seller is a garden (plan 17c follow-up, PRD FR-12.36, FR-7.14, Code Standards F30), in
// the Urdu and the English app. A garden's lot gives no نقد and has no advance tick, and the backend
// refuses a typed نقد on it. So the screen treats it like a lot of own goods: the نقد cell is not
// typed in, what was typed there before the seller became a garden goes, Enter walks past it to
// the sale lines, and the save sends no نقد and no advance tick. The backend proves the rule itself.

import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { AKRAM, ARSHAD, KARIM, PUKHTA_BUYER, PUKHTA_SELLER, RAHIM } from '../helpers/bikri';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  openBook,
  salesGrid,
  textOf,
  typeCodeAndEnter,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** The garden of E2E-40: an account under control 0306. */
const SHAHPUR = account(91, 'other', '0080', 'شاہ پور باغ', 'Shahpur Garden');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a garden as the seller of a lot (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const view = await openBook(language);
    const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER, SHAHPUR];
    fakeAccounts(everyone);
    fakeAccountSearch(everyone);
    return view;
  }

  /** Takes a seller in the seller cell of the first lot. */
  async function takeSeller(user: Awaited<ReturnType<typeof open>>['user'], code: string) {
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await typeCodeAndEnter(user, code);
  }

  it('the نقد typed for another seller goes when the seller becomes a garden, and the cell is not typed in', async () => {
    const { user } = await open();
    await takeSeller(user, RAHIM.code);
    await user.click(cellOf(lotsGrid(language), 0, 'cash'));
    await user.keyboard('500{Enter}');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('500');
    });
    await takeSeller(user, SHAHPUR.code);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('');
    });
    // Typing on the cell does nothing: it is not a cell to type in for this lot.
    await user.click(cellOf(lotsGrid(language), 0, 'cash'));
    await user.keyboard('700');
    expect(textOf(lotsGrid(language), 0, 'cash')).toBe('');
  });

  it('Enter on the last cell before نقد goes on to the sale lines, never stopping at the نقد cell', async () => {
    const { user } = await open();
    await takeSeller(user, SHAHPUR.code);
    await user.click(cellOf(lotsGrid(language), 0, 'munshiana'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
  });

  it('F10 sends no نقد and no advance tick for the garden’s lot, whatever was typed before', async () => {
    const { user, backend } = await open();
    await takeSeller(user, RAHIM.code);
    await user.click(cellOf(lotsGrid(language), 0, 'cash'));
    await user.keyboard('500{Enter}');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('500');
    });
    await takeSeller(user, SHAHPUR.code);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'cash')).toBe('');
    });
    await user.click(cellOf(lotsGrid(language), 0, 'qty'));
    await user.keyboard('50{Enter}');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'qty')).toBe('50');
    });
    press('F10');
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    });
    const sent = backend.saves[0]?.draft.lots[0];
    expect(sent?.seller_account_id).toBe(SHAHPUR.id);
    expect(sent?.cash_paid).toBeNull();
    expect(sent?.adjust_advance).toBe(false);
  });
});
