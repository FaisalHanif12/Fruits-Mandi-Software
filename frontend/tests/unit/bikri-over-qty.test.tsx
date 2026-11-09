// A line that sells more than the lot has left (PRD FR-12.6, plan 57): Umar's own case. The lot
// has 50, a line is typed as 70. The backend refuses the line (`calc` null and ERR_QTY_EXCEEDS_LOT
// with `available`), and the screen must say so at once, in plain view under the sale lines, show
// no figure for that line (blank, never 0), and leave the line out of the totals, saying so. Fixing
// the quantity brings the figures back without a reload. Urdu and English. The numbers are the
// backend's, so the answers here are fitted from its own files; nothing is worked out in the tests.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { NumbersStrip } from '../../src/features/bikri/components/NumbersStrip';
import { SummaryPanel } from '../../src/features/bikri/components/SummaryPanel';
import { buildGridViews } from '../../src/features/bikri/grid-views';
import { problemsOf } from '../../src/features/bikri/problems';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { EVENT1, previewWithOverLine } from '../helpers/bikri-data';
import { OVER_TWO_LINES_PREVIEW } from '../helpers/bikri-over-qty-real';
import { seedBikriDraft } from '../helpers/bikri-seed';
import { cellOf, focusedIn, openBook, salesGrid, textOf } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, renderIn, visible } from '../helpers/render';
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

const FIGURE_CELLS = ['amount', 'sellerComm', 'buyerComm', 'laga', 'buyerTotal'] as const;

