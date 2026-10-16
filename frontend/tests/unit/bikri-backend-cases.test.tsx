// What the real backend says in the cases of the T2 contract (plan 12, T4): a one-time seller paid
// on the spot, shares that do not total 100%, an item sold by weight, a sale with no lines, and
// credit that is not allowed. Each answer is the shape the backend sends; the screen only shows it.
// Urdu and English.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { money } from '../helpers/accounts';
import { previewFor } from '../helpers/bikri-data';
import {
  cellOf,
  focusedIn,
  lotsGrid,
  named,
  openBook,
  salesGrid,
  textOf,
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
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
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

  describe('a one-time seller is paid on the spot (PRD FR-13.10)', () => {
    /** While نقد is empty the backend fills it with the whole seller credit and says so. */
    const pukhtaPaid = (draft: BikriDraft): BikriPreview => {
      const preview = previewFor(draft);
      const totals = preview.lots[0]?.totals;
      if (totals !== null && totals !== undefined && draft.lots[0]?.cash_paid === null) {
        totals.cash_paid = money(19316);
        totals.workings.cash_paid = {
          message_id: 'working-cash-pukhta',
          args: [{ name: 'result', value: { kind: 'money', paisa: money(19316) } }],
          rule_id: null,
        };
      }
      return preview;
    };

    async function typeLotWithoutCash(user: Awaited<ReturnType<typeof openBook>>['user']) {
      await user.keyboard('LES-1234{Enter}');
      await screen.findByRole('combobox', { name: named(message(language, 'glossary-zamindar')) });
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'seller')).not.toBe('');
      });
      await user.keyboard('{Enter}{Enter}{Enter}');
      await waitFor(() => {
        expect(focusedIn(lotsGrid(language))).toBe('0:seller');
      });
      await waitFor(() => {
        expect(textOf(lotsGrid(language), 0, 'packing')).not.toBe('');
      });
      await user.keyboard('{Enter}{Enter}{Enter}50{Enter}3000{Enter}5{Enter}50{Enter}{Enter}');
    }

    it('shows his net in the نقد cell, with its working, and a value typed there is kept as typed', async () => {
      const { backend, user } = await openBook(language, pukhtaPaid);
      await typeLotWithoutCash(user);
      const lots = lotsGrid(language);
      await waitFor(() => {
        expect(textOf(lots, 0, 'cash')).toBe('19,316');
      });
      // The cell was left empty: the backend's number is shown, and nothing was typed for him.
      expect(backend.lastDraft().lots[0]?.cash_paid).toBeNull();
      await user.click(
        within(cellOf(lots, 0, 'cash')).getByRole('button', {
          name: named(message(language, 'calc-info-label')),
        }),
      );
      await waitFor(() => {
        expect(pageText()).toContain(
          message(language, 'working-cash-pukhta', { result: '19,316' }),
        );
      });
      await user.keyboard('{Escape}');
      // He types a value: it is sent as typed, and the cell shows it.
      await user.click(cellOf(lots, 0, 'cash'));
      await user.keyboard('1000{Enter}');
      await waitFor(() => {
        expect(backend.lastDraft().lots[0]?.cash_paid).toBe('1000');
      });
      await waitFor(() => {
        expect(textOf(lots, 0, 'cash')).toBe('1,000');
      });
    });
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
    await user.keyboard('LES-1234');
    await screen.findByRole('heading', { name: named(message(language, 'bikri-summary-title')) });
    expect(pageText()).not.toContain(message(language, 'err-no-sale-lines'));
  });

  it('credit that is not allowed turns the payment cell red and says why', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      errors: [
        {
          code: 'ERR_CREDIT_NOT_ALLOWED',
          field: 'lots[0].sales[0].payment_type',
          values: {},
        },
      ],
    }));
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'payment')).toHaveClass('cell-error');
    });
    expect(pageText()).toContain(message(language, 'err-credit-not-allowed'));
    expect(pageText()).not.toContain('ERR_');
  });
});
