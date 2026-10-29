// The "whole بکری" view of the sale lines (PRD FR-12.1): the lines of ALL lots in one grid with a
// lot column, editable like the grid of one lot, from the same draft. The switch is on the lines
// area and the summary has the same one; pressing either changes both. Urdu and English.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as urduApi from '../../src/shared/urdu/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { previewFor } from '../helpers/bikri-data';
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
  typeTwoLines,
  type User,
} from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { IDS } from '../helpers/bikri-data';
import { EMPTY_PAIRS } from '../helpers/paged';

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

const lotLabel = (language: Language, lot: number) =>
  message(language, 'bikri-all-lot', { lot, seller: names(language).rahim });

/** The section of the lines area (its heading is the one of the lot, or of all the lines). */
function linesArea(language: Language, whole: boolean, lot = 2): HTMLElement {
  const heading = screen.getByRole('heading', {
    name: named(
      whole
        ? message(language, 'bikri-sales-title-all')
        : message(language, 'bikri-sales-title', { lot }),
    ),
  });
  const section = heading.closest('section');
  if (section === null) throw new Error('no lines area');
  return section;
}

function summaryArea(language: Language): HTMLElement {
  const section = screen
    .getByRole('heading', { name: named(message(language, 'bikri-summary-title')) })
    .closest('section');
  if (section === null) throw new Error('no summary');
  return section;
}

const switchTo = (area: HTMLElement, language: Language, whole: boolean, lot = 2) =>
  within(area).getByRole('button', {
    name: named(
      whole
        ? message(language, 'bikri-summary-whole')
        : message(language, 'bikri-summary-lot', { lot }),
    ),
  });

/** Event 1 in lot 1, then a second lot (its seller starts as the first lot's). Lot 2 is selected. */
async function twoLots(user: User, language: Language) {
  await typeHeaderAndLot(user, language);
  await typeTwoLines(user, language);
  await waitFor(() => {
    expect(textOf(salesGrid(language), 1, 'buyerTotal')).toBe('18,684');
  });
  await user.click(
    screen.getByRole('button', { name: named(message(language, 'bikri-lots-add')) }),
  );
  await waitFor(() => {
    expect(textOf(lotsGrid(language), 1, 'seller')).toBe(names(language).rahim);
  });
  await screen.findByRole('heading', {
    name: named(message(language, 'bikri-sales-title', { lot: 2 })),
  });
}