describe.each(LANGUAGES)('a line over the lot (%s)', (language) => {
  const say = (id: string, vars: Record<string, string | number> = {}) =>
    message(language, id, vars);
  const words = say('err-qty-exceeds-lot', { available: '50' });

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  /** The notes under the lines grid that carry these words. */
  const notesWith = (text: string) =>
    screen.queryAllByRole('alert').filter((note) => visible(note.textContent).includes(text));

  /**
   * Umar's own steps: the lot has 50, the line holds 50, and he types 70 in its quantity. The
   * answer for 70 is the refusal. Ends when the note is on screen. The cell is left with Enter or
   * with Tab: both show the note at once, with no other key (plan 57).
   */
  async function openUmarsCase(leave: '{Enter}' | '{Tab}' = '{Enter}') {
    seedBikriDraft({ lot: 'rahim', chuktaBuyer: true, line: { qty: '50', rate: '1200' } });
    const view = await openBook(language, previewWithOverLine);
    await waitFor(() => {
      expect(textOf(salesGrid(language), 0, 'qty')).toBe('50');
    }, SLOW_MS);
    await view.user.click(cellOf(salesGrid(language), 0, 'qty'));
    await view.user.keyboard(`70${leave}`);
    await waitFor(() => {
      expect(notesWith(words)).toHaveLength(1);
    }, SLOW_MS);
    return view;
  }

  it('says why under the lines at once, tells it once, and marks the quantity cell', async () => {
    await openUmarsCase();
    const [note] = notesWith(words);
    // The note says which line it is about, and sits under the sale lines of the lot.
    expect(visible(note?.textContent)).toBe(`${say('bikri-qty-note-line', { line: 1 })} ${words}`);
    expect(note?.closest('section')?.getAttribute('aria-labelledby')).toBe('bikri-sales-heading');
    // Not told a second time in the list of problems above the grids.
    expect(screen.getAllByText((text) => visible(text).includes(words))).toHaveLength(1);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'qty')).toHaveClass('cell-error');
    }, SLOW_MS);
  });

  it('shows the note as soon as the quantity cell is left with Tab, with no other key', async () => {
    await openUmarsCase('{Tab}');
    expect(notesWith(words)).toHaveLength(1);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'qty')).toHaveClass('cell-error');
    }, SLOW_MS);
    expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('');
  });

  it('shows no figure for the refused line, never a 0, and the totals wait for the quantity', async () => {
    await openUmarsCase();
    const sales = salesGrid(language);
    // Every figure column this window draws is blank (the compact window leaves some behind "more").
    const drawn = FIGURE_CELLS.filter(
      (field) => sales.querySelector(`.ag-row[row-index="0"] .ag-cell[col-id="${field}"]`) !== null,
    );
    expect(drawn.length).toBeGreaterThan(0);
    for (const field of drawn) expect(textOf(sales, 0, field)).toBe('');
    // The summary and the strip say the totals wait; no figure of the refused line is on the screen.
    expect(
      screen.getAllByText((text) => visible(text) === say('bikri-summary-waits-qty')).length,
    ).toBeGreaterThan(0);
    const text = visible(document.body.textContent);
    expect(text).not.toContain('64,000');
    expect(text).not.toContain('60,000');
    expect(text).not.toContain('84,000');
  });

  it('brings the figures back, with no reload, when the quantity is fixed', async () => {
    const { user } = await openUmarsCase();
    // The cursor was sent back to the quantity when the refusal landed, typing, with 70 selected (D8
    // Bug 1, B2): the next key replaces it, with no click.
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:qty');
      expect(document.activeElement).toBeInstanceOf(HTMLInputElement);
    }, SLOW_MS);
    await user.keyboard('50{Enter}');
    await waitFor(() => {
      expect(notesWith(words)).toHaveLength(0);
      expect(textOf(salesGrid(language), 0, 'buyerTotal')).toBe('64,000');
    }, SLOW_MS);
    expect(cellOf(salesGrid(language), 0, 'qty')).not.toHaveClass('cell-error');
    expect(
      screen.queryAllByText((text) => visible(text) === say('bikri-summary-waits-qty')),
    ).toHaveLength(0);
    // The backend's own figures for 50 at 1,200 (60,000, 3,600, 3,600, 400, 64,000), in every
    // figure column this window draws.
    const sales = salesGrid(language);
    const real = { amount: '60,000', sellerComm: '3,600', buyerComm: '3,600', laga: '400' };
    for (const [field, figure] of Object.entries(real)) {
      if (sales.querySelector(`.ag-row[row-index="0"] .ag-cell[col-id="${field}"]`) === null) {
        continue;
      }
      expect(textOf(sales, 0, field)).toBe(figure);
    }
  });

  it('in the whole-sale view each note names its lot and line', async () => {
    const { user } = await openUmarsCase();
    const whole = await screen.findAllByRole('button', {
      name: (name) => visible(name) === say('bikri-summary-whole'),
    });
    const first = whole[0];
    if (first === undefined) throw new Error('no switch');
    await user.click(first);
    await waitFor(() => {
      const [note] = notesWith(words);
      expect(visible(note?.textContent)).toBe(
        `${say('bikri-where-line', { lot: 1, line: 1 })} ${words}`,
      );
    }, SLOW_MS);
  });

  it('is written in the other language after a language switch, with the line and the lot kept', async () => {
    const { user } = await openUmarsCase();
    const next: Language = language === 'ur' ? 'en' : 'ur';
    await user.keyboard('{Alt>}l{/Alt}');
    const nextWords = message(next, 'err-qty-exceeds-lot', { available: '50' });
    await waitFor(() => {
      const found = screen
        .queryAllByRole('alert')
        .filter((note) => visible(note.textContent).includes(nextWords));
      expect(found).toHaveLength(1);
      expect(textOf(salesGrid(next), 0, 'qty')).toBe('70');
    }, SLOW_MS);
  });
});

describe.each(LANGUAGES)('two lines of 30 on a lot of 50, the backend answer (%s)', (language) => {
  const say = (id: string, vars: Record<string, string | number> = {}) =>
    message(language, id, vars);
  const lot = OVER_TWO_LINES_PREVIEW.lots[0];

  beforeEach(() => {
    resetClientState(language);
  });

  it('line 1 stands with its figures and only line 2 is refused, with 20 left', () => {
    const problems = problemsOf(OVER_TWO_LINES_PREVIEW.errors, [], true);
    const views = buildGridViews({
      preview: OVER_TWO_LINES_PREVIEW,
      problems,
      lotKeys: ['lot-1'],
      selectedLot: 'lot-1',
      lineKeysByLot: [['line-1', 'line-2']],
      showAll: false,
      lotLabel: () => 'lot',
      grouping: 'international',
      say: (id) => id,
      words: (problem) => problem.item.code,
    });
    expect(views.sales.qtyNotes.map((note) => note.rowId)).toEqual(['line-2']);
    expect(views.sales.errors['line-2']?.['qty']).toBe('ERR_QTY_EXCEEDS_LOT');
    expect(views.sales.errors['line-1']).toBeUndefined();
    expect(views.sales.calc['line-1']?.['buyerTotal']).toBe('19,320');
    expect(views.sales.calc['line-2']?.['buyerTotal']).toBeUndefined();
  });

  it('the panel shows only line 1 in its totals and says one line is left out', () => {
    renderIn(
      language,
      <SummaryPanel
        totals={lot?.totals ?? null}
        whole={false}
        lotNumber={1}
        netIsNegative={false}
        leftOut={1}
      />,
    );
    expect(visible(document.body.textContent)).toContain('18,000');
    expect(
      screen.queryAllByText((text) => visible(text) === say('bikri-left-out', { count: 1 })),
    ).toHaveLength(1);
  });
});

