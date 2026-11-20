// تجزیات, the measure "ناپ" (plan 84, PRD FR-18.10), in the Urdu and the English app, against a fake backend that
// answers by the measure it is asked for, with the numbers of the plan (October 2026, after Events 1 to 5):
//   view D by sales: کیلا 29,400, سیب 12,000, total 41,400; by the total commission 3,928, 1,520, 5,448;
//   by quantity 50, 10, 60 (a count, never rupees); view H by sales: items کیلا 29,400 then سیب 12,000, buyers
//   ارشد 17,400, اکرم 12,000, شاہد 12,000.
// The box shows on the views that take the measure and not on the others; a choice is sent, and the chart title,
// the name of its series, the row total and the matrix cells follow it; a saved view and the Owner's memory keep
// it; the print names it. The screen adds up nothing: every figure is one the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AnalyticsMeasure } from '../../src/bindings/AnalyticsMeasure';
import * as analyticsApi from '../../src/features/analytics/api';
import { EMPTY_ANALYTICS_FILTERS, filtersToWire } from '../../src/features/analytics/filters';
import { useAnalyticsStore } from '../../src/features/analytics/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import * as accountApi from '../../src/shared/accounts/api';
import * as screenApi from '../../src/shared/screen-state/api';
import * as searchApi from '../../src/shared/search/api';
import { fakeAnalytics, type AnalyticsOptions } from '../helpers/analytics';
import {
  MEASURES,
  MEASURE_CATALOG,
  MEASURE_VIEWS,
  NO_MEASURE_VIEWS,
  TOTAL,
  answerByMeasure,
  printDataD,
} from '../helpers/analytics-measure';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { PAGE_ROWS } from '../helpers/pager';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
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
  vi.mocked(accountApi.searchAccounts).mockResolvedValue([]);
  vi.mocked(accountApi.searchAccountsRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  act(() => {
    useAnalyticsStore.getState().reset();
  });
  window.localStorage.clear();
});

afterEach(() => {
  leaveAppWindow();
});

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function totalsCell(colId: string): string {
  const row = document.querySelector('.ag-row-pinned');
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

/** The words of a column heading, with the spaces of the grid taken out. */
function headerText(colId: string): string {
  const header = document.querySelector(`.ag-header-cell[col-id="${colId}"]`);
  return visible(header?.textContent).replace(/\s+/g, ' ').trim();
}

function drawnColumns(): string[] {
  return Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map(
    (header) => header.getAttribute('col-id') ?? '',
  );
}

/** What a column holds in every row that has something in it (the heading rows of view H are empty there). */
function valuesOf(colId: string): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)'))
    .map((row) => visible(row.querySelector(`[col-id="${colId}"]`)?.textContent))
    .filter((text) => text !== '');
}

/** The last page the view asked for, with the measure the screen sent. */
function lastRun() {
  return vi
    .mocked(analyticsApi.analyticsRun)
    .mock.calls.map((call) => call[0])
    .filter((input) => input.limit === PAGE_ROWS)
    .at(-1);
}

