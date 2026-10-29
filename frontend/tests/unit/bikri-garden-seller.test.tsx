// An open garden as the seller of a lot (plan 17c, PRD FR-12.36, E2E-40), in the Urdu and the English
// app. The seller cell of the lots grid asks the backend with the scope `lot_seller`, which also
// answers the open gardens (control 0306) next to the sellers; the buyer cell of the sale lines
// asks with the scope `buyer`, which never answers a garden. A closed garden is not offered: the
// backend leaves it out of the answer. The screen filters nothing itself: this proves the cells ask
// with those scopes, show a garden the backend offers, take it with Enter, and that a garden is not
// offered in a buyer field. The backend proves the rule itself.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import { AKRAM, ARSHAD, KARIM, PUKHTA_BUYER, PUKHTA_SELLER, RAHIM } from '../helpers/bikri';
import { cellOf, lotsGrid, openBook, salesGrid, textOf } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { visible } from '../helpers/render';

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

/** The garden of E2E-40: an account of type "other" under control 0306 with the garden's code, 0080. */
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

describe.each(LANGUAGES)('an open garden as the seller of a lot (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const gardenName = language === 'ur' ? SHAHPUR.name_ur : SHAHPUR.name_en;

  async function open() {
    const view = await openBook(language);
    const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER, SHAHPUR];
    fakeAccounts(everyone);
    fakeAccountSearch(everyone);
    return view;
  }

  it('the seller cell of the lots grid offers the garden, and Enter takes it as the seller', async () => {
    const { user } = await open();
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(language === 'ur' ? 'شاہ' : 'shahpur');
    const options = await screen.findAllByRole('option');
    expect(options.map((option) => visible(option.textContent)).join(' ')).toContain(gardenName);
    // The cell asked with the scope of the seller of a lot.
    expect(accountApi.searchAccounts).toHaveBeenCalledWith(
      expect.objectContaining({ scope: 'lot_seller' }),
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
    });
  });

  it('its full code works in the seller cell', async () => {
    const { user } = await open();
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(`${SHAHPUR.code}{Enter}`);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(gardenName);
    });
  });

  it('a garden is not offered in the buyer cell of a sale line, which asks with the scope buyer', async () => {
    const { user } = await open();
    // A lot with a seller first, so its sale lines can be typed.
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(`${RAHIM.code}{Enter}`);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).not.toBe('');
    });
    await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
    await user.keyboard(language === 'ur' ? 'شاہ' : 'shahpur');
    await waitFor(() => {
      expect(accountApi.searchAccounts).toHaveBeenCalledWith(
        expect.objectContaining({ scope: 'buyer' }),
      );
    });
    expect(
      screen
        .queryAllByRole('option')
        .map((option) => visible(option.textContent))
        .join(' '),
    ).not.toContain(gardenName);
    expect(within(salesGrid(language)).queryByText(gardenName)).toBeNull();
  });

  it('a closed garden is not in the backend’s answer, so the seller cell does not offer it', async () => {
    const { user } = await open();
    // The backend leaves a closed garden out of the accounts it answers with.
    const open_ones = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER];
    fakeAccounts(open_ones);
    fakeAccountSearch(open_ones);
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(language === 'ur' ? 'شاہ' : 'shahpur');
    await waitFor(() => {
      expect(accountApi.searchAccounts).toHaveBeenCalledWith(
        expect.objectContaining({ scope: 'lot_seller' }),
      );
    });
    expect(
      screen
        .queryAllByRole('option')
        .map((option) => visible(option.textContent))
        .join(' '),
    ).not.toContain(gardenName);
  });
});
