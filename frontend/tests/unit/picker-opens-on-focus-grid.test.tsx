// Plan 46, F1 and F3 in the grids of the بکری بک (PRD FR-7.16 of v1.14): a picker cell of the lots
// and of the sale lines opens its list as soon as it gets the focus, by Enter, Tab, an arrow or a
// click. While the list is open ↑ ↓ choose, Enter picks and goes on, ← → and Tab leave the cell and
// Esc closes the list; with the list closed ↑ ↓ move between rows again. Both languages, by
// keyboard and by mouse, against the fake backend of Event 1.

import { act, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
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
  lotsGrid,
  named,
  names,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
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

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const options = () => screen.queryAllByRole('option');
const optionText = (index: number) => visible(options()[index]?.textContent);
const selected = () =>
  options().filter((option) => option.getAttribute('aria-selected') === 'true');

/** The key that leaves a cell toward the next column: the next column is on the left in Urdu. */
const TOWARD_QTY = (language: Language) => (language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');

const buyerListCalls = () =>
  vi
    .mocked(accountApi.searchAccounts)
    .mock.calls.filter(([input]) => input.scope === 'buyer' && input.text === '');

describe.each(LANGUAGES)('picker cells of the lots and sale lines (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('Enter onto the lot seller opens its list, with his name highlighted; Enter keeps him and the item list opens next', async () => {
    const { user } = await openBook(language);
    await user.keyboard('LES-1234{Enter}');
    await waitFor(() => {
      expect(
        screen.getByRole('combobox', { name: named(message(language, 'glossary-zamindar')) }),
      ).toHaveValue(names(language).rahim);
    });
    await user.keyboard('{Enter}{Enter}');
    const lots = lotsGrid(language);
    await waitFor(() => {
      expect(focusedIn(lots)).toBe('0:seller');
    });
    // The list is open at once, and the seller the lot already has is the highlighted row.
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
    await waitFor(() => {
      expect(selected()).toHaveLength(1);
    });
    expect(visible(selected()[0]?.textContent)).toContain(names(language).rahim);
    // Enter keeps the seller (it does not swap him for another) and goes on to the item, whose
    // list opens in its turn.
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(lots)).toBe('0:item');
    });
    expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(0);
    });
  });

  it('Enter from the lot onto the buyer opens the list; ↑ ↓ choose, Enter picks and moves to the quantity', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    expect(buyerListCalls().length).toBeGreaterThan(0);
    // Both regular buyers of Event 1 are listed, in the language of the screen.
    const shown = options().map((option) => visible(option.textContent));
    expect(shown.some((row) => row.includes(names(language).akram))).toBe(true);
    expect(shown.some((row) => row.includes(names(language).arshad))).toBe(true);
    // An empty cell highlights none, so a stray Enter picks nothing; ↓ points at the first row and
    // the next ↓ at the second.
    expect(selected()).toHaveLength(0);
    await user.keyboard('{ArrowDown}');
    expect(options()[0]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{ArrowDown}');
    expect(options()[1]).toHaveAttribute('aria-selected', 'true');
    expect(options()[0]).toHaveAttribute('aria-selected', 'false');
    // The cursor stayed in the cell: ↓ chose, it did not move to another row.
    expect(focusedIn(sales)).toBe('0:buyer');
    const second = optionText(1);
    const wanted = second.includes(names(language).akram)
      ? names(language).akram
      : names(language).arshad;
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    expect(textOf(sales, 0, 'buyer')).toBe(wanted);
    // The quantity is not a picker: nothing is open there.
    expect(options()).toHaveLength(0);
  });

  it('Esc closes the list, and then ↓ moves to the next row, whose list opens at once', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
    // Closed: ↓ is a key of the grid again. It is the last row, so a new row is made.
    await user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    });
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
  });

  it('← or → leaves the cell toward the next column; Tab leaves too', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    await user.keyboard(TOWARD_QTY(language));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    expect(options()).toHaveLength(0);
    // Back to the buyer: its list opens again. Tab goes on to the quantity.
    await user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    await user.keyboard('{Tab}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    expect(options()).toHaveLength(0);
  });

  it('a click on a picker cell opens its list at the first click; a click on another cell opens nothing', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    await user.click(cellOf(sales, 0, 'qty'));
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    });
    expect(options()).toHaveLength(0);
    vi.mocked(accountApi.searchAccounts).mockClear();
    // One click is enough; it does not need a second one on the selected cell.
    await user.click(cellOf(sales, 0, 'buyer'));
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    expect(buyerListCalls().length).toBeGreaterThan(0);
    // Esc closes it; a click on the cell with the focus opens it again.
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(options()).toHaveLength(0);
    });
  });

  it('a language switch with the list open loses nothing the lot holds', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    await waitFor(() => {
      expect(options().length).toBeGreaterThan(1);
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    // The grids are made again from the draft: the lot reads the same in the other language.
    await waitFor(() => {
      expect(textOf(lotsGrid(other), 0, 'seller')).toBe(names(other).rahim);
    });
    expect(textOf(lotsGrid(other), 0, 'qty')).toBe('50');
    expect(textOf(lotsGrid(other), 0, 'packing')).toBe(names(other).box);
  });
});
