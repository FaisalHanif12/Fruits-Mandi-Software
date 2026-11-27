// The quality of a sale line on the sale book (PRD FR-12.42, plan 58): a list cell after the rate in
// the grid, reached with the arrow keys, never an Enter stop (a line is still buyer, qty, rate). Its list
// opens by itself when the cursor reaches the cell, and leaving with no choice keeps it empty (PRD
// FR-12.52, plan 95 item 3, Umar 23-11-2026). It is sent with the line only when chosen, changes no figure, and survives a
// language switch in the middle of the entry. The payment type lives in the details of the line
// (option C), not in the grid, so the table has the same columns at every size. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  cellOf,
  focusedIn,
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
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the quality of a sale line (%s)', (language) => {
  const say = (id: string) => message(language, id);
  /** The key that moves to the next column in reading order: → in English, ← in Urdu. */
  const forward = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
  /** The list of the quality (the page also holds the list of global search, so it is asked by name). */
  const gradeList = () => screen.queryByRole('listbox', { name: say('glossary-darja') });

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function twoLines() {
    const view = await openBook(language);
    await typeHeaderAndLot(view.user, language);
    await typeTwoLines(view.user, language);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
    }, SLOW_MS);
    return view;
  }

  it('is a column after the rate, and the grid has no payment column: the table is the same everywhere', async () => {
    await twoLines();
    const heads = [...salesGrid(language).querySelectorAll('.ag-header-cell')].map((cell) =>
      cell.getAttribute('col-id'),
    );
    expect(heads.indexOf('grade')).toBe(heads.indexOf('rate') + 1);
    expect(heads).not.toContain('payment');
  });

  it('takes no Enter stop: buyer, qty, rate, paid, then the next line, four stops for each line', async () => {
    const { user } = await openBook(language);
    await typeHeaderAndLot(user, language);
    const sales = salesGrid(language);
    await user.keyboard('0{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('0:qty');
    }, SLOW_MS);
    // Four Enters fill the line (buyer, qty, rate, paid) and the cursor is on the next line's buyer. The quality is between
    // the rate and the paid cell and is not a stop. The line is 20 of 50, so the lot has goods left and a line is added (a
    // sold out lot adds none, plan 97 item 2: bikri-sold-out-keys).
    await user.keyboard('20{Enter}600{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('1:buyer');
    }, SLOW_MS);
  });

  /** Moves the cursor from the rate to the quality; its list opens by itself. */
  async function reachGrade(user: Awaited<ReturnType<typeof twoLines>>['user']) {
    await user.click(cellOf(salesGrid(language), 0, 'rate'));
    await user.keyboard(forward);
    await waitFor(() => {
      expect(screen.getByRole('listbox', { name: say('glossary-darja') })).toBeInTheDocument();
    }, SLOW_MS);
    expect(focusedIn(salesGrid(language))).toBe('0:grade');
  }

  it('is reached with the arrow key from the rate, its list opens by itself, chosen with the arrows and Enter, and sent', async () => {
    const { backend, user } = await twoLines();
    await reachGrade(user);
    // The empty choice is first; two ↓ reach the second word.
    await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.grade).toBe('second');
    }, SLOW_MS);
    expect(visible(cellOf(salesGrid(language), 0, 'grade').textContent)).toContain(
      say('glossary-darja-doyam'),
    );
    // The other line was not touched: it is sent with no quality at all.
    expect(backend.lastDraft().lots[0]?.sales[1]).not.toHaveProperty('grade');
    // The focus stayed on the cell, and no figure changed.
    expect(focusedIn(salesGrid(language))).toBe('0:grade');
    expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('12,880');
  });

  it('leaving with no choice keeps it empty: Esc closes the list, and an arrow goes on to the next cell', async () => {
    const { backend, user } = await twoLines();
    await reachGrade(user);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(gradeList()).not.toBeInTheDocument();
    }, SLOW_MS);
    expect(backend.lastDraft().lots[0]?.sales[0]).not.toHaveProperty('grade');
    expect(visible(cellOf(salesGrid(language), 0, 'grade').textContent).trim()).toBe('');
    // Going on with the arrow from the cell: the list does not trap the cursor, and nothing was chosen.
    await user.click(cellOf(salesGrid(language), 0, 'rate'));
    await user.keyboard(forward);
    await waitFor(() => {
      expect(gradeList()).toBeInTheDocument();
    }, SLOW_MS);
    await user.keyboard(forward);
    await waitFor(() => {
      expect(gradeList()).not.toBeInTheDocument();
    }, SLOW_MS);
    expect(focusedIn(salesGrid(language))).not.toBe('0:grade');
    expect(backend.lastDraft().lots[0]?.sales[0]).not.toHaveProperty('grade');
  });

  it('is cleared with Delete and then sent with no quality', async () => {
    const { backend, user } = await twoLines();
    await reachGrade(user);
    await user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.grade).toBe('first');
    }, SLOW_MS);
    await user.keyboard('{Delete}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]).not.toHaveProperty('grade');
    }, SLOW_MS);
  });

  it('keeps the chosen word through a language switch in the middle of the entry, and says it in the other language', async () => {
    const { backend, user } = await twoLines();
    await reachGrade(user);
    await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[0]?.sales[0]?.grade).toBe('third');
    }, SLOW_MS);
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(visible(cellOf(salesGrid(next), 0, 'grade').textContent)).toContain(
        message(next, 'glossary-darja-soyam'),
      );
    }, SLOW_MS);
    expect(textOf(salesGrid(next), 0, 'buyerTotal')).toBe('12,880');
  });
});
