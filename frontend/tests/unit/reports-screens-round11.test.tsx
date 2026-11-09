// The party day sheet (R35, PRD FR-19.12, FR-19.2, FR-19.10, FR-19.13, FR-13.5, E2E-43) on the ONE
// report screen (round 11), in the Urdu and in the English app, against a fake backend that answers
// with the backend's own fixtures. The table of every party (the buyers, then the sellers, a group
// row for each with its own figures, a blank column for the collector's note that waits for the wide
// step) and the lines under it (the shop's total اگراہی and the sellers' totals); the slip of one
// party, opened with Enter on his row (a one-time person with his key), with its labelled lines in the
// order the backend sends, as a card beside the goods (plan 42, PRD v1.12); the way back to all parties;
// the print with the same order; the four states; a language switch in the middle of a slip and the
// keyboard-only path. The screen adds up nothing: every figure is one the backend sent. The layout
// step in a test with no layout is the middle one, so the collector's column of the wide step is
// not drawn here; the screenshots show it.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportRow } from '../../src/bindings/ReportRow';
import * as gardensApi from '../../src/features/gardens/api';
import * as reportsApi from '../../src/features/reports/api';
import * as searchApi from '../../src/shared/search/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeAccountSearch } from '../helpers/accounts';
import { fakeGardens } from '../helpers/gardens';
import { fakePartners } from '../helpers/partners';
import { paisaForTest } from '../helpers/paisa';
import { printFirmName } from '../helpers/print-frame';
import { message, visible, paragraph } from '../helpers/render';
import {
  CATALOG,
  PRINT_R35_SLIP_AKRAM_EN,
  PRINT_R36_PARTNER_EN,
  R34_GARDEN_DAY,
  R35_TABLE,
  R36_GARDEN,
  fakeReports,
} from '../helpers/reports';
import { PAGE_ROWS } from '../helpers/pager';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one.
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners([]);
  fakeGardens();
  // The party box of a report asks the account search page by page. After `resetAllMocks` it would answer
  // nothing at all, and a picker that gets no page throws; a real empty page is the honest answer here.
  fakeAccountSearch([]);
  // The collector box of R35 (plan 55 Section 11.1) asks the staff list the same way: an empty page, not none.
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

/** Where the garden's state and its balance are among the cells of an R36 row (by garden). */
const STATUS_COLUMN = 2;
const BALANCE_COLUMN = 13;

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function totalsCell(colId: string): string {
  const row = document.querySelector('.ag-row-pinned');
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

function drawnColumns(): string[] {
  return Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map(
    (header) => header.getAttribute('col-id') ?? '',
  );
}

/** The element of a selector, or a plain failure that names it. */
function pick(selector: string): Element {
  const found = document.querySelector(selector);
  if (found === null) throw new Error(`nothing matches ${selector}`);
  return found;
}

function lastRun() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
}

function runCount() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.length;
}

/** What a box shows: its text, read from the input. */
function boxText(name: string): string {
  const box = screen.getByRole('combobox', { name });
  return box instanceof HTMLInputElement ? box.value : visible(box.textContent);
}

/**
 * The labelled lines of a party's day slip, as "label value" text in the order they are drawn: the card
 * beside the goods (plan 42). A balance has its side word after the figure.
 */
function slipLines(): string[] {
  const dl = document.querySelector('section dl.flex-col');
  return Array.from(dl?.children ?? []).map((line) => {
    const value = line.querySelector('dd');
    const full = visible(value?.textContent);
    const word = visible(value?.querySelector('.slip-side')?.textContent);
    const figure = word === '' ? full : `${full.slice(0, -word.length)} ${word}`;
    return `${visible(line.querySelector('dt')?.textContent)} ${figure}`.trim();
  });
}

/** The lines of the footer, as "label value" text in the order they are drawn. */
function footerLines(): string[] {
  const list = document.querySelector('dl.grid-cols-4');
  return Array.from(list?.querySelectorAll('div') ?? []).map((line) =>
    visible(line.textContent).replace(/\s+/g, ' ').trim(),
  );
}

