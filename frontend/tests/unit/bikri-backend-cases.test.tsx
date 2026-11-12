// What the real backend says in the cases of the T2 contract (plan 12, T4): shares that do not total
// 100%, an item sold by weight, a sale with no lines, and credit that is not allowed. Each answer is the shape the backend sends; the screen only shows it.
// Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor } from '../helpers/bikri-data';
import {
  cellOf,
  named,
  openBook,
  salesGrid,
  textOf,
  typeCodeAndEnter,
  typeHeaderAndLot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const pageText = () => visible(document.body.textContent);

describe.each(LANGUAGES)('the backend’s cases (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  describe('the partners’ shares do not total 100% (PRD FR-17.1)', () => {
    const sharesBad = (draft: BikriDraft): BikriPreview => ({
      ...previewFor(draft),
      split: [],
      errors: [{ code: 'ERR_SHARES_NOT_100', field: 'split', values: { total_bps: 9000 } }],
    });

    it('says so in words and points to the partner setup; the totals are still there', async () => {
      const { user, router } = await openBook(language, sharesBad);
      await typeHeaderAndLot(user, language);
      await typeTwoLines(user, language);
      await waitFor(() => {
        expect(pageText()).toContain(message(language, 'err-shares-not-100', { total: '90' }));
      });
      // The numbers of the sale are still shown: only the split is empty.
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('12,880');
      });
      await user.click(
        screen.getByRole('button', { name: named(message(language, 'bikri-shares-open-setup')) }),
      );
      // The sale has typed work in it, so leaving asks first (PRD FR-7.9); the user leaves.
      await user.click(
        await screen.findByRole('button', { name: named(message(language, 'layout-save-leave')) }),
      );
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/partners/setup');
      });
    });
  });

  describe('an item sold by weight (LotPreview.sold_by_weight)', () => {
    const byWeight = (sold: boolean) => (draft: BikriDraft) => {
      const preview = previewFor(draft);
      for (const lot of preview.lots) lot.sold_by_weight = sold;
      return preview;
    };

    it('asks for the weight on each line when the backend says the item is sold by weight', async () => {
      const { user } = await openBook(language, byWeight(true));
      await typeHeaderAndLot(user, language);
      await typeTwoLines(user, language);
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('12,880');
      });
      await user.click(cellOf(salesGrid(language), 0, 'qty'));
      expect(
        await screen.findAllByRole('textbox', {
          name: named(message(language, 'bikri-line-weight')),
        }),
      ).not.toHaveLength(0);
    });

    it('shows no weight field on a line for an item sold by نگ', async () => {
      const { user } = await openBook(language, byWeight(false));
      await typeHeaderAndLot(user, language);
      await typeTwoLines(user, language);
      await waitFor(() => {
        expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('12,880');
      });
      await user.click(cellOf(salesGrid(language), 0, 'qty'));
      await screen.findByRole('textbox', {
        name: named(message(language, 'bikri-line-seller-comm')),
      });
      expect(
        screen.queryByRole('textbox', { name: named(message(language, 'bikri-line-weight')) }),
      ).not.toBeInTheDocument();
    });
  });

  it('a sale with no lines is not shown as an error while the user is still typing', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      errors: [{ code: 'ERR_NO_SALE_LINES', field: 'lots', values: {} }],
    }));
    // The first typed work is the seller of the first lot (the date alone is not typed work).
    await user.keyboard('{Enter}');
    await typeCodeAndEnter(user, '0');
    await screen.findByRole('heading', { name: named(message(language, 'bikri-summary-title')) });
    expect(pageText()).not.toContain(message(language, 'err-no-sale-lines'));
  });

  it('money paid above the buyer total says why on the line, with the most allowed', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      errors: [
        {
          code: 'ERR_PAID_EXCEEDS_TOTAL',
          field: 'lots[0].sales[0].paid',
          values: { max: 12_880 },
        },
      ],
    }));
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(pageText()).toContain(message(language, 'err-paid-exceeds-total', { max: '12,880' }));
    });
    expect(pageText()).not.toContain('ERR_');
  });
});
