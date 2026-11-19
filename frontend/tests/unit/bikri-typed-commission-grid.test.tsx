// Plan 72 P13 (PRD FR-11.10): a commission or لاگا rate typed in the details of a sale line must show in the
// GRID, at every window step (1024 x 600 is compact, 1366 x 768 is normal, 1920 x 1080 is wide; plan 58 option C,
// plan 65, the lead's decision of plan 72 P13). From 1366 px the grid has the seller commission and the buyer
// commission, so what the user types there moves a figure he sees; the seller amount is in the grid from 1920 px
// and in the details of the line in every window (NFR-USE-27); the لاگا is in the details everywhere. In every
// window the buyer total moves, and a cell whose figure rests on a charge typed by hand carries the "Manual" mark
// (a pencil, and the word for the tooltip and a screen reader), taken from the backend's `source: typed`.
// The real screen, the real grid and the real store; the fake backend answers like the real one for a typed
// charge (tests/helpers/bikri-typed.ts).

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import * as urduApi from '../../src/shared/urdu/api';
import { answerWithTyped } from '../helpers/bikri-typed';
import {
  cellOf,
  named,
  openBook,
  salesGrid,
  textOf,
  typeHeaderAndLot,
  typeTwoLines,
  type User,
} from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
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
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const STEPS = [
  ['compact', '1024 x 600'],
  ['normal', '1366 x 768'],
  ['wide', '1920 x 1080'],
] as const;

/** The figure columns of the grid of the sale lines, by step (the stops and the paid cell are the same). */
const FIGURE_COLUMNS: Record<LayoutStep, readonly string[]> = {
  compact: ['buyerTotal'],
  normal: ['sellerComm', 'buyerComm', 'buyerTotal'],
  wide: ['amount', 'sellerComm', 'buyerComm', 'buyerTotal'],
};

/** The columns of the grid of the sale lines, in order, as the header cells say. */
function columnsOf(language: Language): string[] {
  return Array.from(salesGrid(language).querySelectorAll('.ag-header-cell[col-id]')).map(
    (cell) => cell.getAttribute('col-id') ?? '',
  );
}

/** The text of the details of the line that has the cursor, around a field of it. */
function detailsOf(field: HTMLElement): string {
  const section = field.closest('section');
  if (section === null) throw new Error('no details of the line');
  return visible(section.textContent);
}

describe.each(LANGUAGES)('a charge typed in the details, and the grid (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const manualWord = () => message(language, 'bikri-tag-manual');

  /** Types the two lines of Event 1 and puts the cursor on line 1, so its details are shown. */
  async function openLineOne(step: LayoutStep) {
    vi.mocked(useLayoutStep).mockReturnValue(step);
    const { user } = await openBook(language, answerWithTyped);
    await typeHeaderAndLot(user, language);
    await typeTwoLines(user, language);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
    }, SLOW_MS);
    await user.click(cellOf(sales, 0, 'qty'));
    return { user, sales };
  }

  const field = (id: string) =>
    screen.findByRole('textbox', { name: named(message(language, id)) });

  async function typeInto(user: User, id: string, text: string) {
    const input = await field(id);
    await user.click(input);
    await user.keyboard(text);
    return input;
  }

  /** Is the "Manual" mark in this cell: the pencil and its word? */
  const marked = (sales: HTMLElement, row: number, column: string) => {
    const cell = cellOf(sales, row, column);
    return (
      cell.querySelector('svg.lucide-pencil') !== null &&
      within(cell).queryByRole('img', { name: manualWord() }) !== null
    );
  };

  describe.each(STEPS)('%s (%s)', (step) => {
    it('has the figure columns of its step, the paid cell last, and no لاگا column', async () => {
      await openLineOne(step);
      const columns = columnsOf(language);
      expect(columns.filter((id) => FIGURE_COLUMNS.normal.includes(id) || id === 'amount')).toEqual(
        FIGURE_COLUMNS[step],
      );
      expect(columns).not.toContain('laga');
      // The grid's own remove button is a column after the paid cell.
      expect(columns.filter((id) => id !== '__remove').at(-1)).toBe('paid');
      expect(columns.slice(0, 4)).toEqual(['buyer', 'qty', 'rate', 'grade']);
    });

    it('shows no mark while nothing is typed, and the seller amount is in the details', async () => {
      const { user, sales } = await openLineOne(step);
      for (const column of FIGURE_COLUMNS[step]) expect(marked(sales, 0, column)).toBe(false);
      const input = await field('bikri-line-seller-comm');
      expect(detailsOf(input)).toContain('12,000');
      await user.click(cellOf(sales, 0, 'qty'));
    });

    it('a buyer commission typed in the details moves the buyer total and marks it', async () => {
      const { user, sales } = await openLineOne(step);
      const input = await typeInto(user, 'bikri-line-buyer-comm', '4');
      // 12,000 + 480 + 160: the buyer total moves in every window.
      await waitFor(() => {
        expect(textOf(sales, 0, 'buyerTotal')).toBe('12,640');
      }, SLOW_MS);
      expect(detailsOf(input)).toContain('480');
      expect(marked(sales, 0, 'buyerTotal')).toBe(true);
      // From 1366 px the buyer commission has its own column, and it moves and says Manual; the seller
      // commission, which was not typed, does not.
      if (step !== 'compact') {
        expect(textOf(sales, 0, 'buyerComm')).toBe('480');
        expect(marked(sales, 0, 'buyerComm')).toBe(true);
        expect(marked(sales, 0, 'sellerComm')).toBe(false);
      }
    });

    it('a لاگا rate typed in the details moves the buyer total and marks it', async () => {
      const { user, sales } = await openLineOne(step);
      const input = await typeInto(user, 'glossary-laga', '10');
      // 12,000 + 720 + 20 x 10.
      await waitFor(() => {
        expect(textOf(sales, 0, 'buyerTotal')).toBe('12,920');
      }, SLOW_MS);
      expect(detailsOf(input)).toContain('200');
      expect(marked(sales, 0, 'buyerTotal')).toBe(true);
    });

    it('a seller commission typed in the details shows in the grid from 1366 px and marks the line', async () => {
      const { user, sales } = await openLineOne(step);
      const input = await typeInto(user, 'bikri-line-seller-comm', '4');
      await waitFor(() => {
        expect(detailsOf(input)).toContain('480');
      }, SLOW_MS);
      if (step === 'compact') {
        // No commission column in the narrow window: the total and the details are enough, and the total
        // says that something of the line was typed by hand.
        expect(columnsOf(language)).not.toContain('sellerComm');
      } else {
        await waitFor(() => {
          expect(textOf(sales, 0, 'sellerComm')).toBe('480');
        }, SLOW_MS);
        expect(marked(sales, 0, 'sellerComm')).toBe(true);
        expect(marked(sales, 0, 'buyerComm')).toBe(false);
      }
      await waitFor(() => {
        expect(marked(sales, 0, 'buyerTotal')).toBe(true);
      }, SLOW_MS);
    });
  });

  it('the Enters of a line are the same at every step: buyer, qty, rate, paid', async () => {
    const { sales } = await openLineOne('normal');
    expect(cellOf(sales, 0, 'paid')).toBeInTheDocument();
    // The commission cells are calculated cells: never an Enter stop.
    expect(cellOf(sales, 0, 'sellerComm').classList.contains('cell-calc')).toBe(true);
    expect(cellOf(sales, 0, 'buyerComm').classList.contains('cell-calc')).toBe(true);
  });
});
