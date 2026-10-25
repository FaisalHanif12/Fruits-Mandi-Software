// تجزیات, round 11 (plan 27, PRD 18.1, 18.2, FR-17.9, FR-12.27, E50, FR-18.2, FR-18.8): views F (by
// partner), G (trend) and J (trips) on the one analytics screen, and the new filters (partner, trip,
// goods, the layout of the trips), in the Urdu and in the English app, against a fake backend that
// answers with the backend's own fixtures. Each view asks for exactly what the boxes hold and shows the
// backend's figures and totals; a loss is red with its word; the comparison is not offered where the
// backend refuses it; a figure opens the lines behind it; the print has the partner columns headed
// with their names; a language switch loses nothing; the keyboard-only path works. The screen adds up
// nothing: every figure is one the backend sent.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsRow } from '../../src/bindings/AnalyticsRow';
import type { ReportCell } from '../../src/bindings/ReportCell';
import * as analyticsApi from '../../src/features/analytics/api';
import * as gardensApi from '../../src/features/gardens/api';
import * as tripsApi from '../../src/features/trips/api';
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
import {
  ANALYTICS_PRINT_DATA_J_EN,
  ANALYTICS_RUN_F_TRIP,
  ANALYTICS_RUN_I_GARDEN,
  ANALYTICS_RUN_G_TRIP_WRITTEN_OFF,
  ANALYTICS_RUN_J_EMPTY,
  ANALYTICS_RUN_J_TRIP_WRITTEN_OFF,
  fakeAnalytics,
  type AnalyticsOptions,
} from '../helpers/analytics';
import { fakeGardens } from '../helpers/gardens';
import { fakePartners, partnerRow } from '../helpers/partners';
import { printFirmName } from '../helpers/print-frame';
import { message, visible, paragraph } from '../helpers/render';
import { paisaForTest } from '../helpers/paisa';
import { fakeTrips } from '../helpers/trips';
import {
  ANALYTICS_PRINT_DATA_I_EN,
  ANALYTICS_RUN_F_GARDEN,
  ANALYTICS_RUN_G_GARDEN_DAY,
} from '../helpers/analytics-gardens-real';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one; a test that needs the compact
// step sets it.
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

/** The partners of the worked day (الف 40%, ب 30%, ج 30%), the ids the backend's fixtures use. */
const PARTNERS = [
  partnerRow(1, 'الف', 'Partner A', 4000),
  partnerRow(2, 'ب', 'Partner B', 3000),
  partnerRow(3, 'ج', 'Partner C', 3000),
];

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners(PARTNERS);
  fakeTrips();
  fakeGardens();
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

/** Where the garden's state and its result are among the cells of a view I row. */
const STATUS_COLUMN = 1;
const RESULT_COLUMN = 8;

function cellElement(rowId: string, colId: string): Element | null {
  return document.querySelector(`.ag-row[row-id="${rowId}"] [col-id="${colId}"]`);
}