/** A page of the table with nothing in it, as the backend answers for a day with no entry. */
const EMPTY_TABLE: ReportPage = {
  ...R35_TABLE,
  rows: [],
  total_count: 0,
  summary: [R35_TABLE.summary[0]].flatMap((line) => (line === undefined ? [] : [line])),
};

describe.each(LANGUAGES)('the party day sheet R35 (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openSheet(options: Parameters<typeof fakeReports>[0] = {}) {
    const fake = fakeReports(options);
    const view = await renderApp('/reports/R35');
    await screen.findByRole('heading', { level: 1, name: named(m('report-r35-title')) });
    return { ...view, fake, user: userEvent.setup() };
  }

  /** Types the day and opens the slip of the party in a row, with the keys. */
  async function openSlip(user: ReturnType<typeof userEvent.setup>, rowId: string) {
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(7);
    }, SLOW_MS);
    await user.click(pick(`.ag-row[row-id="${rowId}"] [col-id="party"]`));
    await user.keyboard('{Enter}');
  }

  describe('the table of every party', () => {
    it('is in the catalogue with a day box, a party box and a collector box, and no period', async () => {
      expect(CATALOG.find((entry) => entry.id === 'R35')).toMatchObject({
        filters: ['as_of', 'party', 'collector'],
        party_required: false,
      });
      await openSheet();
      expect(screen.getByLabelText(m('report-filter-day'))).toBeVisible();
      expect(screen.getByRole('combobox', { name: m('report-filter-party') })).toBeVisible();
      // One collector can be picked (plan 55 Section 11.1); with none the sheet has every collector.
      expect(
        screen.getByRole('combobox', { name: named(m('report-filter-collector')) }),
      ).toBeVisible();
      expect(screen.queryByRole('combobox', { name: m('report-filter-period') })).toBeNull();
      expect(screen.queryByLabelText(m('report-filter-as-of'))).toBeNull();
    });

    it('asks for no party and shows the buyers then the sellers, a group row for each with its figures', async () => {
      await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2', '3', '4', '5', '6']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({
        id: 'R35',
        filters: { party: null, as_of: '' },
        offset: 0,
        limit: PAGE_ROWS,
      });
      // The group rows say who the parties under them are, with the figures of the whole group.
      expect(cell('0', 'code')).toBe(m('report-r35-buyers'));
      expect(cell('0', 'debit')).toBe('44,364');
      expect(cell('0', 'credit')).toBe('22,800');
      expect(cell('4', 'code')).toBe(m('report-r35-sellers'));
      expect(cell('4', 'debit')).toBe('9,316');
      expect(cell('4', 'credit')).toBe('34,476');
      expect(document.querySelector('.ag-row[row-id="0"]')?.className).toContain('row-group');
      // A buyer: his short code, his name with his code, and what was debited, credited and is left.
      expect(cell('2', 'code')).toBe('0');
      expect(cell('2', 'party')).toBe(`0020 ${names('اکرم ٹول ٹیکس', 'Akram Toll Tax')}`);
      expect(cell('2', 'debit')).toBe('12,880');
      expect(cell('2', 'credit')).toBe('10,000');
      expect(cell('2', 'remaining')).toContain('2,880');
      expect(cell('2', 'remaining')).toContain(m('report-side-dr'));
      // A one-time buyer who paid on the spot has nothing left, and says so with the word for zero.
      expect(cell('3', 'party')).toContain(names('شاہد', 'Shahid'));
      // His phone follows his name, in the dash form with English digits (PRD FR-13.5).
      expect(cell('3', 'party')).toContain('0300-0000000');
      expect(cell('3', 'remaining')).toContain('0');
      // A seller is owed: his balance is on the other side.
      expect(cell('6', 'remaining')).toContain('10,160');
      expect(cell('6', 'remaining')).toContain(m('report-side-cr'));
    });

    it('has no totals row of its own: the buyers’ total is in their group row and in the line under the table', async () => {
      await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      // A pinned row under both sections would be read as the total of all of them.
      expect(document.querySelector('.ag-row-pinned')).toBeNull();
      expect(cell('0', 'remaining')).toContain('21,564');
      expect(footerLines()[0]).toContain('21,564');
    });

    it('draws the columns that fit, and leaves the collector’s blank column for the wide step and the print', async () => {
      await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      expect(drawnColumns()).toEqual(['code', 'party', 'previous', 'debit', 'credit', 'remaining']);
    });

    it('writes the lines under the table: the shop’s total اگراہی, then the sellers’ credit, debit and balance', async () => {
      await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      const lines = footerLines();
      expect(lines).toHaveLength(4);
      expect(lines[0]).toContain(m('report-r35-collection'));
      expect(lines[0]).toContain('21,564');
      expect(lines[1]).toContain(m('report-r35-sellers-credit'));
      expect(lines[1]).toContain('34,476');
      expect(lines[2]).toContain(m('report-r35-sellers-debit'));
      expect(lines[2]).toContain('9,316');
      expect(lines[3]).toContain(m('report-r35-sellers-remaining'));
      expect(lines[3]).toContain('10,160');
      expect(lines[3]).toContain(m('report-side-cr'));
      // The lines are under the table, where the client's slip has its footer.
      const grid = pick('.ag-root-wrapper');
      const footer = pick('dl.grid-cols-4');
      expect(grid.compareDocumentPosition(footer) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    });

    it('sends the day as it was typed, and says which day the sheet is for', async () => {
      const { user } = await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      await user.type(screen.getByLabelText(m('report-filter-day')), '10-10-2026');
      await waitFor(() => {
        expect(lastRun()?.filters.as_of).toBe('10-10-2026');
      }, SLOW_MS);
      // The backend echoes the day it read; the sheet says "for" that day, not "balances as of".
      expect(
        await screen.findByText(paragraph(m('report-day-shown', { date: '10-10-2026' }))),
      ).toBeVisible();
      expect(
        screen.queryByText(paragraph(m('report-as-of-shown', { date: '10-10-2026' }))),
      ).toBeNull();
    });
  });

  describe('the slip of one party', () => {
    it('Enter on a buyer opens his slip: the goods of his day and, beside them, the lines of what the day comes to', async () => {
      const { user } = await openSheet();
      await openSlip(user, '2');
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['item', 'packing', 'qty', 'rate', 'amount']);
      }, SLOW_MS);
      expect(lastRun()?.filters.party).toEqual({ account_id: 20, person_key: null });
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
      }, SLOW_MS);
      expect(drawnRows()).toEqual(['0']);
      expect(cell('0', 'packing')).toBe(names('پیٹی', 'Box'));
      expect(cell('0', 'qty')).toBe('20');
      expect(cell('0', 'rate')).toBe('600');
      // The goods value: the quantity times the rate, not the amount with commission and لاگا.
      expect(cell('0', 'amount')).toBe('12,000');
      // The slip has no totals row: its lines are the card beside the goods.
      expect(totalsCell('qty')).toBe('');
      expect(totalsCell('amount')).toBe('');
      // Each line on its own row, in the order the backend sent them, with the worked figures.
      const lines = slipLines();
      expect(lines).toEqual([
        `${m('report-r35-goods')} 12,000`,
        `${m('report-r35-commission')} 720`,
        `${m('report-r35-laga')} 160`,
        `${m('report-r35-today-total')} 12,880`,
        `${m('report-r35-paid-today')} 10,000`,
        `${m('report-r35-today-left')} 2,880 ${m('report-side-dr')}`,
        `${m('report-r35-previous')} 0`,
        `${m('report-r35-new-balance')} 2,880 ${m('report-side-dr')}`,
        `${m('report-r35-total-qty')} 20`,
      ]);
    });

    it('shows the party in the party box with his name in the app language', async () => {
      const { user } = await openSheet();
      await openSlip(user, '2');
      await waitFor(() => {
        expect(boxText(m('report-filter-party'))).toContain(
          names('اکرم ٹول ٹیکس', 'Akram Toll Tax'),
        );
      }, SLOW_MS);
    });

    it('Enter on a seller opens his slip: his lots, the item total, each deduction and his net sale of the day', async () => {
      const { user } = await openSheet();
      await openSlip(user, '5');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({ account_id: 18, person_key: null });
      }, SLOW_MS);
      await waitFor(() => {
        expect(slipLines()).toHaveLength(11);
      }, SLOW_MS);
      expect(slipLines()).toEqual([
        `${m('report-r35-gross')} 29,400`,
        `${m('report-r35-commission')} 1,764`,
        `${m('report-r35-freight')} 3,000`,
        `${m('report-r35-labour')} 250`,
        `${m('report-r35-munshiana')} 50`,
        `${m('report-r35-phone')} 20`,
        `${m('report-r35-net-sale')} 24,316 ${m('report-side-cr')}`,
        `${m('report-r35-paid-today')} 9,316`,
        `${m('report-r35-today-left')} 15,000 ${m('report-side-cr')}`,
        `${m('report-r35-previous')} 15,000 ${m('report-side-dr')}`,
        `${m('report-r35-new-balance')} 0`,
      ]);
      // His two lots, each with the seller's rate, and the total row of the item set apart.
      await waitFor(() => {
        expect(cell('2', 'amount')).toBe('29,400');
      }, SLOW_MS);
      expect(cell('0', 'rate')).toBe('600');
      expect(cell('1', 'rate')).toBe('580');
      expect(cell('0', 'amount')).toBe('12,000');
      expect(cell('1', 'amount')).toBe('17,400');
      expect(cell('2', 'item')).toBe(names('کیلا کل', 'Banana total'));
      expect(cell('2', 'qty')).toBe('50');
      expect(document.querySelector('.ag-row[row-id="2"]')?.className).toContain('row-group');
    });

    it('a چکتا buyer is asked for with his own key, so his slip is his and not the whole account’s', async () => {
      const { user } = await openSheet();
      await openSlip(user, '3');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({
          account_id: 9,
          person_key: 'شاہد|03000000000',
        });
      }, SLOW_MS);
      // There is no person box: the key goes with the row, and no filter offers a person (D8).
    });

    it('Enter on a line of the slip opens the بکری behind it', async () => {
      const { router, user } = await openSheet();
      await openSlip(user, '2');
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="item"]'));
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });

    it('Enter on the item total row opens nothing: it is a total, not a line of a بکری', async () => {
      const { router, user } = await openSheet();
      await openSlip(user, '5');
      await waitFor(() => {
        expect(cell('2', 'item')).toBe(names('کیلا کل', 'Banana total'));
      }, SLOW_MS);
      const before = router.state.location.pathname;
      await user.click(pick('.ag-row[row-id="2"] [col-id="item"]'));
      await user.keyboard('{Enter}');
      expect(router.state.location.pathname).toBe(before);
    });

    it('“All parties” goes back to the table, with the day still typed', async () => {
      const { user } = await openSheet();
      await user.type(screen.getByLabelText(m('report-filter-day')), '10-10-2026');
      await openSlip(user, '2');
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['item', 'packing', 'qty', 'rate', 'amount']);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-r35-all-parties')) }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      expect(lastRun()?.filters).toMatchObject({ party: null, as_of: '10-10-2026' });
      expect(screen.getByLabelText(m('report-filter-day'))).toHaveValue('10-10-2026');
      expect(screen.queryByRole('button', { name: named(m('report-r35-all-parties')) })).toBeNull();
    });

    it('the button back is only there while a slip is open', async () => {
      await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      expect(screen.queryByRole('button', { name: named(m('report-r35-all-parties')) })).toBeNull();
    });
  });

  describe('the print', () => {
    it('is the day slip: the goods first and the labelled lines under them, with the day named "Day"', async () => {
      const { user } = await openSheet({ print: PRINT_R35_SLIP_AKRAM_EN });
      await openSlip(user, '2');
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['item', 'packing', 'qty', 'rate', 'amount']);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByText(printFirmName(language), undefined, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R35',
        language,
        paper: 'a4',
      });
      // The lines are a card under the goods, in the order the backend sent them.
      const table = pick('.print-report .print-table');
      const summary = pick('.print-report .print-slip-lines');
      expect(
        table.compareDocumentPosition(summary) & Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
      const labels = Array.from(summary.querySelectorAll('th')).map((term) =>
        visible(term.textContent),
      );
      expect(labels).toEqual(
        PRINT_R35_SLIP_AKRAM_EN.summary.map((line) => message(language, line.label_id)),
      );
      // The header names the day with the word "Day", not "As of", and the date the backend sent.
      const facts = visible(pick('.print-report .print-facts').textContent);
      expect(facts).toContain(message(language, 'report-filter-day'));
      expect(facts).not.toContain(message(language, 'report-filter-as-of'));
      expect(facts).toContain('10-10-2026');
    });
  });

  describe('the four states', () => {
    it('shows the loading state while the backend answers', async () => {
      fakeReports();
      vi.mocked(reportsApi.reportsRun).mockImplementation(() => new Promise(() => undefined));
      await renderApp('/reports/R35');
      await screen.findByRole('heading', { level: 1, name: named(m('report-r35-title')) });
      expect(await screen.findByRole('status')).toBeInTheDocument();
    });

    it('says there is nothing for the day, with no footer lines under it', async () => {
      await openSheet({ pages: { R35: EMPTY_TABLE } });
      await screen.findByText(named(m('report-empty')), undefined, SLOW_MS);
      expect(visible(document.body.textContent)).not.toContain(m('report-r35-collection'));
    });

    it('says what happened and tries again when the backend fails', async () => {
      let failing = true;
      const { user } = await openSheet({
        refuse: () => {
          if (!failing) return null;
          failing = false;
          return {
            ok: false,
            errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
            ref: 'E-1',
          } as never;
        },
      });
      await screen.findByText(named(m('report-load-failed')), undefined, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
    });

    it('a day the backend refuses shows under the day box and the rows stay', async () => {
      const { user } = await openSheet({
        refuse: (input) =>
          input.filters.as_of === '99-99-9999'
            ? ({
                ok: false,
                errors: [{ code: 'ERR_DATE_INVALID', field: 'filters.as_of', values: {} }],
                ref: 'E-2',
              } as never)
            : null,
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      await user.type(screen.getByLabelText(m('report-filter-day')), '99-99-9999');
      await waitFor(() => {
        expect(lastRun()?.filters.as_of).toBe('99-99-9999');
      }, SLOW_MS);
      // The rows already on screen are kept, and the refusal is not a page of its own.
      expect(drawnRows()).toHaveLength(7);
      expect(screen.queryByText(named(m('report-load-failed')))).toBeNull();
    });
  });

  describe('a language switch in the middle of a slip', () => {
    it('keeps the day and the party, writes the lines again in the other language, and asks nothing new', async () => {
      const { user } = await openSheet();
      await user.type(screen.getByLabelText(m('report-filter-day')), '10-10-2026');
      await openSlip(user, '2');
      await waitFor(() => {
        expect(slipLines()).toHaveLength(9);
      }, SLOW_MS);
      const before = runCount();
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(slipLines()[1]).toContain(message(other, 'report-r35-commission'));
      }, SLOW_MS);
      expect(screen.getByLabelText(message(other, 'report-filter-day'))).toHaveValue('10-10-2026');
      expect(boxText(message(other, 'report-filter-party'))).toContain(
        other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
      );
      expect(slipLines()[7]).toContain('2,880');
      expect(runCount()).toBe(before);
    });
  });

  describe('keyboard only', () => {
    it('Tab to the table, ↓ to a party, Enter for his slip, then the button back with Enter', async () => {
      const { user } = await openSheet();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
      for (let tabs = 0; tabs < 40; tabs += 1) {
        if (document.activeElement?.closest('.ag-root-wrapper') != null) break;
        await user.tab();
      }
      expect(document.activeElement?.closest('.ag-root-wrapper')).not.toBeNull();
      // The first row is the heading of the buyers; the second is Arshad, the third Akram.
      await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}{Enter}');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({ account_id: 20, person_key: null });
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['item', 'packing', 'qty', 'rate', 'amount']);
      }, SLOW_MS);
      const back = screen.getByRole('button', { name: named(m('report-r35-all-parties')) });
      await act(async () => {
        back.focus();
        await Promise.resolve();
      });
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(7);
      }, SLOW_MS);
    });
  });
});