describe.each(LANGUAGES)('the whole-sale view of the lines (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the switch on the lines area and the one on the summary are the same: either changes both', async () => {
    const { user } = await openBook(language);
    await twoLots(user, language);
    const lines = linesArea(language, false);
    const summary = summaryArea(language);
    expect(switchTo(lines, language, true)).toHaveAttribute('aria-pressed', 'false');
    await user.click(switchTo(lines, language, true));
    expect(switchTo(summary, language, true)).toHaveAttribute('aria-pressed', 'true');
    expect(switchTo(linesArea(language, true), language, true)).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await user.click(switchTo(summary, language, false));
    await screen.findByRole('heading', {
      name: named(message(language, 'bikri-sales-title', { lot: 2 })),
    });
    expect(switchTo(linesArea(language, false), language, true)).toHaveAttribute(
      'aria-pressed',
      'false',
    );
  });

  it('the docked strip names the lot on screen, and the whole sale when the switch says so (plan 38)', async () => {
    const { user } = await openBook(language);
    await twoLots(user, language);
    const strip = () =>
      screen.getByRole('region', { name: named(message(language, 'bikri-strip-label')) });
    // Two lots: the one line names the lot it shows, so the numbers are not taken for the sale's.
    expect(visible(strip().textContent)).toContain(
      message(language, 'bikri-summary-lot', { lot: 2 }),
    );
    await user.click(switchTo(linesArea(language, false), language, true));
    await waitFor(() => {
      expect(visible(strip().textContent)).toContain(message(language, 'bikri-summary-whole'));
    });
    expect(visible(strip().textContent)).not.toContain(
      message(language, 'bikri-summary-lot', { lot: 2 }),
    );
  });

  it('lists the lines of every lot in one grid, each with its lot', async () => {
    const { user } = await openBook(language);
    await twoLots(user, language);
    await user.click(switchTo(linesArea(language, false), language, true));
    const sales = salesGrid(language);
    // Lot 1: Akram, Arshad and the blank line the Enter made; lot 2: its own blank line.
    await waitFor(() => {
      expect(sales.querySelectorAll('.ag-row')).toHaveLength(4);
      expect(textOf(sales, 0, 'buyer')).toBe(names(language).akram);
      expect(textOf(sales, 1, 'buyer')).toBe(names(language).arshad);
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
      expect(textOf(sales, 1, 'buyerTotal')).toBe('18,684');
      for (const row of [0, 1, 2]) expect(textOf(sales, row, 'lot')).toBe(lotLabel(language, 1));
      expect(textOf(sales, 3, 'lot')).toBe(lotLabel(language, 2));
    });
  });

  it('a line typed in the whole view goes to the lot that owns it, and the view of that lot shows it', async () => {
    const { backend, user } = await openBook(language);
    await twoLots(user, language);
    await user.click(switchTo(linesArea(language, false), language, true));
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(sales.querySelectorAll('.ag-row')).toHaveLength(4);
    });
    // The buyer of the line of lot 2 (the fourth line), by short code 2, then Enter.
    await user.click(cellOf(sales, 3, 'buyer'));
    await user.keyboard('1{Enter}');
    await waitFor(() => {
      expect(backend.lastDraft().lots[1]?.sales[0]?.buyer_account_id).toBe(IDS.arshad);
    });
    // Lot 1 is as it was.
    expect(backend.lastDraft().lots[0]?.sales[0]?.buyer_account_id).toBe(IDS.akram);
    expect(backend.lastDraft().lots[0]?.sales[1]?.buyer_account_id).toBe(IDS.arshad);
    // The cursor moved on to the quantity of that line, and the details are those of lot 2.
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('3:qty');
    });
    await screen.findByRole('textbox', {
      name: named(message(language, 'bikri-line-seller-comm')),
    });
    // Back to one lot: lot 2 is selected, with the buyer in its own first line, cursor kept.
    await user.click(switchTo(linesArea(language, true), language, false));
    await screen.findByRole('heading', {
      name: named(message(language, 'bikri-sales-title', { lot: 2 })),
    });
    const one = salesGrid(language);
    await waitFor(() => {
      expect(textOf(one, 0, 'buyer')).toBe(names(language).arshad);
      expect(focusedIn(one)).toBe('0:qty');
    });
  });

  it('Enter after the last stop of a lot’s last line goes on to the first line of the next lot', async () => {
    const { user } = await openBook(language);
    await twoLots(user, language);
    await user.click(switchTo(linesArea(language, false), language, true));
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(sales.querySelectorAll('.ag-row')).toHaveLength(4);
    });
    await user.click(cellOf(sales, 2, 'rate'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(focusedIn(sales)).toBe('3:buyer');
    });
    expect(sales.querySelectorAll('.ag-row')).toHaveLength(4);
  });

  it('a language switch in the whole view keeps the lines, the words of the lot column and the cursor', async () => {
    const { user } = await openBook(language);
    await twoLots(user, language);
    await user.click(switchTo(linesArea(language, false), language, true));
    await waitFor(() => {
      expect(salesGrid(language).querySelectorAll('.ag-row')).toHaveLength(4);
    });
    await user.click(cellOf(salesGrid(language), 1, 'qty'));
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(textOf(salesGrid(next), 3, 'lot')).toBe(lotLabel(next, 2));
    });
    const sales = salesGrid(next);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyer')).toBe(names(next).akram);
      expect(textOf(sales, 0, 'buyerTotal')).toBe('12,880');
      expect(focusedIn(sales)).toBe('1:qty');
    });
  });

  it('empty: on an untouched screen the whole view is one blank line of lot 1, ready to type in', async () => {
    const { backend, user } = await openBook(language);
    // Lot 1 is selected: the switch on the lines area is "Lot 1 | Whole sale".
    await user.click(switchTo(linesArea(language, false, 1), language, true, 1));
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(sales.querySelectorAll('.ag-row')).toHaveLength(1);
    });
    expect(textOf(sales, 0, 'lot')).toBe(message(language, 'bikri-all-lot-no-seller', { lot: 1 }));
    expect(backend.previews).toHaveLength(0);
  });

  it('error: when the numbers cannot be worked out the whole grid still holds what was typed, and says so', async () => {
    let failing = false;
    const { user } = await openBook(language, (draft) =>
      failing ? Promise.reject(fail({ code: 'ERR_NOT_IMPLEMENTED' })) : previewFor(draft),
    );
    await twoLots(user, language);
    await user.click(switchTo(linesArea(language, false), language, true));
    await waitFor(() => {
      expect(salesGrid(language).querySelectorAll('.ag-row')).toHaveLength(4);
    });
    failing = true;
    await user.click(cellOf(salesGrid(language), 3, 'buyer'));
    await user.keyboard('1{Enter}');
    expect(
      await screen.findByText(
        (text) => visible(text) === message(language, 'bikri-preview-failed'),
      ),
    ).toBeInTheDocument();
    // What was typed is still there, in the lot that owns it.
    expect(textOf(salesGrid(language), 3, 'buyer')).toBe(names(language).arshad);
    expect(textOf(salesGrid(language), 0, 'buyer')).toBe(names(language).akram);
  });

  it('a problem of any lot shows on its own cell of the whole view', async () => {
    const { user } = await openBook(language, (draft) => ({
      ...previewFor(draft),
      errors: [
        { code: 'ERR_QTY_EXCEEDS_LOT', field: 'lots[1].sales[0].qty', values: { available: 0 } },
      ],
    }));
    await twoLots(user, language);
    await user.click(switchTo(linesArea(language, false), language, true));
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(sales.querySelectorAll('.ag-row')).toHaveLength(4);
    });
    await waitFor(() => {
      expect(cellOf(sales, 3, 'qty')).toHaveClass('cell-error');
      expect(cellOf(sales, 0, 'qty')).not.toHaveClass('cell-error');
      expect(visible(document.body.textContent)).toContain(
        message(language, 'bikri-where-line', { lot: 2, line: 1 }),
      );
    });
  });
});
