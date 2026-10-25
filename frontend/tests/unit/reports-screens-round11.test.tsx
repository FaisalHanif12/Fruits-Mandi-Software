// The party day sheet (R35, PRD FR-19.12, FR-19.2, FR-19.10, FR-19.13, FR-13.5, E2E-43) on the ONE
// report screen (round 11), in the Urdu and in the English app, against a fake backend that answers
// with the backend's own fixtures. The table of every party (the buyers, then the sellers, a group
// row for each with its own figures, a blank column for the collector's note that waits for the wide
// step) and the lines under it (the shop's total اگراہی and the sellers' totals); the slip of one
// party, opened with Enter on his row (a one-time person with his key), with its footer lines in the
// order the backend sends, under the lines like the client's own slip; the way back to all parties;
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
import { fakeGardens } from '../helpers/gardens';
import { fakePartners } from '../helpers/partners';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import {
  CATALOG,
  PRINT_R35_SLIP_AKRAM_EN,
  PRINT_R36_PARTNER_EN,
  R34_GARDEN_DAY,
  R35_TABLE,
  R36_GARDEN,
  fakeReports,
} from '../helpers/reports';

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
    it('is in the catalogue with a day box and a party box, and no period', async () => {
      expect(CATALOG.find((entry) => entry.id === 'R35')).toMatchObject({
        filters: ['as_of', 'party'],
        party_required: false,
      });
      await openSheet();
      expect(screen.getByLabelText(m('report-filter-day'))).toBeVisible();
      expect(screen.getByRole('combobox', { name: m('report-filter-party') })).toBeVisible();
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
        limit: 100,
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
      expect(cell('2', 'code')).toBe('1');
      expect(cell('2', 'party')).toBe(`03040001 ${names('اکرم ٹول ٹیکس', 'Akram Toll Tax')}`);
      expect(cell('2', 'debit')).toBe('12,880');
      expect(cell('2', 'credit')).toBe('10,000');
      expect(cell('2', 'remaining')).toContain('2,880');
      expect(cell('2', 'remaining')).toContain(m('report-side-dr'));
      // A one-time buyer who paid on the spot has nothing left, and says so with the word for zero.
      expect(cell('3', 'party')).toContain(names('شاہد', 'Shahid'));
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
        await screen.findByText(named(m('report-day-shown', { date: '10-10-2026' }))),
      ).toBeVisible();
      expect(screen.queryByText(named(m('report-as-of-shown', { date: '10-10-2026' })))).toBeNull();
    });
  });

  describe('the slip of one party', () => {
    it('Enter on a buyer opens his slip: the lines of his day and, under them, what the day comes to', async () => {
      const { user } = await openSheet();
      await openSlip(user, '2');
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['qty', 'item', 'rate', 'amount']);
      }, SLOW_MS);
      expect(lastRun()?.filters.party).toEqual({ account_id: 20, person_key: null });
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(names('پیٹی کیلا', 'Box Banana'));
      }, SLOW_MS);
      expect(drawnRows()).toEqual(['0']);
      expect(cell('0', 'qty')).toBe('20');
      expect(cell('0', 'rate')).toBe('600');
      expect(cell('0', 'amount')).toBe('12,880');
      expect(totalsCell('qty')).toBe('20');
      expect(totalsCell('amount')).toBe('12,880');
      // The footer is in the order of the client's own slip, as the backend sent it.
      const lines = footerLines();
      expect(lines).toHaveLength(5);
      expect(lines[0]).toContain(m('report-r35-total-qty'));
      expect(lines[0]).toContain('20');
      expect(lines[1]).toContain(m('report-r35-previous'));
      expect(lines[2]).toContain(m('report-r35-bought'));
      expect(lines[2]).toContain('12,880');
      // A buyer's slip says what was received (the client's وصولی), not what was paid.
      expect(lines[3]).toContain(m('report-r35-received'));
      expect(lines[3]).toContain('10,000');
      expect(lines[4]).toContain(m('report-r35-remaining'));
      expect(lines[4]).toContain('2,880');
      expect(lines[4]).toContain(m('report-side-dr'));
      // No line the backend did not send: nothing on the spot, no other debits.
      expect(visible(document.body.textContent)).not.toContain(m('report-r35-on-spot'));
      expect(visible(document.body.textContent)).not.toContain(m('report-r35-other-debits'));
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

    it('Enter on a seller opens his slip with the cash given and what was paid', async () => {
      const { user } = await openSheet();
      await openSlip(user, '5');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({ account_id: 18, person_key: null });
      }, SLOW_MS);
      await waitFor(() => {
        expect(footerLines().some((line) => line.includes(m('report-r35-cash-given')))).toBe(true);
      }, SLOW_MS);
      const lines = footerLines();
      expect(lines).toHaveLength(6);
      expect(lines[1]).toContain(m('report-r35-previous'));
      expect(lines[1]).toContain('15,000');
      expect(lines[2]).toContain(m('report-r35-sold'));
      expect(lines[2]).toContain('24,316');
      expect(lines[3]).toContain(m('report-r35-cash-given'));
      expect(lines[3]).toContain('5,000');
      expect(lines[4]).toContain(m('report-r35-paid'));
      expect(lines[4]).toContain('4,316');
      expect(lines[5]).toContain(m('report-r35-remaining'));
      // A seller's rate is the lot's, so the backend sends none and the cell is blank.
      expect(cell('0', 'rate')).toBe('');
      expect(cell('0', 'amount')).toBe('24,316');
    });

    it('a one-time buyer is asked for with his own key, so his slip is his and not the whole account’s', async () => {
      const { user } = await openSheet();
      await openSlip(user, '3');
      await waitFor(() => {
        expect(lastRun()?.filters.party).toEqual({
          account_id: 9,
          person_key: 'شاہد|03000000000',
        });
      }, SLOW_MS);
      // His person box shows him by name, under the account box.
      await waitFor(() => {
        expect(screen.getByRole('combobox', { name: m('report-filter-person') })).toHaveValue(
          names('شاہد', 'Shahid'),
        );
      }, SLOW_MS);
    });

    it('Enter on a line of the slip opens the بکری behind it', async () => {
      const { router, user } = await openSheet();
      await openSlip(user, '2');
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(names('پیٹی کیلا', 'Box Banana'));
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="item"]'));
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });

    it('“All parties” goes back to the table, with the day still typed', async () => {
      const { user } = await openSheet();
      await user.type(screen.getByLabelText(m('report-filter-day')), '10-10-2026');
      await openSlip(user, '2');
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['qty', 'item', 'rate', 'amount']);
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
    it('is the client’s slip: the lines first and what the day comes to under them, with the day named "Day"', async () => {
      const { user } = await openSheet({ print: PRINT_R35_SLIP_AKRAM_EN });
      await openSlip(user, '2');
      await waitFor(() => {
        expect(drawnColumns()).toEqual(['qty', 'item', 'rate', 'amount']);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByText(PRINT_R35_SLIP_AKRAM_EN.firm.name, undefined, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R35',
        language,
        paper: 'a4',
      });
      const table = pick('.print-report .print-table');
      const summary = pick('.print-report .print-summary-lines');
      expect(
        table.compareDocumentPosition(summary) & Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
      const labels = Array.from(summary.querySelectorAll('dt')).map((term) =>
        visible(term.textContent),
      );
      expect(labels).toEqual(
        PRINT_R35_SLIP_AKRAM_EN.summary.map((line) => message('en', line.label_id)),
      );
      // The day is named "Day" in the header, not "As of".
      const facts = visible(pick('.print-report .print-facts').textContent);
      expect(facts).toContain(message('en', 'report-filter-day'));
      expect(facts).not.toContain(message('en', 'report-filter-as-of'));
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
        expect(footerLines()).toHaveLength(5);
      }, SLOW_MS);
      const before = runCount();
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(footerLines()[2]).toContain(message(other, 'report-r35-bought'));
      }, SLOW_MS);
      expect(screen.getByLabelText(message(other, 'report-filter-day'))).toHaveValue('10-10-2026');
      expect(boxText(message(other, 'report-filter-party'))).toContain(
        other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
      );
      expect(footerLines()[4]).toContain('2,880');
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
        expect(drawnColumns()).toEqual(['qty', 'item', 'rate', 'amount']);
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
          limit: 100,
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
        await choose(user, 'report-filter-garden', names('شاہ پور باغ', 'Shahpur Garden'));
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
        await choose(user, 'report-filter-garden', names('شاہ پور باغ', 'Shahpur Garden'));
        expect(
          await screen.findByRole('button', { name: named(m('report-clear-filters')) }, SLOW_MS),
        ).toBeVisible();
      });

      it('keeps the garden and the layout across a language switch, and asks nothing new', async () => {
        const { user } = await openGardens();
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(1);
        }, SLOW_MS);
        await choose(user, 'report-filter-garden', names('شاہ پور باغ', 'Shahpur Garden'));
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
        await screen.findByText(PRINT_R36_PARTNER_EN.firm.name, undefined, SLOW_MS);
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
        // The part of the day with the trips and the gardens, as a short list that is drawn whole.
        const block: ReportPage = {
          ...R34_GARDEN_DAY,
          rows: R34_GARDEN_DAY.rows.slice(15, 21),
          total_count: 6,
        };
        fakeReports({ pages: { R34: block } });
        await renderApp('/reports/R34');
        await screen.findByRole('heading', { level: 1, name: named(m('report-r34-title')) });
        await waitFor(() => {
          expect(drawnRows()).toHaveLength(6);
        }, SLOW_MS);
        expect(cell('1', 'item')).toBe(m('report-r34-trips-sold'));
        expect(cell('2', 'item')).toBe(m('report-r34-gardens-paid'));
        expect(cell('2', 'count')).toBe('0');
        expect(cell('2', 'amount')).toBe('0');
        expect(cell('3', 'item')).toBe(m('report-r34-gardens-in'));
        // The money that came into the gardens' accounts on 12-10-26: the sale in our shop and the other shop.
        expect(cell('3', 'amount')).toBe('66,400');
        expect(cell('3', 'count')).toBe('');
        expect(cell('4', 'item')).toBe(m('report-r34-partner-shares'));
      });
    });
  },
);