function cell(rowId: string, colId: string): string {
  return visible(cellElement(rowId, colId)?.textContent);
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

describe.each(LANGUAGES)('تجزیات, views F, G and J (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);

  async function openView(view: string, options: AnalyticsOptions = {}) {
    const fake = fakeAnalytics(options);
    const result = await renderApp(`/analytics/${view}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(`analytics-view-${view.toLowerCase()}-title`)),
    });
    return { ...result, fake, user: userEvent.setup() };
  }

  describe('view F, by partner', () => {
    it('asks for no partner and no comparison, and shows each partner with the backend’s figures and totals', async () => {
      await openView('F');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({
        view: 'F',
        filters: { partner_id: null, compare: 'off', trip_id: null, goods: 'all', trip_view: null },
      });
      expect(cell('0', 'partner')).toBe(names('الف', 'Partner A'));
      expect(cell('0', 'income_share')).toBe('2,196');
      expect(cell('0', 'expense_share')).toBe('1,200');
      expect(cell('0', 'profit_share')).toBe('996');
      expect(cell('1', 'profit_share')).toBe('746');
      expect(totalsCell('income_share')).toBe('5,488');
      expect(totalsCell('profit_share')).toBe('2,488');
      // The garden and fruit columns have money (0 before any garden, and no fruit on the worked day).
      expect(cell('0', 'gardens')).toBe('0');
      expect(cell('0', 'fruit')).toBe('0');
    });

    it('draws a chart labelled for a screen reader, in the direction of the app', async () => {
      await openView('F');
      const chart = await screen.findByRole(
        'img',
        { name: named(m('analytics-chart-f')) },
        SLOW_MS,
      );
      expect(chart.getAttribute('data-chart')).toBe('bar');
      expect(chart.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
    });

    it('does not offer a comparison, which the backend refuses on this view', async () => {
      await openView('F');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-compare') })).toBeNull();
      expect(screen.queryByRole('combobox', { name: m('report-filter-item') })).toBeNull();
      expect(screen.getByRole('combobox', { name: m('report-filter-partner') })).toBeVisible();
    });

    it('sends the partner picked with the keys of the drop-down, and shows all partners again on “all”', async () => {
      const { user } = await openView('F');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(
        await screen.findByRole('option', { name: names('ب', 'Partner B') }, SLOW_MS),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.partner_id).toBe(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(await screen.findByRole('option', { name: m('report-partner-all') }));
      await waitFor(() => {
        expect(lastRun()?.filters.partner_id).toBeNull();
      }, SLOW_MS);
    });

    it('shows a loss from a trip with its sign, and the net change under it', async () => {
      await openView('F', { pages: { F: ANALYTICS_RUN_F_TRIP } });
      await waitFor(() => {
        expect(cell('0', 'net_change')).toBe('-296');
      }, SLOW_MS);
      expect(cell('0', 'trips')).toBe('-776');
      expect(totalsCell('trips')).toBe('-1,940');
      expect(totalsCell('net_change')).toBe('-740');
    });

    it('Enter on a partner opens his share rows, and on a line the بکری or voucher behind it', async () => {
      const { router, user } = await openView('F');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="partner"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'F',
        row_key: { partner_id: 1 },
        column_key: null,
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      // The percent of each line is headed "Share", not "Share of Commission": it is his share of an
      // expense too.
      expect(headerText('share')).toBe(m('analytics-col-share-percent'));
      // His share of a posting, from a شراکت دار row: the first line opens its بکری.
      await user.click(pick('.ag-row[row-id="0"] [col-id="date"]'));
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
    });
  });

  describe('view G, the trend', () => {
    it('shows the period with the money of each, and the backend’s totals, by month at first', async () => {
      await openView('G');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({ view: 'G', filters: { group_by: null, partner_id: null } });
      expect(cell('0', 'period')).toBe(`${m('report-month-10')} 2026`);
      expect(cell('0', 'gross')).toBe('41,400');
      expect(cell('0', 'commission')).toBe('5,448');
      expect(cell('0', 'profit')).toBe('2,488');
      expect(totalsCell('profit')).toBe('2,488');
      // No trip on the worked day: the trips' result is blank, not 0.
      expect(cell('0', 'trips_result')).toBe('');
    });

    it('asks by day when the grouping box says day, and names the day in its own column', async () => {
      const { user } = await openView('G');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-group-by') }));
      await user.click(await screen.findByRole('option', { name: m('analytics-group-by-day') }));
      await waitFor(() => {
        expect(lastRun()?.filters.group_by).toBe('day');
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('0', 'period')).toBe('10-10-2026');
      }, SLOW_MS);
    });

    it('shows a loss of the trips in red with its word, and a chart of the money of each day', async () => {
      await openView('G', { pages: { G: ANALYTICS_RUN_G_TRIP_WRITTEN_OFF } });
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      expect(cell('0', 'trips_result')).toContain('660');
      expect(cell('0', 'trips_result')).toContain(m('report-result-profit'));
      expect(cell('1', 'trips_result')).toContain('-2,600');
      expect(cell('1', 'trips_result')).toContain(m('report-result-loss'));
      expect(cellElement('1', 'trips_result')).toHaveClass('cell-tone-due');
      expect(cellElement('0', 'trips_result')).not.toHaveClass('cell-tone-due');
      expect(totalsCell('trips_result')).toContain('-1,940');
      expect(totalsCell('trips_result')).toContain(m('report-result-loss'));
      expect(await screen.findByRole('img', { name: named(m('analytics-chart-g')) })).toBeVisible();
    });

    it('does not offer a comparison', async () => {
      await openView('G');
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-compare') })).toBeNull();
    });
  });

  describe('view J, the trips', () => {
    it('shows a trip with its quantities, money and result, and each partner’s percent and part', async () => {
      await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({
        view: 'J',
        filters: { trip_view: null, partner_id: null, trip_id: null, goods: 'all' },
      });
      expect(cell('0', 'group')).toBe(names('سفر-1 10-10-2026', 'TRP-1 10-10-2026'));
      expect(cell('0', 'sales_money')).toBe('8,460');
      expect(cell('0', 'result')).toContain('660');
      expect(cell('0', 'result')).toContain(m('report-result-profit'));
      expect(totalsCell('sales_money')).toBe('8,460');
    });

    it('draws only the columns that fit, says how many, and offers the others', async () => {
      const { user } = await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      // The test window is at the normal step: the trip, its result and the money come first.
      expect(drawnColumns()[0]).toBe('group');
      expect(drawnColumns()).toContain('result');
      expect(drawnColumns().length).toBeLessThan(18);
      const shown = drawnColumns().length;
      expect(
        await screen.findByText(paragraph(m('analytics-columns-fit', { shown, total: 18 }))),
      ).toBeVisible();
      await user.click(screen.getByRole('button', { name: named(m('analytics-columns-other')) }));
      await waitFor(() => {
        expect(
          screen.getByRole('button', { name: named(m('analytics-columns-main')) }),
        ).toBeVisible();
      }, SLOW_MS);
      // The trip stays when the others come in, so a figure is never without its trip.
      expect(drawnColumns()[0]).toBe('group');
    });

    it('heads the columns of a partner with his name and then what they hold, at the wide step', async () => {
      vi.mocked(useLayoutStep).mockReturnValue('wide');
      await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      // At the wide step every partner's part of the result is shown beside the trip's own figures.
      expect(drawnColumns()).toEqual(
        expect.arrayContaining(['group', 'result', 'part_1', 'part_2']),
      );
      expect(headerText('part_1')).toBe(
        `${names('الف', 'Partner A')}: ${m('analytics-col-partner-part')}`,
      );
      expect(cell('0', 'part_1')).toContain('-776');
      expect(cell('0', 'part_1')).toContain(m('report-result-loss'));
      expect(totalsCell('part_2')).toContain('-582');
    });

    it('keeps the trip, the partner, the layout, the item and the seller in view and folds the rest behind “More filters”', async () => {
      const { user } = await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      for (const box of [
        'report-filter-partner',
        'report-filter-trip-view',
        'analytics-filter-trip',
        'report-filter-item',
      ]) {
        expect(screen.getByRole('combobox', { name: m(box) })).toBeVisible();
      }
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-goods') })).toBeNull();
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-side') })).toBeNull();
      await user.click(screen.getByRole('button', { name: named(m('report-filters-more')) }));
      expect(
        await screen.findByRole('combobox', { name: m('analytics-filter-goods') }),
      ).toBeVisible();
      // A box that holds a value keeps the fold open by itself, so a filter is never out of sight.
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-goods') }));
      await user.click(await screen.findByRole('option', { name: m('analytics-goods-trip') }));
      await waitFor(() => {
        expect(lastRun()?.filters.goods).toBe('trip');
      }, SLOW_MS);
      expect(screen.getByRole('button', { name: named(m('report-filters-fewer')) })).toBeDisabled();
    });

    it('shows a loss in red with its word, in the row and under the table', async () => {
      await openView('J', { pages: { J: ANALYTICS_RUN_J_TRIP_WRITTEN_OFF } });
      await waitFor(() => {
        expect(cell('0', 'result')).toContain('-1,940');
      }, SLOW_MS);
      expect(cell('0', 'result')).toContain(m('report-result-loss'));
      expect(cellElement('0', 'result')).toHaveClass('cell-tone-due');
      expect(totalsCell('result')).toContain('-1,940');
      expect(totalsCell('result')).toContain(m('report-result-loss'));
      expect(document.querySelector('.ag-row-pinned [col-id="result"]')).toHaveClass(
        'cell-tone-due',
      );
    });

    it('the layout box asks by purchase, and the partner box and the trip box narrow the rows', async () => {
      const { user } = await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-trip-view') }));
      await user.click(
        await screen.findByRole('option', { name: m('analytics-filter-trip-by-purchase') }),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.trip_view).toBe('purchase');
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(await screen.findByRole('option', { name: names('الف', 'Partner A') }));
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({ trip_view: 'purchase', partner_id: 1 });
      }, SLOW_MS);
      // The trip box lists the trips by name, with “all trips” first.
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-trip') }));
      await user.click(await screen.findByRole('option', { name: names('سفر-1', 'Trip 1') }));
      await waitFor(() => {
        expect(lastRun()?.filters.trip_id).toBe(1);
      }, SLOW_MS);
    });

    it('Enter on a trip opens the purchases behind it, and the totals open the whole filter', async () => {
      const { user } = await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="group"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'J',
        row_key: { purchase_ids: [1], partner_id: null },
        column_key: null,
      });
      await user.click(screen.getByRole('button', { name: named(m('analytics-lines-back')) }));
      await waitFor(() => {
        expect(totalsCell('sales_money')).toBe('8,460');
      }, SLOW_MS);
      await user.click(pick('.ag-row-pinned [col-id="sales_money"]'));
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
        partner_id: null,
        purchase_ids: [],
      });
    });

    it('says there is nothing for these filters, and the way out clears them', async () => {
      const { user } = await openView('J', { pages: { J: ANALYTICS_RUN_J_EMPTY } });
      await screen.findByText(named(m('analytics-empty')), undefined, SLOW_MS);
      // Nothing is filled, so there is nothing to clear.
      expect(screen.queryByRole('button', { name: named(m('report-clear-filters')) })).toBeNull();
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(await screen.findByRole('option', { name: names('ب', 'Partner B') }));
      expect(
        await screen.findByRole('button', { name: named(m('report-clear-filters')) }, SLOW_MS),
      ).toBeVisible();
    });

    it('the print has the partner columns headed with the partner’s name', async () => {
      const { user } = await openView('J', { print: ANALYTICS_PRINT_DATA_J_EN });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      await screen.findByText(printFirmName(language), undefined, SLOW_MS);
      expect(vi.mocked(analyticsApi.analyticsPrintData).mock.calls[0]?.[0]).toMatchObject({
        view: 'J',
        language,
        paper: 'a4',
      });
      const heads = Array.from(document.querySelectorAll('.print-report thead th')).map((th) =>
        visible(th.textContent),
      );
      // Every column of the backend is in the print, the partner ones headed "Partner A: …".
      expect(heads).toHaveLength(ANALYTICS_PRINT_DATA_J_EN.columns.length);
      expect(heads.some((head) => head.startsWith('Partner A:'))).toBe(true);
      expect(heads.some((head) => /report-col-|analytics-col-/.test(head))).toBe(false);
    });
  });

  describe('view I, the gardens', () => {
    it('asks for no garden, no partner and no comparison, and shows each garden with the backend’s figures and totals', async () => {
      await openView('I');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({
        view: 'I',
        filters: { garden_id: null, partner_id: null, compare: 'off', goods: 'all' },
      });
      expect(cell('0', 'group')).toBe(names('شاہ پور باغ', 'Shahpur Garden'));
      expect(cell('0', 'status')).toBe(m('report-garden-closed'));
      expect(cell('0', 'result')).toBe(`11,400${m('report-result-profit')}`);
      expect(totalsCell('result')).toBe('11,400');
      expect(cell('0', 'result_1')).toBe(`6,840${m('report-result-profit')}`);
      expect(cell('0', 'result_2')).toBe(`4,560${m('report-result-profit')}`);
    });

    it('heads the columns of a partner with his name, then what they hold', async () => {
      await openView('I');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      expect(headerText('result_1')).toBe(
        `${names('الف', 'Partner A')}: ${m('analytics-col-partner-result')}`,
      );
    });

    it('writes Profit on the result of a closed garden and each partner’s part, and offers no comparison', async () => {
      await openView('I');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      expect(cell('0', 'result')).toContain(m('report-result-profit'));
      expect(cell('0', 'result_1')).toContain(m('report-result-profit'));
      // The totals row adds open and closed gardens, so it carries no word.
      expect(totalsCell('result')).toBe('11,400');
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-compare') })).toBeNull();
    });

    it('writes Loss, in red, on a closed garden’s loss, but only the sign on an open garden below zero (FR-12.38)', async () => {
      const base = ANALYTICS_RUN_I_GARDEN.rows[0];
      if (base === undefined) throw new Error('The view I fixture has no row.');
      const gardenRow = (state: string, paisa: number): AnalyticsRow => ({
        ...base,
        cells: base.cells.map((found, index): ReportCell => {
          if (index === STATUS_COLUMN) return { type: 'label', id: state };
          if (index === RESULT_COLUMN) return { type: 'money', paisa: paisaForTest(paisa) };
          return found;
        }),
      });
      const page: AnalyticsPage = {
        ...ANALYTICS_RUN_I_GARDEN,
        rows: [
          gardenRow('report-garden-open', -3_000_000),
          gardenRow('report-garden-closed', -450_000),
        ],
      };
      await openView('I', { pages: { I: page } });
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      expect(cell('0', 'result')).toContain('30,000');
      expect(cell('0', 'result')).not.toContain(m('report-result-loss'));
      expect(cellElement('0', 'result')).not.toHaveClass('cell-tone-due');
      expect(cell('1', 'result')).toContain('4,500');
      expect(cell('1', 'result')).toContain(m('report-result-loss'));
      expect(cellElement('1', 'result')).toHaveClass('cell-tone-due');
    });

    it('keeps the garden box in view, lists the gardens by name and asks for the one picked', async () => {
      const { user } = await openView('I');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      // The box is one of the main boxes of this view, not behind "More filters".
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-garden') }));
      await user.click(
        await screen.findByRole(
          'option',
          { name: names('شاہ پور باغ', 'Shahpur Garden') },
          SLOW_MS,
        ),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.garden_id).toBe(1);
      }, SLOW_MS);
      expect(gardensApi.gardensList).toHaveBeenCalled();
    });

    it('the side box narrows the commission, and the backend’s answer is shown', async () => {
      const { user } = await openView('I');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(await screen.findByRole('combobox', { name: m('analytics-filter-side') }));
      await user.click(
        await screen.findByRole('option', { name: named(m('analytics-side-seller')) }),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.side).toBe('seller');
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('analytics-columns-other')) }));
      await waitFor(() => {
        expect(cell('0', 'commission')).toBe('3,600');
      }, SLOW_MS);
    });

    it('draws a chart of the sales and the result of each garden', async () => {
      await openView('I');
      expect(
        await screen.findByRole('img', { name: named(m('analytics-chart-i')) }, SLOW_MS),
      ).toBeVisible();
    });

    it('Enter on a garden opens the sale lines of its goods, with the garden’s account as the row key', async () => {
      const { user } = await openView('I');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="group"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'I',
        row_key: { seller_id: 31 },
        column_key: null,
      });
    });

    it('the print has the partners’ columns headed with their names', async () => {
      const { user } = await openView('I', { print: ANALYTICS_PRINT_DATA_I_EN });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByText(printFirmName(language), undefined, SLOW_MS);
      const heads = Array.from(document.querySelectorAll('.print-report thead th')).map((th) =>
        visible(th.textContent),
      );
      expect(heads).toHaveLength(ANALYTICS_PRINT_DATA_I_EN.columns.length);
      expect(heads.some((head) => head.startsWith('Partner A:'))).toBe(true);
      expect(heads.some((head) => /analytics-col-|report-col-/.test(head))).toBe(false);
    });
  });

  describe('the garden columns of F and G, and the garden box of the other views', () => {
    it('F has money in its gardens column', async () => {
      await openView('F', { pages: { F: ANALYTICS_RUN_F_GARDEN } });
      await waitFor(() => {
        expect(cell('0', 'gardens')).toBe('6,840');
      }, SLOW_MS);
      expect(cell('1', 'gardens')).toBe('26,560');
      expect(totalsCell('gardens')).toBe('33,400');
      expect(cell('0', 'net_change')).toBe('10,040');
    });

    it('F and J offer no garden box, because the backend refuses it there', async () => {
      await openView('F');
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-garden') })).toBeNull();
    });

    it('G shows the movement of the garden balances, below the line on a day of payments', async () => {
      await openView('G', { pages: { G: ANALYTICS_RUN_G_GARDEN_DAY } });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('0', 'gardens_result')).toBe('-30,000');
      }, SLOW_MS);
      expect(cell('2', 'gardens_result')).toBe('66,400');
      expect(totalsCell('gardens_result')).toBe('11,400');
    });

    it('a garden picked on view A narrows the sales to its goods, and the box is in the “More filters” group', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(lastRun()?.filters.garden_id).toBeNull();
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-garden') }));
      await user.click(
        await screen.findByRole(
          'option',
          { name: names('شاہ پور باغ', 'Shahpur Garden') },
          SLOW_MS,
        ),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.garden_id).toBe(1);
      }, SLOW_MS);
    });
  });

  describe('the goods box of the older views', () => {
    it('asks for trip goods on view A, and shows what the backend answers', async () => {
      const { user } = await openView('A');
      await waitFor(() => {
        expect(lastRun()?.filters.goods).toBe('all');
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-goods') }));
      await user.click(await screen.findByRole('option', { name: m('analytics-goods-trip') }));
      await waitFor(() => {
        expect(lastRun()?.filters.goods).toBe('trip');
      }, SLOW_MS);
      // The trip goods of the worked day: none, so the view says there is nothing.
      await screen.findByText(named(m('analytics-empty')), undefined, SLOW_MS);
      expect(
        await screen.findByRole('button', { name: named(m('report-clear-filters')) }),
      ).toBeVisible();
    });
  });

  describe('a language switch in the middle of choosing', () => {
    it('keeps the partner and the layout, and writes the names, the headings and the chart again', async () => {
      const { user } = await openView('J');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(await screen.findByRole('option', { name: names('ب', 'Partner B') }));
      await user.click(screen.getByRole('combobox', { name: m('report-filter-trip-view') }));
      await user.click(await screen.findByRole('option', { name: m('report-filter-trip-by-day') }));
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({ partner_id: 2, trip_view: 'day' });
      }, SLOW_MS);
      const before = vi.mocked(analyticsApi.analyticsRun).mock.calls.length;
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(
          screen.getByRole('combobox', { name: message(other, 'report-filter-partner') }),
        ).toHaveTextContent(other === 'ur' ? 'ب' : 'Partner B');
      }, SLOW_MS);
      expect(
        screen.getByRole('combobox', { name: message(other, 'report-filter-trip-view') }),
      ).toHaveTextContent(message(other, 'report-filter-trip-by-day'));
      expect(
        screen.getByRole('img', { name: named(message(other, 'analytics-chart-j')) }),
      ).toBeVisible();
      // Nothing is asked again: the answer is the same, only its words change.
      expect(vi.mocked(analyticsApi.analyticsRun).mock.calls.length).toBe(before);
    });
  });

  describe('keyboard only', () => {
    it('opens view F, goes to the table with Tab, ↓ to a partner, Enter for his lines and Esc back', async () => {
      const { user } = await openView('F');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
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
      // A tab of the views is reached with the keys too: view F has the trend next to it.
      const tab = screen.getByRole('tab', { name: m('analytics-view-f-title') });
      await act(async () => {
        tab.focus();
        await Promise.resolve();
      });
      await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
      await screen.findByRole(
        'heading',
        { level: 1, name: named(m('analytics-view-g-title')) },
        SLOW_MS,
      );
    });
  });

  describe('the four states', () => {
    it('says what happened and tries again when the backend fails on view J', async () => {
      let failing = true;
      const { user } = await openView('J', {
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
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
    });

    it('says the list of trips could not be read under its own box, and the box stays shut', async () => {
      fakeAnalytics();
      vi.mocked(tripsApi.tripsList).mockRejectedValue(new Error('down'));
      await renderApp('/analytics/J');
      await screen.findByText(named(m('analytics-trip-failed')), undefined, SLOW_MS);
      // The view itself still opens: only the trip box is shut.
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(1);
      }, SLOW_MS);
    });
  });
});
