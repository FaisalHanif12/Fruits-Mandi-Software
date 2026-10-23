// تجزیات (plan 21, PRD 18, FR-18.1 to FR-18.8): the six views on the one analytics screen, in the Urdu
// and in the English app, against a fake backend that answers with the backend's own fixtures. Each
// view asks for exactly what the filters hold and shows the backend's totals and shares; a comparison
// without its dates is refused under the date boxes; a figure opens the lines behind it and a line its
// بکری; saved views are named, opened and deleted; the print preview, the chart, the window of the
// columns of views D and E, the four states, a language switch in the middle of choosing filters and
// the keyboard-only path are each proved. The screen adds up nothing: every figure is one the backend
// sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import * as analyticsApi from '../../src/features/analytics/api';
import { EMPTY_ANALYTICS_FILTERS } from '../../src/features/analytics/filters';
import { useAnalyticsStore } from '../../src/features/analytics/store';
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
import { message, visible } from '../helpers/render';
import {
  ANALYTICS_PRINT_DATA_D_UR,
  ANALYTICS_RUN_A_REQUEST,
  ANALYTICS_RUN_D_MONTH,
  VIEWS_GET_TWO,
  VIEWS_SAVE_TWO_REQUEST,
  fakeAnalytics,
  type AnalyticsOptions,
} from '../helpers/analytics';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one; a test that needs the compact
// step (the fold of the filter boxes) sets it.
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
  act(() => {
    useAnalyticsStore.getState().reset();
  });
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

/** The words of a column heading, with the spaces of the grid taken out. */
function headerText(colId: string): string {
  const header = document.querySelector(`.ag-header-cell[col-id="${colId}"]`);
  return visible(header?.textContent).replace(/\s+/g, ' ').trim();
}

/** The element of a selector, or a plain failure that names it. */
function pick(selector: string): Element {
  const found = document.querySelector(selector);
  if (found === null) throw new Error(`nothing matches ${selector}`);
  return found;
}

function lastRun() {
  return vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0];
}

function lastLines() {
  return vi.mocked(analyticsApi.analyticsLines).mock.calls.at(-1)?.[0];
}