describe.each(LANGUAGES)('تجزیات, the measure (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const measureWords = (measure: AnalyticsMeasure) => m('analytics-measure-name', { measure });
  const viewTitle = (view: string) => m(`analytics-view-${view.toLowerCase()}-title`);

  async function openView(view: string, options: AnalyticsOptions = {}) {
    const fake = fakeAnalytics({ catalog: MEASURE_CATALOG, ...options });
    answerByMeasure();
    const result = await renderApp(`/analytics/${view}`);
    await screen.findByRole(
      'heading',
      { level: 1, name: (name) => visible(name).startsWith(viewTitle(view)) },
      SLOW_MS,
    );
    return { ...result, fake, user: userEvent.setup() };
  }

  /** Picks a measure in the box: the box is a drop-down, so it opens and one choice is picked. */
  async function choose(user: ReturnType<typeof userEvent.setup>, measure: AnalyticsMeasure) {
    await user.click(screen.getByRole('combobox', { name: m('analytics-filter-measure') }));
    // The words of a measure hold glossary terms, which Fluent isolates: compare what is seen.
    await user.click(
      await screen.findByRole('option', {
        name: (name) => visible(name) === measureWords(measure),
      }),
    );
  }

  /** The words the measure box shows now. */
  const boxText = () =>
    visible(screen.getByRole('combobox', { name: m('analytics-filter-measure') }).textContent);

  async function openSavedMenu(user: ReturnType<typeof userEvent.setup>) {
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(m('analytics-saved-label')),
      }),
    );
  }

  /** The charts whose title starts with these words (view H has one for each list). */
  const chartsNamed = (title: string) =>
    screen.findAllByRole('img', { name: (name) => visible(name).startsWith(title) }, SLOW_MS);
  const chartNamed = async (title: string) => {
    const found = await chartsNamed(title);
    expect(found).toHaveLength(1);
  };

  describe('the box', () => {
    it('shows on views A, B, C, D, E, G, H and L, and on no other view', async () => {
      const { router } = await openView('A');
      const box = () => screen.queryByRole('combobox', { name: m('analytics-filter-measure') });
      const go = async (view: string) => {
        await act(async () => {
          await router.navigate({ to: '/analytics/$view', params: { view } });
        });
        await screen.findByRole(
          'heading',
          { level: 1, name: (name) => visible(name).startsWith(viewTitle(view)) },
          SLOW_MS,
        );
      };
      for (const view of MEASURE_VIEWS) {
        await go(view);
        expect(box(), view).not.toBeNull();
      }
      for (const view of NO_MEASURE_VIEWS) {
        await go(view);
        expect(box(), view).toBeNull();
      }
    }, 60_000);

    it('starts with sales and offers the seven measures in order, in the words of the app', async () => {
      const { user } = await openView('D');
      const box = screen.getByRole('combobox', { name: m('analytics-filter-measure') });
      expect(visible(box.textContent)).toBe(measureWords('gross'));
      await user.click(box);
      const options = (await screen.findAllByRole('option')).map((option) =>
        visible(option.textContent),
      );
      expect(options).toEqual(MEASURES.map((measure) => measureWords(measure)));
    });

    it('is in view in the compact window, never behind “More filters”', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('compact');
      await openView('A');
      expect(
        screen.getByRole('combobox', { name: m('analytics-filter-measure') }),
      ).toBeInTheDocument();
    });
  });

  describe('view D, item × month', () => {
    it('asks for sales first, then the total commission and the quantity: cells, totals, row total and chart follow', async () => {
      const { user } = await openView('D');
      // Sales: the plan's October figures.
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('29,400');
      }, SLOW_MS);
      expect(lastRun()?.filters.measure).toBe('gross');
      expect(cell('1', 'p_2026_10')).toBe('12,000');
      expect(totalsCell('p_2026_10')).toBe('41,400');
      expect(totalsCell('gross')).toBe(TOTAL.gross.toLocaleString('en-US'));
      expect(headerText('gross')).toBe(m('analytics-col-row-total-gross'));
      await chartNamed(m('analytics-chart-d', { measure: 'gross' }));

      // The total commission.
      await choose(user, 'total_comm');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      expect(lastRun()?.filters.measure).toBe('total_comm');
      expect(cell('1', 'p_2026_10')).toBe('1,520');
      expect(totalsCell('p_2026_10')).toBe('5,448');
      expect(headerText('total_comm')).toBe(m('analytics-col-row-total'));
      await chartNamed(m('analytics-chart-d', { measure: 'total_comm' }));

      // The quantity is a count: 50, 10 and 60, with its own heading and chart title.
      await choose(user, 'qty');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('50');
      }, SLOW_MS);
      expect(lastRun()?.filters.measure).toBe('qty');
      expect(cell('1', 'p_2026_10')).toBe('10');
      expect(totalsCell('p_2026_10')).toBe('60');
      expect(headerText('qty')).toBe(m('analytics-col-row-total-qty'));
      await chartNamed(m('analytics-chart-d', { measure: 'qty' }));
      expect(boxText()).toBe(measureWords('qty'));
    }, 60_000);
  });

  describe('an order chosen on the measure column', () => {
    it('does not carry over to another measure: the matrix is read again in its own order, with no refusal', async () => {
      const { user } = await openView('D');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('29,400');
      }, SLOW_MS);
      // The row total of sales is sorted: the screen asks for the order on its key, `gross`.
      const title = document.querySelector('.ag-header-cell[col-id="gross"] .ag-header-cell-text');
      if (title === null) throw new Error('no heading for the row total');
      await user.click(title);
      await waitFor(() => {
        expect(lastRun()?.sort).toMatchObject({ column: 'gross' });
      }, SLOW_MS);
      // Another measure has no column `gross`: the order is dropped, and the backend refuses nothing.
      await choose(user, 'qty');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('50');
      }, SLOW_MS);
      expect(lastRun()?.sort).toBeNull();
      expect(screen.queryByText(m('analytics-load-failed'))).toBeNull();
    }, 60_000);
  });

  describe('view A, by item', () => {
    it('keeps every column of its table whatever the measure, and the chart names the measure', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(cell('0', 'total_comm')).toBe('3,928');
      }, SLOW_MS);
      const columns = drawnColumns();
      expect(columns).toContain('gross');
      expect(columns).toContain('total_comm');
      await chartNamed(m('analytics-chart-a', { measure: 'gross' }));

      await choose(user, 'buyer_total');
      await chartNamed(m('analytics-chart-a', { measure: 'buyer_total' }));
      expect(lastRun()?.filters.measure).toBe('buyer_total');
      // The table is the same: the same columns and the same figures.
      expect(drawnColumns()).toEqual(columns);
      expect(cell('0', 'total_comm')).toBe('3,928');
      expect(cell('0', 'gross')).toBe('29,400');
    }, 60_000);
  });

  describe('view H, the top 10', () => {
    it('lists by sales and then by quantity, with the lists and the charts following the measure', async () => {
      const { user } = await openView('H');
      await waitFor(() => {
        expect(valuesOf('gross')).toEqual([
          '29,400',
          '12,000',
          '29,400',
          '12,000',
          '17,400',
          '12,000',
          '12,000',
        ]);
      }, SLOW_MS);
      expect(headerText('gross')).toBe(m('report-col-gross'));
      expect(await chartsNamed(m('analytics-chart-h', { measure: 'gross' }))).toHaveLength(3);

      await choose(user, 'qty');
      await waitFor(() => {
        expect(valuesOf('qty')).toEqual(['50', '10', '50', '10', '30', '20', '10']);
      }, SLOW_MS);
      expect(lastRun()?.filters.measure).toBe('qty');
      expect(headerText('qty')).toBe(m('report-col-qty'));
      expect(await chartsNamed(m('analytics-chart-h', { measure: 'qty' }))).toHaveLength(3);
    }, 60_000);
  });

  describe('saved views and what the Owner’s screen remembers', () => {
    it('a saved view keeps the measure and opens with it', async () => {
      const { fake, user } = await openView('D', { saved: [] });
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('29,400');
      }, SLOW_MS);
      await choose(user, 'qty');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('50');
      }, SLOW_MS);
      await openSavedMenu(user);
      await user.click(await screen.findByRole('menuitem', { name: m('analytics-saved-save') }));
      await user.type(await screen.findByLabelText(m('analytics-saved-name')), 'qty by month');
      await user.click(screen.getByRole('button', { name: m('analytics-saved-confirm') }));
      await waitFor(() => {
        expect(fake.saved()).toHaveLength(1);
      }, SLOW_MS);
      expect(fake.saved()[0]).toMatchObject({
        name: 'qty by month',
        view: 'D',
        filters: { measure: 'qty' },
      });

      // Back to sales, then open the saved view: the box says quantity and the screen asks for it.
      await choose(user, 'gross');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('29,400');
      }, SLOW_MS);
      await openSavedMenu(user);
      await user.click(await screen.findByRole('menuitem', { name: 'qty by month' }));
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('50');
      }, SLOW_MS);
      expect(boxText()).toBe(measureWords('qty'));
      expect(lastRun()?.filters.measure).toBe('qty');
    }, 60_000);

    it('a view saved before the measure existed opens as the total commission it was', async () => {
      const before = filtersToWire(EMPTY_ANALYTICS_FILTERS, ['period', 'group_by'], 'D');
      delete before.measure;
      const { user } = await openView('A', {
        saved: [{ name: 'old commission', view: 'D', filters: before }],
      });
      await waitFor(() => {
        expect(lastRun()?.filters.measure).toBe('gross');
      }, SLOW_MS);
      await openSavedMenu(user);
      await user.click(await screen.findByRole('menuitem', { name: 'old commission' }));
      await screen.findByRole(
        'heading',
        { level: 1, name: (name) => visible(name).startsWith(viewTitle('D')) },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('3,928');
      }, SLOW_MS);
      expect(lastRun()?.filters.measure).toBe('total_comm');
      expect(boxText()).toBe(measureWords('total_comm'));
    }, 60_000);

    it('is remembered for the Owner: a kept choice comes back, and a new one is kept', async () => {
      const kept = JSON.stringify({ measure: 'qty' });
      vi.mocked(screenApi.prefsScreenGet).mockImplementation(({ screen: name }) =>
        Promise.resolve({
          screen: name,
          value:
            name === 'analytics.filters' ? kept : name === 'app.guide' ? '{"done":true}' : null,
        }),
      );
      vi.mocked(screenApi.prefsScreenSet).mockImplementation(({ screen: name, value }) =>
        Promise.resolve({ screen: name, value: value === '' ? null : value }),
      );
      const { user } = await openView('D');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('50');
      }, SLOW_MS);
      expect(lastRun()?.filters.measure).toBe('qty');
      expect(boxText()).toBe(measureWords('qty'));
      await choose(user, 'laga');
      await waitFor(() => {
        const keptNow = vi
          .mocked(screenApi.prefsScreenSet)
          .mock.calls.filter(([input]) => input.screen === 'analytics.filters')
          .at(-1)?.[0].value;
        expect(keptNow).toContain('"measure":"laga"');
      }, SLOW_MS);
    }, 60_000);
  });

  describe('the print', () => {
    it('asks for the measure and names it: the row total says what its cells hold', async () => {
      const { user } = await openView('D', { print: printDataD('qty') });
      await choose(user, 'qty');
      await waitFor(() => {
        expect(cell('0', 'p_2026_10')).toBe('50');
      }, SLOW_MS);
      await user.click(
        screen.getByRole('button', {
          name: (name) => visible(name).startsWith(m('report-preview')),
        }),
      );
      await screen.findByRole('heading', { level: 2, name: m('report-print-title') }, SLOW_MS);
      await waitFor(() => {
        expect(analyticsApi.analyticsPrintData).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(analyticsApi.analyticsPrintData).mock.calls.at(-1)?.[0]).toMatchObject({
        view: 'D',
        language,
        filters: { measure: 'qty' },
      });
      await waitFor(() => {
        const heads = Array.from(document.querySelectorAll('.print-report thead th')).map((th) =>
          visible(th.textContent),
        );
        expect(heads).toContain(m('analytics-col-row-total-qty'));
      }, SLOW_MS);
      // The picture in the print is titled with the measure too.
      const title = m('analytics-chart-d', { measure: 'qty' });
      const figures = Array.from(document.querySelectorAll('.print-chart')).map((figure) =>
        visible(figure.getAttribute('aria-label')),
      );
      expect(figures).toContain(title);
      expect(within(document.body).queryByText(m('analytics-col-row-total'))).toBeNull();
    }, 60_000);
  });
});