describe.each(LANGUAGES)(
  'the gardens report R36 and the garden rows of R34 (%s)',
  (language: Language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);
    const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
    const other: Language = language === 'ur' ? 'en' : 'ur';

    async function openGardens(options: Parameters<typeof fakeReports>[0] = {}) {
      const fake = fakeReports(options);
      const view = await renderApp('/reports/R36');
      await screen.findByRole('heading', { level: 1, name: named(m('report-r36-title')) });
      return { ...view, fake, user: userEvent.setup() };
    }

    async function choose(user: ReturnType<typeof userEvent.setup>, box: string, option: string) {
      await user.click(screen.getByRole('combobox', { name: m(box) }));
      await user.click(await screen.findByRole('option', { name: option }, SLOW_MS));
    }

    describe('R36', () => {
      it('is in the catalogue with the boxes of the PRD: the days, the item, the partner, the garden and the layout', async () => {
        expect(CATALOG.find((entry) => entry.id === 'R36')?.filters).toEqual([
          'date_range',
          'item',
          'partner',
          'garden',
          'garden_view',
        ]);
        await openGardens();
        for (const box of [
          'report-filter-period',
          'report-filter-item',
          'report-filter-partner',
          'report-filter-garden',
          'report-filter-garden-view',
        ]) {
          const found = screen.queryByRole('combobox', { name: m(box) });
          // The item is a picker (a text box with a list), the others are drop-downs.
          expect(found ?? screen.queryByLabelText(m(box)), box).not.toBeNull();
        }
      });

      it('asks by garden with no garden and no layout, and draws the garden with the backend’s figures', async () => {
        await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0']);
        }, SLOW_MS);
        expect(lastRun()).toMatchObject({
          id: 'R36',
          filters: { garden_id: null, partner_id: null },
          offset: 0,
          limit: PAGE_ROWS,
        });
        expect(lastRun()?.filters.garden_view).toBeUndefined();
        expect(cell('0', 'garden')).toBe(names('شاہ پور باغ', 'Shahpur Garden'));
        expect(cell('0', 'status')).toBe(m('report-garden-closed'));
        expect(cell('0', 'rent')).toBe('50,000');
        expect(cell('0', 'installments')).toBe('50,000');
        expect(cell('0', 'costs')).toBe('5,000');
        expect(cell('0', 'money_in')).toBe('66,400');
        // A closed garden: its result is a profit, so the figure carries the word (FR-12.38).
        expect(cell('0', 'balance')).toBe(`11,400${m('report-result-profit')}`);
        expect(cell('0', 'rent_due')).toBe('0');
        // The totals row is the backend's: the sums of the rows.
        expect(totalsCell('money_in')).toBe('66,400');
        expect(totalsCell('balance')).toBe('11,400');
      });

      it('draws the columns that fit: the installments and costs come with the normal step, what was paid and the sales with the wide one', async () => {
        await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        expect(drawnColumns()).toEqual([
          'garden',
          'status',
          'rent',
          'installments',
          'costs',
          'money_in',
          'balance',
          'rent_due',
        ]);
      });

      it.each([
        ['compact', ['garden', 'status', 'rent', 'money_in', 'balance', 'rent_due']],
        [
          'wide',
          [
            'garden',
            'status',
            'rent',
            'installments',
            'costs',
            'paid_shop',
            'paid_pocket',
            'sales_our_shop',
            'money_in',
            'balance',
            'rent_due',
          ],
        ],
      ] as const)(
        'draws only the columns that fit at the %s step, so the table never scrolls sideways',
        async (step, columns) => {
          vi.mocked(useLayoutStep).mockReturnValue(step);
          await openGardens();
          await waitFor(() => {
            expect(drawnRows()).toHaveLength(1);
          }, SLOW_MS);
          expect(drawnColumns()).toEqual([...columns]);
        },
      );

      it('writes Profit on the result of a closed garden, with no colour for a profit', async () => {
        await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        // Shahpur is closed with a profit of 11,400 (E2E-40).
        expect(cell('0', 'balance')).toContain('11,400');
        expect(cell('0', 'balance')).toContain(m('report-result-profit'));
        expect(document.querySelector('.ag-row[row-id="0"] [col-id="balance"]')).not.toHaveClass(
          'cell-tone-due',
        );
      });

      it('writes Loss, in red, on a closed garden’s loss, but only the sign on an open garden below zero (FR-12.38)', async () => {
        // Row 0: an open garden, running balance −30,000 (it starts below zero by design).
        // Row 1: a closed garden that lost 4,500.
        const base = R36_GARDEN.rows[0];
        if (base === undefined) throw new Error('The R36 fixture has no row.');
        const gardenRow = (state: string, paisa: number): ReportRow => ({
          ...base,
          cells: base.cells.map((found, index): ReportCell => {
            if (index === STATUS_COLUMN) return { type: 'label', id: state };
            if (index === BALANCE_COLUMN) return { type: 'money', paisa: paisaForTest(paisa) };
            return found;
          }),
        });
        const page: ReportPage = {
          ...R36_GARDEN,
          rows: [
            gardenRow('report-garden-open', -3_000_000),
            gardenRow('report-garden-closed', -450_000),
          ],
        };
        await openGardens({ pages: { R36: page } });
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0', '1']);
        }, SLOW_MS);
        const open = cell('0', 'balance');
        expect(open).toContain('30,000');
        expect(open).not.toContain(m('report-result-loss'));
        expect(open).not.toContain(m('report-result-profit'));
        expect(document.querySelector('.ag-row[row-id="0"] [col-id="balance"]')).not.toHaveClass(
          'cell-tone-due',
        );
        const closed = cell('1', 'balance');
        expect(closed).toContain('4,500');
        expect(closed).toContain(m('report-result-loss'));
        expect(document.querySelector('.ag-row[row-id="1"] [col-id="balance"]')).toHaveClass(
          'cell-tone-due',
        );
        // The totals row adds open and closed gardens, so it carries no word.
        expect(totalsCell('balance')).not.toContain(m('report-result-loss'));
        expect(totalsCell('balance')).not.toContain(m('report-result-profit'));
      });

      it('the layout box asks by partner and shows each partner’s percent and money, with his name', async () => {
        const { user } = await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        await choose(user, 'report-filter-garden-view', m('report-filter-garden-by-partner'));
        await waitFor(() => {
          expect(lastRun()?.filters.garden_view).toBe('partner');
        }, SLOW_MS);
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0', '1']);
        }, SLOW_MS);
        expect(drawnColumns()).toEqual([
          'garden',
          'partner',
          'share',
          'paid_shop',
          'paid_pocket',
          'money_back',
          'result',
        ]);
        await waitFor(() => {
          expect(cell('0', 'partner')).toBe(names('الف', 'Partner A'));
        }, SLOW_MS);
        expect(cell('0', 'share')).toBe('60.00%');
        expect(cell('0', 'paid_shop')).toBe('33,000');
        expect(cell('0', 'money_back')).toBe('39,840');
        expect(cell('0', 'result')).toBe('6,840');
        expect(cell('1', 'paid_pocket')).toBe('22,000');
        expect(totalsCell('result')).toBe('11,400');
      });

      it('the garden box lists the gardens by name with “all gardens” first, and asks for the one picked', async () => {
        const { user } = await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        await choose(
          user,
          'report-filter-garden',
          `0080 ${names('شاہ پور باغ', 'Shahpur Garden')}`,
        );
        await waitFor(() => {
          expect(lastRun()?.filters.garden_id).toBe(1);
        }, SLOW_MS);
        expect(gardensApi.gardensList).toHaveBeenCalled();
      });

      it('Enter on a garden opens its screen', async () => {
        const { router, user } = await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        await user.click(pick('.ag-row[row-id="0"] [col-id="garden"]'));
        await user.keyboard('{Enter}');
        await waitFor(() => {
          expect(router.state.location.pathname).toBe('/gardens/1');
        }, SLOW_MS);
      });

      it('says there is nothing, and offers to clear a filter that is set', async () => {
        const { user } = await openGardens({
          pages: { R36: { ...R36_GARDEN, rows: [], total_count: 0 } },
        });
        await screen.findByText(named(m('report-empty')), undefined, SLOW_MS);
        expect(screen.queryByRole('button', { name: named(m('report-clear-filters')) })).toBeNull();
        await choose(
          user,
          'report-filter-garden',
          `0080 ${names('شاہ پور باغ', 'Shahpur Garden')}`,
        );
        expect(
          await screen.findByRole('button', { name: named(m('report-clear-filters')) }, SLOW_MS),
        ).toBeVisible();
      });

      it('keeps the garden and the layout across a language switch, and asks nothing new', async () => {
        const { user } = await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        await choose(
          user,
          'report-filter-garden',
          `0080 ${names('شاہ پور باغ', 'Shahpur Garden')}`,
        );
        await choose(user, 'report-filter-garden-view', m('report-filter-garden-by-partner'));
        await waitFor(() => {
          expect(lastRun()?.filters).toMatchObject({ garden_id: 1, garden_view: 'partner' });
        }, SLOW_MS);
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(2);
        }, SLOW_MS);
        const before = runCount();
        act(() => {
          useLanguageStore.getState().setLanguage(other);
        });
        await waitFor(() => {
          expect(
            screen.getByRole('combobox', { name: message(other, 'report-filter-garden-view') }),
          ).toHaveTextContent(message(other, 'report-filter-garden-by-partner'));
        }, SLOW_MS);
        expect(
          screen.getByRole('combobox', { name: message(other, 'report-filter-garden') }),
        ).toHaveTextContent(other === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden');
        await waitFor(() => {
          expect(cell('0', 'partner')).toBe(other === 'ur' ? 'الف' : 'Partner A');
        }, SLOW_MS);
        expect(runCount()).toBe(before);
      });

      it('prints the report by partner with every column and the partner’s name', async () => {
        const { user } = await openGardens({ print: PRINT_R36_PARTNER_EN });
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
        await screen.findByText(printFirmName(language), undefined, SLOW_MS);
        expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
          id: 'R36',
          language,
          paper: 'a4',
        });
        const heads = Array.from(document.querySelectorAll('.print-report thead th')).map((th) =>
          visible(th.textContent),
        );
        expect(heads).toHaveLength(PRINT_R36_PARTNER_EN.columns.length);
        expect(visible(document.querySelector('.print-report tbody')?.textContent)).toContain(
          'Partner A',
        );
      });
    });

    describe('R34', () => {
      it('has two rows for the gardens: what was paid into them and what came in, as information', async () => {
        // The part of the day with the trips and the gardens, as a short list that is drawn whole (the
        // short buyers' and sellers' tables of the day are left out: they have their own tests).
        const block: ReportPage = {
          ...R34_GARDEN_DAY,
          rows: R34_GARDEN_DAY.rows.slice(24, 31),
          total_count: 7,
          tables: [],
        };
        fakeReports({ pages: { R34: block } });
        await renderApp('/reports/R34');
        await screen.findByRole('heading', { level: 1, name: named(m('report-r34-title')) });
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(7);
        }, SLOW_MS);
        expect(cell('1', 'item')).toBe(m('report-r34-trips-sold'));
        expect(cell('2', 'item')).toBe(m('report-r34-gardens-paid'));
        expect(cell('2', 'count')).toBe('0');
        expect(cell('2', 'amount')).toBe('0');
        expect(cell('3', 'item')).toBe(m('report-r34-gardens-in'));
        // The money that came into the gardens' accounts on 12-10-26: the sale in our shop and the other shop.
        expect(cell('3', 'amount')).toBe('66,400');
        expect(cell('3', 'count')).toBe('');
        // The fruit row (D4R 17d) comes right after the gardens' two, before the partners' shares.
        expect(cell('4', 'item')).toBe(m('report-r34-fruit-taken'));
        expect(cell('5', 'item')).toBe(m('report-r34-partner-shares'));
      });
    });
  },
);
