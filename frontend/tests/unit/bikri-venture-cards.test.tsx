// Plan 97 item 5 (PRD FR-12.52, FR-12.26, FR-12.36, Umar 24-11-2026): a بکری that holds a garden lot or a trip lot has one
// card for each garden and each trip in the summary, in the grid of the seller cards. The garden card: its name, پچھلا بقایا,
// آج کی صافی بکری, نیا بقایا (no موقع پر ادائیگی row). The trip card: its number, خام بکری, کٹوتی, آج کی صافی بکری, and the line
// "to the trip's partners by their share" (no partner figure). Every figure is the backend's, from the preview; the screen
// works out nothing. In the Urdu and the English app, whole view and the view of one lot.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { gardenCardsFor, tripCardsFor } from '../../src/features/bikri/venture-cards';
import { LANGUAGES, type Language } from '../../src/i18n';
import type { EntryRow } from '../../src/shared/grid';
import * as urduApi from '../../src/shared/urdu/api';
import { previewFor } from '../helpers/bikri-data';
import { chooseInSummary, named, openBook, typeEvent1Lot } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { GARDEN_ACCOUNT, gardenCard, tripCard, withCards } from '../helpers/venture-cards';

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

/** The element, or a failed test that says what was missing. */
function found(element: HTMLElement | null): HTMLElement {
  if (element === null) throw new Error('the card is not on the screen');
  return element;
}

describe('which card belongs to which lot', () => {
  const lotRow = (rowId: string, sellerId: number): EntryRow => ({
    rowId,
    typed: {},
    picked: { seller: { id: sellerId, code: 'x', name_ur: 'x', name_en: 'x' } },
  });
  const garden = gardenCard();
  const otherGarden = gardenCard({ id: 6, account: 88 });
  const trip = tripCard();

  it('a garden card goes with the lots whose seller is the garden’s account', () => {
    const inGarden = lotRow('a', GARDEN_ACCOUNT);
    const ordinary = lotRow('b', 19);
    expect(gardenCardsFor([inGarden], [garden, otherGarden])).toEqual([garden]);
    expect(gardenCardsFor([ordinary], [garden, otherGarden])).toEqual([]);
    expect(gardenCardsFor([inGarden, ordinary], [garden, otherGarden])).toEqual([garden]);
  });

  it('a trip card goes with the lots whose preview names that trip', () => {
    const previewLots = [
      { key: 'a', own: { trip: trip.trip } },
      { key: 'b', own: null },
    ] as unknown as Parameters<typeof tripCardsFor>[1];
    expect(tripCardsFor([lotRow('a', 9)], previewLots, [trip])).toEqual([trip]);
    expect(tripCardsFor([lotRow('b', 9)], previewLots, [trip])).toEqual([]);
  });
});

describe.each(LANGUAGES)(
  'the garden and trip cards on the بکری page (%s)',
  (language: Language) => {
    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });
    const side = (id: 'dr' | 'cr') => m(id === 'dr' ? 'common-side-dr' : 'common-side-cr');
    const rows = (card: HTMLElement) =>
      Array.from(card.querySelectorAll('dl > div')).map((row) => ({
        label: visible(row.querySelector('dt')?.textContent),
        value: visible(row.querySelector('dd')?.textContent),
      }));
    const gardenBlock = () => document.querySelector<HTMLElement>('[data-garden-card]');
    const tripBlock = () => document.querySelector<HTMLElement>('[data-trip-card]');

    async function openWithCards() {
      const book = await openBook(language, (draft) =>
        withCards(previewFor(draft), [gardenCard()], [tripCard()]),
      );
      await typeEvent1Lot(book.user, language);
      return book;
    }

    async function showWholeSale(user: Awaited<ReturnType<typeof openWithCards>>['user']) {
      const summary = screen
        .getByRole('heading', { name: named(m('bikri-summary-title')) })
        .closest('section');
      if (summary === null) throw new Error('no summary');
      await chooseInSummary(user, summary, m('bikri-summary-whole'));
    }

    it('the whole sale shows a card for the garden and a card for the trip, with the backend’s figures', async () => {
      const { user } = await openWithCards();
      await showWholeSale(user);
      await waitFor(() => {
        expect(gardenBlock()).not.toBeNull();
        expect(tripBlock()).not.toBeNull();
      });
      const garden = found(gardenBlock());
      expect(visible(garden.querySelector('h3')?.textContent)).toBe(
        language === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden',
      );
      expect(rows(garden)).toEqual([
        { label: m('bikri-seller-paid-previous'), value: `12,000${side('cr')}` },
        { label: m('bikri-seller-paid-net'), value: '56,400' },
        { label: m('bikri-seller-paid-new'), value: expect.stringContaining('68,400') as string },
      ]);
      expect(rows(garden).at(-1)?.value).toContain(side('cr'));
      // A garden lot takes no cash, so there is no "paid on the spot" row.
      expect(visible(garden.textContent)).not.toContain(m('bikri-seller-paid-paid'));

      const trip = found(tripBlock());
      expect(visible(trip.querySelector('h3')?.textContent)).toBe(
        m('bikri-trip-card-title', { code: '0070' }).replace(/<\/?code>/g, ''),
      );
      expect(rows(trip).map((row) => row.label)).toEqual([
        m('glossary-kham-bikri'),
        m('bikri-trip-card-deductions'),
        m('bikri-seller-paid-net'),
      ]);
      expect(rows(trip).map((row) => row.value.replace(/[^0-9,]/g, ''))).toEqual([
        '9,000',
        '540',
        '8,460',
      ]);
      // The line that says where the credit goes, and no figure of any partner.
      expect(visible(trip.textContent)).toContain(m('bikri-trip-card-partners'));
    });

    it('the cards stand in the same grid as the seller cards', async () => {
      const { user } = await openWithCards();
      await showWholeSale(user);
      await waitFor(() => {
        expect(gardenBlock()).not.toBeNull();
      });
      const grid = found(gardenBlock()).parentElement;
      expect(grid?.contains(tripBlock())).toBe(true);
      expect(grid?.querySelector('[data-seller-paid]')).not.toBeNull();
    });

    it('the view of an ordinary lot shows neither card', async () => {
      await openWithCards();
      // The lot of Event 1 is Rahim’s: it is neither a garden lot nor a trip lot.
      await waitFor(() => {
        expect(
          screen.getAllByRole('heading', { name: named(m('bikri-summary-title')) }),
        ).toHaveLength(1);
      });
      expect(gardenBlock()).toBeNull();
      expect(tripBlock()).toBeNull();
    });
  },
);
