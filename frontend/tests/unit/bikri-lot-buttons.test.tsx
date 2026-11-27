// Plan 95 items 2 and 6 (PRD FR-12.52, FR-12.49, Umar 23-11-2026), on the بکری screen against the fake backend of Event 1:
// - a lot that is sold out: the "+" of its sale lines adds no line and says "All of this seller's goods are sold" for 5
//   seconds, again on each press (the lot's balance is the backend's `balance_qty`);
// - the small button beside the seller's name in each lot row selects that lot and brings its sale lines into view, or its
//   summary when it is sold out, and never opens the picker of the cell.
// In the Urdu and the English app.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { NOTICE_MS } from '../../src/features/bikri/notices';
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
  focusedIn,
  lotsGrid,
  openBook,
  salesGrid,
  typeEvent1Lot,
  typeTwoLines,
} from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the buttons of the lots and the sale lines (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });
  const say = (id: string) => message(language, id);
  const rowCount = () => salesGrid(language).querySelectorAll('.ag-row').length;
  const plus = () =>
    screen.getByRole('button', {
      name: (name) => visible(name) === say('bikri-sales-add-row'),
    });
  const soldOutNote = () =>
    screen.queryByText((text) => visible(text) === say('bikri-sales-sold-out'));

  it('the + of a sold out lot adds nothing, says why for a few seconds, and says it again on each press', async () => {
    const { user } = await openBook(language);
    await typeEvent1Lot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(screen.getAllByText((text) => visible(text) === '18,684').length).toBeGreaterThan(0);
    }, SLOW_MS);
    // The last Enter of the typed lines is a try to add a line after a sold out lot (plan 97 item 2): if the backend's answer
    // had come by then it said why, so the test waits for those words to go before it presses the + itself.
    await waitFor(
      () => {
        expect(soldOutNote()).toBeNull();
      },
      { timeout: NOTICE_MS + 4_000 },
    );
    const before = rowCount();
    await user.click(plus());
    expect(soldOutNote()).not.toBeNull();
    expect(rowCount()).toBe(before);
    // It goes by itself.
    await waitFor(
      () => {
        expect(soldOutNote()).toBeNull();
      },
      { timeout: NOTICE_MS + 4_000 },
    );
    // A second press shows it again, and still no line.
    await user.click(plus());
    expect(soldOutNote()).not.toBeNull();
    expect(rowCount()).toBe(before);
  }, 60_000);

  it('the + of a lot with goods left still adds a line', async () => {
    const { user } = await openBook(language);
    await typeEvent1Lot(user, language);
    const before = rowCount();
    await user.click(plus());
    await waitFor(() => {
      expect(rowCount()).toBe(before + 1);
    }, SLOW_MS);
    expect(soldOutNote()).toBeNull();
  });

  describe('the button beside the seller', () => {
    const showButton = () =>
      within(lotsGrid(language)).getByRole('button', {
        name: (name) => visible(name).startsWith(say('bikri-lot-show-lines').slice(0, 8)),
      });

    it('selects the lot, puts the cursor on its first sale line and brings the lines into view, without opening the picker', async () => {
      const scrolled = vi.fn();
      Element.prototype.scrollIntoView = function scroll(this: Element) {
        scrolled(this.id);
      };
      const { user } = await openBook(language);
      await typeEvent1Lot(user, language);
      await user.click(showButton());
      await waitFor(() => {
        expect(focusedIn(salesGrid(language))).toBe('0:buyer');
      }, SLOW_MS);
      await waitFor(() => {
        expect(scrolled).toHaveBeenCalledWith('bikri-sales-heading');
      });
      // The seller's picker did not open: no list of sellers is on screen.
      expect(screen.queryAllByRole('option')).toHaveLength(0);
    });

    it('a sold out lot shows its summary instead', async () => {
      const scrolled = vi.fn();
      Element.prototype.scrollIntoView = function scroll(this: Element) {
        scrolled(this.id);
      };
      const { user } = await openBook(language);
      await typeEvent1Lot(user, language);
      await typeTwoLines(user, language);
      await waitFor(() => {
        expect(screen.getAllByText((text) => visible(text) === '18,684').length).toBeGreaterThan(0);
      }, SLOW_MS);
      await user.click(showButton());
      await waitFor(() => {
        expect(scrolled).toHaveBeenCalledWith('bikri-summary-heading');
      });
    });
  });
});
