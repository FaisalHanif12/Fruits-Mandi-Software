// Plan 97 item 2 (PRD FR-12.52, FR-12.6, Umar 24-11-2026): on a sold out lot (the backend's `balance_qty` is 0) no key makes a
// new sale line: not `↓` on the last line, not `Enter` on its last stop, not the "+" button. Each try shows "All of this
// seller's goods are sold" for 5 seconds, and again on the next try. A lot with goods left still takes a new line by `↓`.
// The screen against the fake backend of Event 1, with real keys, in the Urdu and the English app.

import { screen, waitFor } from '@testing-library/react';
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
  cellOf,
  focusedIn,
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

describe.each(LANGUAGES)('a sold out lot takes no new line by any key (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });
  const say = (id: string) => message(language, id);
  const rowCount = () => salesGrid(language).querySelectorAll('.ag-row').length;
  const note = () => screen.queryByText((text) => visible(text) === say('bikri-sales-sold-out'));
  /** Both lines typed and the lot is sold out by the backend's answer (50 of 50). */
  async function soldOutLot() {
    const { user } = await openBook(language);
    await typeEvent1Lot(user, language);
    await typeTwoLines(user, language);
    await waitFor(() => {
      expect(screen.getAllByText((text) => visible(text) === '18,684').length).toBeGreaterThan(0);
    }, SLOW_MS);
    return user;
  }

  it('↓ on the last line adds no line, shows the words, hides them after 5 seconds, and shows them again on the next try', async () => {
    const user = await soldOutLot();
    const sales = salesGrid(language);
    const before = rowCount();
    const last = before - 1;
    await user.click(cellOf(sales, last, 'qty'));
    await user.keyboard('{ArrowDown}');
    expect(rowCount()).toBe(before);
    expect(note()).not.toBeNull();
    // The cursor stays where it was.
    expect(focusedIn(sales)).toBe(`${String(last)}:qty`);
    // The words hide by themselves after 5 seconds.
    await waitFor(
      () => {
        expect(note()).toBeNull();
      },
      { timeout: NOTICE_MS + 3_000 },
    );
    // The same key again: the words are back for another 5 seconds, and still no line.
    await user.keyboard('{ArrowDown}');
    expect(note()).not.toBeNull();
    expect(rowCount()).toBe(before);
  }, 30_000);

  it('Enter on the last stop of the last line adds no line and shows the words', async () => {
    const user = await soldOutLot();
    const sales = salesGrid(language);
    const before = rowCount();
    const last = before - 1;
    await user.click(cellOf(sales, last, 'paid'));
    await user.keyboard('{Enter}');
    expect(rowCount()).toBe(before);
    expect(note()).not.toBeNull();
  });

  it('the + button adds no line and shows the words', async () => {
    const user = await soldOutLot();
    const before = rowCount();
    await user.click(
      screen.getByRole('button', { name: (name) => visible(name) === say('bikri-sales-add-row') }),
    );
    expect(rowCount()).toBe(before);
    expect(note()).not.toBeNull();
  });

  it('a lot with goods left still takes a new line by ↓, and shows no words', async () => {
    const { user } = await openBook(language);
    await typeEvent1Lot(user, language);
    const sales = salesGrid(language);
    // One line of 20 of the 50: 30 are left.
    await user.keyboard('0{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    await user.keyboard('20{Enter}600{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:paid');
    });
    const before = rowCount();
    await waitFor(() => {
      expect(screen.getAllByText((text) => visible(text) === '12,880').length).toBeGreaterThan(0);
    }, SLOW_MS);
    await user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(rowCount()).toBe(before + 1);
    }, SLOW_MS);
    expect(note()).toBeNull();
  });
});
