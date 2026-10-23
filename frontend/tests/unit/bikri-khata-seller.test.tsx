// A الگ کھاتہ as the seller of a lot (plan 17 follow-up, PRD FR-16.7, E2E-38), in the Urdu and the
// English app. The seller cell of the lots grid asks the backend with the scope `lot_seller`, which
// now also answers the separate khatas (control 0305) next to the sellers; the buyer cell of the
// sale lines asks with the scope `buyer`, which never answers a khata. The screen filters nothing
// itself: this proves the cells ask with those scopes, show a khata the backend offers, take it
// with Enter, and that a khata is not offered in a buyer field. The backend proves the rule itself.

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

/** The family khata of E2E-38: an account of type "other" under control 0305. */
const FAMILY_KHATA = account(81, 'other', '03050001', 'خاندانی کھاتہ', 'Family Khata');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('a الگ کھاتہ as the seller of a lot (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const khataName = language === 'ur' ? FAMILY_KHATA.name_ur : FAMILY_KHATA.name_en;

  async function open() {
    const view = await openBook(language);
    const everyone = [RAHIM, KARIM, AKRAM, ARSHAD, PUKHTA_BUYER, PUKHTA_SELLER, FAMILY_KHATA];
    fakeAccounts(everyone);
    fakeAccountSearch(everyone);
    return view;
  }

  it('the seller cell of the lots grid offers the khata, and Enter takes it as the seller', async () => {
    const { user } = await open();
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(language === 'ur' ? 'خاندانی' : 'family');
    const options = await screen.findAllByRole('option');
    expect(options.map((option) => visible(option.textContent)).join(' ')).toContain(khataName);
    // The cell asked with the scope of the seller of a lot.
    expect(accountApi.searchAccounts).toHaveBeenCalledWith(
      expect.objectContaining({ scope: 'lot_seller' }),
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(khataName);
    });
  });

  it('its full code works in the seller cell', async () => {
    const { user } = await open();
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(`${FAMILY_KHATA.code}{Enter}`);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).toBe(khataName);
    });
  });

  it('a khata is not offered in the buyer cell of a sale line, which asks with the scope buyer', async () => {
    const { user } = await open();
    // A lot with a seller first, so its sale lines can be typed.
    await user.click(await waitFor(() => cellOf(lotsGrid(language), 0, 'seller')));
    await user.keyboard(`${RAHIM.code}{Enter}`);
    await waitFor(() => {
      expect(textOf(lotsGrid(language), 0, 'seller')).not.toBe('');
    });
    await user.click(await waitFor(() => cellOf(salesGrid(language), 0, 'buyer')));
    await user.keyboard(language === 'ur' ? 'خاندانی' : 'family');
    await waitFor(() => {
      expect(accountApi.searchAccounts).toHaveBeenCalledWith(
        expect.objectContaining({ scope: 'buyer' }),
      );
    });
    // Nothing is listed for the khata: no option names it, in either scope list that is open.
    expect(
      screen
        .queryAllByRole('option')
        .map((option) => visible(option.textContent))
        .join(' '),
    ).not.toContain(khataName);
    expect(within(salesGrid(language)).queryByText(khataName)).toBeNull();
  });
});