describe.each(LANGUAGES)('تجزیات (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const VIEW_IDS = ['a', 'b', 'c', 'd', 'e', 'h'] as const;

  async function openView(view: string, options: AnalyticsOptions = {}) {
    const fake = fakeAnalytics(options);
    const path = view === '' ? '/analytics' : `/analytics/${view}`;
    const shown = view === '' ? 'a' : view.toLowerCase();
    const result = await renderApp(path);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(`analytics-view-${shown}-title`)),
    });
    return { ...result, fake, user: userEvent.setup() };
  }

  describe('the area', () => {
    it('opens on view A, with a tab for each of the six views, and asks for nothing filtered', async () => {
      await openView('');
      const tabs = screen.getByRole('tablist', { name: m('analytics-views-label') });
      const labels = within(tabs)
        .getAllByRole('tab')
        .map((tab) => visible(tab.textContent));
      expect(labels).toEqual(VIEW_IDS.map((id) => m(`analytics-view-${id}-title`)));
      await waitFor(() => {
        expect(lastRun()).toEqual(ANALYTICS_RUN_A_REQUEST_WITH_PAGE);
      }, SLOW_MS);
    });

    it('moves between the views with the tabs, and keeps the filters', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-period') }));
      await user.click(
        await screen.findByRole('option', { name: m('analytics-period-this-year') }),
      );
      await user.click(screen.getByRole('tab', { name: m('analytics-view-b-title') }));
      await screen.findByRole('heading', { level: 1, name: named(m('analytics-view-b-title')) });
      await waitFor(() => {
        expect(lastRun()?.view).toBe('B');
        expect(lastRun()?.filters.period).toBe('this_year');
      }, SLOW_MS);
    });

    it('says a view that does not exist is not there, and offers the first', async () => {
      fakeAnalytics();
      const { router } = await renderApp('/analytics/Z');
      expect(
        await screen.findByText(named(m('analytics-view-unknown')), undefined, SLOW_MS),
      ).toBeVisible();
      await userEvent
        .setup()
        .click(screen.getByRole('button', { name: named(m('analytics-back-to-views')) }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/analytics/A');
      }, SLOW_MS);
    });
  });

  describe('view A by item (E2E-11, PRD 18.4)', () => {
    it('shows each item with its share, the totals row and the days it covers', async () => {
      await openView('A');
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
      }, SLOW_MS);
      expect(cell('0', 'qty')).toBe('50');
      expect(cell('0', 'gross')).toBe('29,400');
      expect(cell('0', 'total_comm')).toBe('3,928');
      // 72.10 percent, written from hundredths with no division.
      expect(cell('0', 'share')).toBe('72.10%');
      expect(cell('1', 'share')).toBe('27.90%');
      expect(totalsCell('total_comm')).toBe('5,448');
      expect(totalsCell('share')).toBe('100.00%');
      expect(totalsCell('qty')).toBe('60');
      expect(screen.getByText(named(m('report-range-all')))).toBeInTheDocument();
    });

    it('draws a chart above the table that is labelled for a screen reader', async () => {
      await openView('A');
      const chart = await screen.findByRole(
        'img',
        { name: named(m('analytics-chart-a')) },
        SLOW_MS,
      );
      expect(chart.getAttribute('data-chart')).toBe('bar');
      expect(chart.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
    });

    it('draws only the columns that fit the window, the print has the rest', async () => {
      await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      // The test window has no layout, so it is the normal step: avg_rate waits for the wide step.
      expect(drawnColumns()).toEqual([
        'item',
        'qty',
        'gross',
        'seller_comm',
        'buyer_comm',
        'laga',
        'total_comm',
        'share',
      ]);
      // It says so, with the numbers, so a column that waits is never a surprise.
      expect(
        await screen.findByText(named(m('analytics-columns-fit', { shown: 8, total: 9 }))),
      ).toBeInTheDocument();
    });

    it('shows the average rate of E2E-11 on a press, in the same room, and goes back on another', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      expect(drawnColumns()).not.toContain('avg_rate');

      await user.click(screen.getByRole('button', { name: named(m('analytics-columns-other')) }));
      // The average rate of both items is the backend's, 588 and 1,200; the laga column made room.
      await waitFor(() => {
        expect(drawnColumns()).toContain('avg_rate');
      }, SLOW_MS);
      expect(drawnColumns()).not.toContain('laga');
      expect(drawnColumns()).toHaveLength(8);
      expect(cell('0', 'avg_rate')).toBe('588');
      expect(cell('1', 'avg_rate')).toBe('1,200');
      expect(totalsCell('item')).toBe(m('report-total'));
      // The same text still says 8 of 9, and the button offers the way back.
      expect(
        screen.getByText(named(m('analytics-columns-fit', { shown: 8, total: 9 }))),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(m('analytics-columns-main')) }));
      await waitFor(() => {
        expect(drawnColumns()).toContain('laga');
      }, SLOW_MS);
      expect(drawnColumns()).not.toContain('avg_rate');
    });

    it('keeps the choice of the other columns through a language switch', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('analytics-columns-other')) }));
      await waitFor(() => {
        expect(drawnColumns()).toContain('avg_rate');
      }, SLOW_MS);
      // The choice lives above the screen (the store), so building the grid again keeps it.
      expect(useAnalyticsStore.getState().otherColumns).toBe(true);
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(other === 'ur' ? 'کیلا' : 'Banana');
      }, SLOW_MS);
      expect(drawnColumns()).toContain('avg_rate');
      expect(drawnColumns()).not.toContain('laga');
      expect(cell('0', 'avg_rate')).toBe('588');
    });
  });

  describe('the other views', () => {
    it('B shows each seller with his net sale', async () => {
      await openView('B');
      await waitFor(() => {
        expect(cell('0', 'net')).toBe('19,316');
      }, SLOW_MS);
      expect(cell('0', 'seller')).toBe(`030110001 ${names('رحیم', 'Rahim')}`);
      expect(totalsCell('net')).toBe('29,476');
    });

    it('C shows a buyer’s dues with their side as a word, a one-time buyer by his own name, and no days when none was paid', async () => {
      await openView('C');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2']);
      }, SLOW_MS);
      expect(cell('0', 'dues_now')).toContain('18,684');
      expect(cell('0', 'dues_now')).toContain(m('report-side-dr'));
      expect(cell('2', 'buyer')).toBe(`03049999 ${names('شاہد', 'Shahid')}`);
      expect(cell('2', 'avg_days')).toBe('0');
      expect(cell('0', 'avg_days')).toBe('');
    });

    it('D names the month in a column of its own and the total after it', async () => {
      await openView('D');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      expect(headerText('p_2026_10')).toBe(`${m('report-month-10')} 2026`);
      expect(totalsCell('total_comm')).toBe('5,448');
      expect(drawnColumns()).toEqual(['item', 'p_2026_10', 'total_comm']);
    });

    it('E has a column for each item, headed by the item’s name, and a blank where he sold none', async () => {
      await openView('E');
      await waitFor(() => {
        expect(cell('0', 'i_1')).toBe('3,928');
      }, SLOW_MS);
      expect(cell('0', 'i_2')).toBe('');
      expect(cell('1', 'i_2')).toBe('1,520');
      expect(headerText('i_1')).toBe(names('کیلا', 'Banana'));
    });

    it('H heads each list with its words, and ranks inside it', async () => {
      await openView('H');
      await waitFor(() => {
        expect(cell('0', 'rank')).toBe(m('analytics-top-items'));
      }, SLOW_MS);
      expect(cell('1', 'rank')).toBe('1');
      expect(cell('1', 'name')).toBe(names('کیلا', 'Banana'));
      expect(cell('1', 'share')).toBe('72.10%');
      expect(document.querySelector('.ag-row[row-id="0"]')?.className).toContain('row-group');
      // A chart of the first list only.
      expect(
        await screen.findByRole('img', { name: named(m('analytics-chart-h')) }),
      ).toBeInTheDocument();
    });
  });

  describe('the filters', () => {
    it('a shortcut is sent with no dates, and the days the backend read are shown', async () => {
      const { user } = await openView('A');
      await user.click(screen.getByRole('combobox', { name: m('report-filter-period') }));
      await user.click(await screen.findByRole('option', { name: m('report-period-this-month') }));
      await waitFor(() => {
        expect(lastRun()?.filters.period).toBe('this_month');
      }, SLOW_MS);
      expect(lastRun()?.filters.from).toBe('');
      expect(screen.queryByLabelText(m('report-filter-from'))).toBeNull();
    });

    it('sends the dates as typed once they have rested, and a comparison with both of them', async () => {
      const { user } = await openView('A');
      await user.type(screen.getByLabelText(m('report-filter-from')), '10-10-2026');
      await user.type(screen.getByLabelText(m('report-filter-to')), '10-10-2026');
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-compare') }));
      await user.click(
        await screen.findByRole('option', { name: m('analytics-compare-previous') }),
      );
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({
          from: '10-10-2026',
          to: '10-10-2026',
          compare: 'previous',
        });
      }, SLOW_MS);
      // The compared period and the change are columns, and the days compared with are said.
      await waitFor(() => {
        expect(drawnColumns()).toContain('compare');
      }, SLOW_MS);
      expect(drawnColumns()).toContain('change');
      expect(
        screen.getByText((text) =>
          visible(text).includes(
            m('analytics-compare-range', { from: '09-10-2026', to: '09-10-2026' }),
          ),
        ),
      ).toBeInTheDocument();
    });

    it('refuses a comparison without both dates under the two date boxes, and the rows stay', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-compare') }));
      await user.click(
        await screen.findByRole('option', { name: m('analytics-compare-last-year') }),
      );
      await waitFor(() => {
        expect(screen.getAllByText(named(m('err-required'))).length).toBeGreaterThanOrEqual(2);
      }, SLOW_MS);
      // The rows already on screen stay; this is not the error state.
      expect(drawnRows()).toEqual(['0', '1']);
      expect(screen.queryByText(named(m('analytics-load-failed')))).toBeNull();
    });

    it('shows that refusal under the comparison box when a shortcut hides the date boxes', async () => {
      const { user } = await openView('A', {
        refuse: (input) =>
          input.filters.compare === 'off'
            ? null
            : {
                ok: false,
                errors: [{ code: 'ERR_REQUIRED', field: 'filters.from', values: {} }],
                ref: 'E-1',
              },
      });
      await user.click(screen.getByRole('combobox', { name: m('report-filter-period') }));
      await user.click(await screen.findByRole('option', { name: m('report-period-this-month') }));
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-compare') }));
      await user.click(
        await screen.findByRole('option', { name: m('analytics-compare-previous') }),
      );
      await waitFor(() => {
        expect(screen.getByText(named(m('err-required')))).toBeInTheDocument();
      }, SLOW_MS);
    });

    it('asks for the days of view D when the grouping says so, and only view D has that box', async () => {
      const { user } = await openView('D');
      await user.click(screen.getByRole('combobox', { name: m('report-filter-group-by') }));
      await user.click(await screen.findByRole('option', { name: m('analytics-group-by-day') }));
      await waitFor(() => {
        expect(lastRun()?.filters.group_by).toBe('day');
      }, SLOW_MS);
      await waitFor(() => {
        expect(headerText('p_2026_10_10')).toBe('10-10-2026');
      }, SLOW_MS);
      await user.click(screen.getByRole('tab', { name: m('analytics-view-a-title') }));
      await screen.findByRole('heading', { level: 1, name: named(m('analytics-view-a-title')) });
      expect(screen.queryByRole('combobox', { name: m('report-filter-group-by') })).toBeNull();
    });

    it('sends the side asked for, and the seller side leaves the buyer side blank', async () => {
      const { user } = await openView('A');
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-side') }));
      await user.click(
        await screen.findByRole('option', { name: named(m('analytics-side-seller')) }),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.side).toBe('seller');
      }, SLOW_MS);
    });

    it('says what to do when nothing matches, and clears the filters on the button', async () => {
      const { user } = await openView('A', {
        pages: { A: { ...ANALYTICS_RUN_A_EMPTY } },
      });
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-side') }));
      await user.click(
        await screen.findByRole('option', { name: named(m('analytics-side-buyer')) }),
      );
      expect(
        await screen.findByText(named(m('analytics-empty')), undefined, SLOW_MS),
      ).toBeVisible();
      await user.click(screen.getByRole('button', { name: named(m('report-clear-filters')) }));
      await waitFor(() => {
        expect(lastRun()?.filters.side).toBe('both');
      }, SLOW_MS);
    });
  });

  describe('the filter boxes in a compact window', () => {
    it('keeps period, dates, item and seller in view, folds the rest behind “More filters”, and opens them on a press', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      const { user } = await openView('A');
      const folded = [
        'analytics-filter-buyer',
        'report-filter-city',
        'report-filter-adda',
        'analytics-filter-side',
        'analytics-filter-compare',
      ];
      for (const id of ['report-filter-period', 'report-filter-item', 'report-filter-seller']) {
        expect(screen.getByRole('combobox', { name: m(id) })).toBeInTheDocument();
      }
      for (const id of folded) {
        // A folded box is not drawn at all, so the arrow keys never try to enter it.
        expect(screen.queryByRole('combobox', { name: m(id) })).toBeNull();
      }
      await user.click(screen.getByRole('button', { name: named(m('report-filters-more')) }));
      for (const id of folded) {
        expect(screen.getByRole('combobox', { name: m(id) })).toBeInTheDocument();
      }
      await user.click(screen.getByRole('button', { name: named(m('report-filters-fewer')) }));
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-side') })).toBeNull();
    });

    it('opens the fold by itself when a folded box holds a value', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      act(() => {
        useAnalyticsStore.getState().replace({ ...EMPTY_ANALYTICS_FILTERS, side: 'buyer' });
      });
      await openView('A');
      expect(
        screen.getByRole('combobox', { name: m('analytics-filter-side') }),
      ).toBeInTheDocument();
      expect(screen.getByRole('button', { name: named(m('report-filters-fewer')) })).toBeDisabled();
    });

    it('opens the fold by itself when the backend refuses a folded box', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      await openView('A', {
        refuse: (input) =>
          input.filters.compare === 'off'
            ? null
            : {
                ok: false,
                errors: [{ code: 'ERR_OUT_OF_RANGE', field: 'filters.compare', values: {} }],
                ref: 'E-1',
              },
      });
      act(() => {
        useAnalyticsStore.getState().change({ compare: 'previous' });
      });
      await waitFor(() => {
        expect(
          screen.getByRole('combobox', { name: m('analytics-filter-compare') }),
        ).toBeInTheDocument();
      }, SLOW_MS);
    });
  });

  describe('drill-down: the lines behind a number (FR-18.2)', () => {
    it('Enter on an item opens its lines, and Enter on a line opens its بکری', async () => {
      const { router, user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="item"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'A',
        row_key: { item_id: 1 },
        column_key: null,
        offset: 0,
        limit: 100,
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      // The lines carry the same figures as the row they came from.
      expect(totalsCell('total_comm')).toBe('3,928');
      await user.click(pick('.ag-row[row-id="0"] [col-id="date"]'));
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });

    it('Esc comes back to the view with its rows and its filters where they were', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.type(screen.getByLabelText(m('report-filter-from')), '10-10-2026');
      await user.click(pick('.ag-row[row-id="1"] [col-id="item"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(
          screen.queryByRole('heading', { level: 2, name: named(m('analytics-lines-title')) }),
        ).toBeNull();
      }, SLOW_MS);
      expect(screen.getByLabelText(m('report-filter-from'))).toHaveValue('10-10-2026');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
    });

    it('Enter on a month of view D sends that month with the row; on the totals it sends the whole filter', async () => {
      const { user } = await openView('D');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="p_2026_10"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'D',
        row_key: { item_id: 1 },
        column_key: { from: '2026-10-01', to: '2026-10-31' },
      });
      await user.click(screen.getByRole('button', { name: named(m('analytics-lines-back')) }));
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      // The totals row opens the lines of the whole filter.
      await user.click(pick('.ag-row-pinned [col-id="total_comm"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()?.row_key).toEqual({
        item_id: null,
        seller_id: null,
        buyer_id: null,
        person_key: null,
        from: null,
        to: null,
        not_item_ids: [],
      });
    });

    it('the “All lines” button opens every line of the filters', async () => {
      const { user } = await openView('B');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('analytics-all-lines')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()?.row_key.item_id).toBeNull();
      expect(lastLines()?.row_key.seller_id).toBeNull();
    });

    it('says what happened and tries again when the lines cannot be read', async () => {
      const { user } = await openView('A', {
        lines: new Error('down') as never,
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="item"]'));
      await user.keyboard('{Enter}');
      expect(
        await screen.findByText(named(m('analytics-lines-failed')), undefined, SLOW_MS),
      ).toBeVisible();
    });
  });

  describe('saved views (FR-18.5)', () => {
    it('lists them, and opening one goes to its view with its filters', async () => {
      const { router, user } = await openView('A', { saved: VIEWS_GET_TWO });
      await user.click(await screen.findByRole('combobox', { name: m('analytics-saved-label') }));
      await user.click(await screen.findByRole('option', { name: 'Top 10' }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/analytics/H');
      }, SLOW_MS);
      await screen.findByRole('heading', { level: 1, name: named(m('analytics-view-h-title')) });
      await user.click(screen.getByRole('combobox', { name: m('analytics-saved-label') }));
      await user.click(await screen.findByRole('option', { name: VIEWS_GET_TWO[0]?.name ?? '' }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/analytics/D');
      }, SLOW_MS);
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({
          from: '01-01-2026',
          to: '31-12-2026',
          item_id: 1,
        });
      }, SLOW_MS);
    });

    it('says there are none yet', async () => {
      await openView('A', { saved: [] });
      expect(await screen.findByText(named(m('analytics-saved-none')))).toBeInTheDocument();
    });

    it('saves the view and its filters under a name, as the backend’s own request shows', async () => {
      const { fake, user } = await openView('D', { saved: [] });
      act(() => {
        useAnalyticsStore.getState().replace({
          ...EMPTY_ANALYTICS_FILTERS,
          from: '01-01-2026',
          to: '31-12-2026',
          item: { id: 1, name_ur: 'کیلا', name_en: 'Banana', code: undefined },
        });
      });
      await user.click(
        await screen.findByRole('button', { name: named(m('analytics-saved-save')) }),
      );
      const box = await screen.findByLabelText(m('analytics-saved-name'));
      await user.type(box, VIEWS_SAVE_TWO_REQUEST.views[0]?.name ?? '');
      await user.click(screen.getByRole('button', { name: named(m('analytics-saved-confirm')) }));
      await waitFor(() => {
        expect(analyticsApi.analyticsViewsSave).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(analyticsApi.analyticsViewsSave).mock.calls[0]?.[0].views[0]).toEqual(
        VIEWS_SAVE_TWO_REQUEST.views[0],
      );
      expect(fake.saved()).toHaveLength(1);
      expect(
        await screen.findByText(
          named(m('analytics-saved-done', { name: VIEWS_SAVE_TWO_REQUEST.views[0]?.name ?? '' })),
        ),
      ).toBeInTheDocument();
    });

    it('shows the refusal of an empty name under the name box, and saves nothing', async () => {
      const { fake, user } = await openView('A', { saved: [] });
      await user.click(
        await screen.findByRole('button', { name: named(m('analytics-saved-save')) }),
      );
      await screen.findByLabelText(m('analytics-saved-name'));
      await user.click(screen.getByRole('button', { name: named(m('analytics-saved-confirm')) }));
      expect(await screen.findByText(named(m('err-required')), undefined, SLOW_MS)).toBeVisible();
      expect(fake.saved()).toHaveLength(0);
    });

    it('deletes the one that is open, after asking', async () => {
      const { fake, user } = await openView('A', { saved: VIEWS_GET_TWO });
      await user.click(await screen.findByRole('combobox', { name: m('analytics-saved-label') }));
      await user.click(await screen.findByRole('option', { name: 'Top 10' }));
      await screen.findByRole('heading', { level: 1, name: named(m('analytics-view-h-title')) });
      await user.click(
        await screen.findByRole('button', { name: named(m('analytics-saved-delete')) }),
      );
      expect(await screen.findByText(named(m('analytics-saved-delete-title')))).toBeVisible();
      const dialog = screen.getByRole('alertdialog');
      await user.click(
        within(dialog).getByRole('button', { name: named(m('analytics-saved-delete')) }),
      );
      await waitFor(() => {
        expect(fake.saved().map((saved) => saved.view)).toEqual(['D']);
      }, SLOW_MS);
    });
  });

  describe('the print preview', () => {
    it('F9 asks for every row in the print language, drawn with the month named', async () => {
      const { user } = await openView('D', { print: ANALYTICS_PRINT_DATA_D_UR });
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(analyticsApi.analyticsPrintData).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(analyticsApi.analyticsPrintData).mock.calls[0]?.[0]).toMatchObject({
        view: 'D',
        language,
        paper: 'a4',
      });
      const doc = await screen.findByText(ANALYTICS_PRINT_DATA_D_UR.firm.name, undefined, SLOW_MS);
      expect(doc).toBeInTheDocument();
      const heads = Array.from(document.querySelectorAll('.print-report thead th')).map((th) =>
        visible(th.textContent),
      );
      expect(heads).toContain(`${message('ur', 'report-month-10')} 2026`);
      // The print buttons wait for the print job, as in the reports.
      expect(screen.getByRole('button', { name: named(m('report-print-print')) })).toBeDisabled();
      expect(screen.getByRole('button', { name: named(m('report-print-pdf')) })).toBeDisabled();
    });

    it('Esc closes the preview and the view is where it was', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(
          screen.queryByRole('heading', { level: 2, name: named(m('report-print-title')) }),
        ).toBeNull();
      }, SLOW_MS);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
    });
  });

  describe('views D and E with more columns than fit', () => {
    const wide: AnalyticsPage = (() => {
      const headed = ANALYTICS_RUN_D_MONTH.columns.findIndex((column) => column.heading !== null);
      const template = ANALYTICS_RUN_D_MONTH.columns[headed];
      if (template === undefined) throw new Error('no headed column');
      const months = Array.from({ length: 12 }, (_, index) => index + 1);
      const columns = [
        ...ANALYTICS_RUN_D_MONTH.columns.slice(0, headed),
        ...months.map((month) => ({
          ...template,
          key: `p_2026_${String(month)}`,
          heading: { kind: 'month' as const, year: 2026, month },
        })),
        ...ANALYTICS_RUN_D_MONTH.columns.slice(headed + 1),
      ];
      const spread = <T,>(cells: T[]): T[] => {
        const own = cells[headed];
        if (own === undefined) return cells;
        return [...cells.slice(0, headed), ...months.map(() => own), ...cells.slice(headed + 1)];
      };
      return {
        ...ANALYTICS_RUN_D_MONTH,
        columns,
        rows: ANALYTICS_RUN_D_MONTH.rows.map((row) => ({ ...row, cells: spread(row.cells) })),
        totals: spread(ANALYTICS_RUN_D_MONTH.totals),
      };
    })();

    it('shows a window of the months with its place, and two buttons that move it', async () => {
      const { user } = await openView('D', { pages: { D: wide } });
      // The room of the normal step holds six months beside the item and the total.
      expect(
        await screen.findByText(
          named(m('analytics-columns-shown', { from: 1, to: 6, total: 12 })),
          undefined,
          SLOW_MS,
        ),
      ).toBeInTheDocument();
      await waitFor(() => {
        expect(drawnColumns()).toEqual([
          'item',
          'p_2026_1',
          'p_2026_2',
          'p_2026_3',
          'p_2026_4',
          'p_2026_5',
          'p_2026_6',
          'total_comm',
        ]);
      }, SLOW_MS);
      expect(
        screen.getByRole('button', { name: named(m('analytics-columns-earlier')) }),
      ).toBeDisabled();
      await user.click(screen.getByRole('button', { name: named(m('analytics-columns-later')) }));
      expect(
        await screen.findByText(
          named(m('analytics-columns-shown', { from: 7, to: 12, total: 12 })),
        ),
      ).toBeInTheDocument();
      await waitFor(() => {
        expect(drawnColumns()).toContain('p_2026_12');
      }, SLOW_MS);
      expect(
        screen.getByRole('button', { name: named(m('analytics-columns-later')) }),
      ).toBeDisabled();
      // The chart has every month, whatever the window shows.
      expect(screen.getByRole('img', { name: named(m('analytics-chart-d')) })).toBeInTheDocument();
    });
  });

  describe('the four states', () => {
    it('shows the loading state while the backend answers', async () => {
      fakeAnalytics();
      vi.mocked(analyticsApi.analyticsRun).mockImplementation(() => new Promise(() => undefined));
      await renderApp('/analytics/B');
      await screen.findByRole('heading', { level: 1, name: named(m('analytics-view-b-title')) });
      expect(await screen.findByRole('status')).toBeInTheDocument();
    });

    it('says what happened and tries again when the backend fails', async () => {
      let failing = true;
      const { user } = await openView('C', {
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
      await screen.findByText(named(m('analytics-load-failed')), undefined, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
    });

    it('says the list of views could not be read, and tries again', async () => {
      fakeAnalytics({ catalog: new Error('down') });
      await renderApp('/analytics');
      expect(
        await screen.findByText(named(m('analytics-catalog-failed')), undefined, SLOW_MS),
      ).toBeVisible();
    });
  });

  describe('a language switch in the middle of choosing filters', () => {
    it('keeps the dates, the comparison and the grouping, draws the table and the chart again in the other language, and asks nothing new', async () => {
      const { user } = await openView('D');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      await user.type(screen.getByLabelText(m('report-filter-from')), '01-10-2026');
      await user.click(screen.getByRole('combobox', { name: m('report-filter-group-by') }));
      await user.click(await screen.findByRole('option', { name: m('analytics-group-by-day') }));
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({ group_by: 'day', from: '01-10-2026' });
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('0', 'p_2026_10_10')).toBe('3,928');
      }, SLOW_MS);
      const before = vi.mocked(analyticsApi.analyticsRun).mock.calls.length;
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(screen.getByLabelText(message(other, 'report-filter-from'))).toHaveValue(
          '01-10-2026',
        );
      }, SLOW_MS);
      expect(
        screen.getByRole('combobox', { name: message(other, 'report-filter-group-by') }),
      ).toHaveTextContent(message(other, 'analytics-group-by-day'));
      await waitFor(() => {
        expect(cell('0', 'item')).toBe(other === 'ur' ? 'کیلا' : 'Banana');
      }, SLOW_MS);
      expect(cell('0', 'p_2026_10_10')).toBe('3,928');
      expect(
        screen.getByRole('img', { name: named(message(other, 'analytics-chart-d')) }),
      ).toBeInTheDocument();
      expect(vi.mocked(analyticsApi.analyticsRun).mock.calls.length).toBe(before);
    });
  });

  describe('keyboard only', () => {
    it('type the day, Tab to the table, ↓ to a row, Enter for the lines, Esc back, then a tab to view B', async () => {
      const { user } = await openView('A');
      // The first filter box has focus: the period; ↓ then Tab move on to the dates.
      const period = screen.getByRole('combobox', { name: m('report-filter-period') });
      await waitFor(() => {
        expect(document.activeElement).toBe(period);
      }, SLOW_MS);
      const from = screen.getByLabelText(m('report-filter-from'));
      await act(async () => {
        from.focus();
        await Promise.resolve();
      });
      await user.keyboard('10-10-2026');
      await waitFor(() => {
        expect(lastRun()?.filters.from).toBe('10-10-2026');
      }, SLOW_MS);
      // Tab to the table (the boxes and the buttons come first), a row with ↓, Enter for its lines.
      for (let tabs = 0; tabs < 40; tabs += 1) {
        if (document.activeElement?.closest('.ag-root-wrapper') != null) break;
        await user.tab();
      }
      expect(document.activeElement?.closest('.ag-root-wrapper')).not.toBeNull();
      await user.keyboard('{ArrowDown}{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(
          screen.queryByRole('heading', { level: 2, name: named(m('analytics-lines-title')) }),
        ).toBeNull();
      }, SLOW_MS);
      // A tab is reached with the keyboard too: → moves to the next view (← in Urdu).
      const tab = screen.getByRole('tab', { name: m('analytics-view-a-title') });
      await act(async () => {
        tab.focus();
        await Promise.resolve();
      });
      await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
      await screen.findByRole(
        'heading',
        { level: 1, name: named(m('analytics-view-b-title')) },
        SLOW_MS,
      );
    });
  });
});

/** The first request of view A: the backend's own request, one page of 100. */
const ANALYTICS_RUN_A_REQUEST_WITH_PAGE = { ...ANALYTICS_RUN_A_REQUEST, limit: 100 };

/** View A with nothing in it. */
const ANALYTICS_RUN_A_EMPTY = {
  ...ANALYTICS_RUN_D_MONTH,
  view: 'A' as const,
  rows: [],
  total_count: 0,
};