describe('the notes of the quantity, from the preview and the problems', () => {
  it('are in line order, one for each refused line, and only for errors of the quantity', () => {
    // A 50 lot, two lines of 30: line 2 is the one refused (plan 57, Section 2).
    const problems = problemsOf(
      [
        { code: 'ERR_QTY_EXCEEDS_LOT', field: 'lots[0].sales[1].qty', values: { available: 20 } },
        { code: 'ERR_REQUIRED', field: 'lots[0].sales[1].seller_rate', values: {} },
      ],
      [],
      true,
    );
    const views = buildGridViews({
      preview: undefined,
      problems,
      lotKeys: ['lot-a'],
      selectedLot: 'lot-a',
      lineKeysByLot: [['line-1', 'line-2']],
      showAll: false,
      lotLabel: () => 'lot',
      grouping: 'international',
      say: (id) => id,
      words: (problem) => problem.item.code,
    });
    expect(views.sales.qtyNotes).toEqual([
      { rowId: 'line-2', lot: 0, line: 1, words: 'ERR_QTY_EXCEEDS_LOT' },
    ]);
    expect(views.sales.errors['line-2']?.['qty']).toBe('ERR_QTY_EXCEEDS_LOT');
    expect(views.sales.qtyNotes.some((note) => note.rowId === 'line-1')).toBe(false);
  });
});

describe.each(LANGUAGES)('the summary and the strip when a line is left out (%s)', (language) => {
  const say = (id: string, vars: Record<string, string | number> = {}) =>
    message(language, id, vars);
  const lot = EVENT1.lots[0];
  const totals = lot?.totals ?? null;
  const cash = EVENT1.cash;

  const summary = (leftOut: number, counted: boolean) =>
    renderIn(
      language,
      <SummaryPanel
        totals={counted ? totals : null}
        whole={false}
        lotNumber={1}
        netIsNegative={false}
        leftOut={leftOut}
      />,
    );
  const present = (id: string, vars: Record<string, string | number> = {}) =>
    screen.queryAllByText((text) => visible(text) === say(id, vars)).length;

  beforeEach(() => {
    resetClientState(language);
  });

  it('the panel says the totals wait when no line is worked out and one is refused', () => {
    summary(1, false);
    expect(present('bikri-summary-waits-qty')).toBe(1);
    expect(present('bikri-summary-incomplete')).toBe(0);
  });

  it('the panel keeps the old words when nothing is refused, only incomplete', () => {
    summary(0, false);
    expect(present('bikri-summary-incomplete')).toBe(1);
    expect(present('bikri-summary-waits-qty')).toBe(0);
  });

  it('the panel shows the figures of the lines that are worked out, and counts the lines left out', () => {
    summary(1, true);
    expect(present('bikri-left-out', { count: 1 })).toBe(1);
    expect(visible(document.body.textContent)).toContain('29,400');
  });

  it('the panel says how many lines are left out, in the plural too', () => {
    summary(2, true);
    expect(present('bikri-left-out', { count: 2 })).toBe(1);
  });

  it('the strip says it waits for the quantity, or counts the lines left out beside its figures', () => {
    const strip = (leftOut: number, counted: boolean) =>
      renderIn(
        language,
        <NumbersStrip
          lotNumber={1}
          lotCount={1}
          showWhole={false}
          lot={{ totals: counted ? totals : null, netIsNegative: false, leftOut }}
          whole={{ totals: null, netIsNegative: false, leftOut }}
          cash={cash}
        />,
      );
    const { unmount } = strip(1, false);
    expect(present('bikri-strip-waits-qty')).toBe(1);
    unmount();
    strip(1, true);
    expect(present('bikri-strip-left-out', { count: 1 })).toBe(1);
    expect(visible(document.body.textContent)).toContain('29,400');
  });
});
