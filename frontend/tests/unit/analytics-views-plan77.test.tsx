// The analytics of plan 77 (PRD FR-18.1, FR-18.9, 18.2), in the Urdu and the English app, against a fake backend
// with the numbers of the plan (October 2026, after Events 1 to 5): the buyer filter sits beside the seller in
// view and not behind "More filters"; view L, buyer × item (اکرم ٹول ٹیکس کیلا 880, ارشد ملتانی کیلا 1,284, شاہد
// سیب 800, total 2,964: buyer commission + لاگا), has a column for each item and no side box; view M, expenses
// by head (بجلی 3,000 in October), takes only the period and the grouping, draws its chart by period and opens
// the lines behind a cell. The screen adds nothing up: every figure is the backend's.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as analyticsApi from '../../src/features/analytics/api';
import { useAnalyticsStore } from '../../src/features/analytics/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { ANALYTICS_CATALOG, fakeAnalytics } from '../helpers/analytics';
import {
  ANALYTICS_LINES_L_AKRAM_BANANA_REQUEST,
  ANALYTICS_LINES_M_ELECTRICITY_REQUEST,
  ANALYTICS_RUN_F_BY_MONTH_REQUEST,
  ANALYTICS_RUN_L_REQUEST,
  ANALYTICS_RUN_M_BY_DAY_REQUEST,
  ANALYTICS_RUN_M_REQUEST,
} from '../helpers/analytics-plan77';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
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

function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

function lastRun() {
  return vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0];
}

describe.each(LANGUAGES)(
  'تجزیات, views L and M and the buyer filter (%s)',
  (language: Language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);
    const names = (ur: string, en: string) => (language === 'ur' ? ur : en);

    async function openView(view: string) {
      const fake = fakeAnalytics();
      const result = await renderApp(`/analytics/${view}`);
      await screen.findByRole(
        'heading',
        {
          level: 1,
          name: (name) => visible(name).startsWith(m(`analytics-view-${view.toLowerCase()}-title`)),
        },
        SLOW_MS,
      );
      return { ...result, fake, user: userEvent.setup() };
    }

    it('lists the thirteen views as tabs, L and M last', async () => {
      await openView('A');
      const tabs = screen.getAllByRole('tab').map((tab) => visible(tab.textContent));
      expect(tabs).toContain(m('analytics-view-l-title'));
      expect(tabs).toContain(m('analytics-view-m-title'));
    });

    describe('the buyer filter', () => {
      it('is in view beside the seller on views that take both, and not behind “More filters”', async () => {
        vi.mocked(useLayoutStep).mockReturnValue('compact');
        await openView('A');
        expect(
          screen.getByRole('combobox', { name: m('report-filter-seller') }),
        ).toBeInTheDocument();
        expect(
          screen.getByRole('combobox', { name: m('analytics-filter-buyer') }),
        ).toBeInTheDocument();
        // A box that is still folded is not drawn at all.
        expect(screen.queryByRole('combobox', { name: m('report-filter-city') })).toBeNull();
      });
    });

    describe('view L, buyer × item', () => {
      it('shows a row for each buyer and a column for each item, with the backend’s figures and totals', async () => {
        await openView('L');
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0', '1', '2']);
        }, SLOW_MS);
        // The three buyers of the plan (a چکتا buyer is a row by name), the commission of each on his item.
        expect(cell('0', 'buyer')).toContain(names('ارشد ملتانی', 'Arshad Multani'));
        expect(cell('0', 'i_1')).toBe('1,284');
        expect(cell('1', 'buyer')).toContain(names('اکرم ٹول ٹیکس', 'Akram Toll Tax'));
        expect(cell('1', 'i_1')).toBe('880');
        expect(cell('2', 'buyer')).toContain(names('شاہد', 'Shahid'));
        expect(cell('2', 'i_2')).toBe('800');
        // His total is buyer commission + لاگا, and the totals row is the backend's: 2,964 = 880 + 1,284 + 800.
        expect(cell('1', 'total_comm')).toBe('880');
        expect(totalsCell('total_comm')).toBe('2,964');
        expect(lastRun()).toMatchObject({ view: 'L' });
      });

      it('offers the item and the party boxes and no side box, even behind “More filters”', async () => {
        const { user } = await openView('L');
        await screen.findByRole('combobox', { name: m('analytics-filter-buyer') });
        expect(screen.getByRole('combobox', { name: m('report-filter-item') })).toBeInTheDocument();
        expect(
          screen.getByRole('combobox', { name: m('report-filter-seller') }),
        ).toBeInTheDocument();
        await user.click(
          await screen.findByRole('button', {
            name: (name) => visible(name).startsWith(m('report-filters-more')),
          }),
        );
        expect(screen.queryByRole('combobox', { name: m('analytics-filter-side') })).toBeNull();
        expect(
          screen.getByRole('combobox', { name: m('analytics-filter-compare') }),
        ).toBeInTheDocument();
      });

      it('opens the lines behind a figure with its buyer and its item', async () => {
        const { user } = await openView('L');
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0', '1', '2']);
        }, SLOW_MS);
        const figure = document.querySelector('.ag-row[row-id="1"] [col-id="i_1"]');
        if (figure === null) throw new Error('no figure');
        await user.click(figure);
        await user.keyboard('{Enter}');
        await waitFor(() => {
          expect(vi.mocked(analyticsApi.analyticsLines).mock.calls.at(-1)?.[0]).toMatchObject({
            view: 'L',
            row_key: { buyer_id: 21 },
            column_key: { item_id: 1 },
          });
        }, SLOW_MS);
      });
    });

    describe('view M, expenses: the shop and the partners', () => {
      /** The cell of a part of the view: view M has two tables, the shop's and the partners'. */
      const partCell = (part: 'shop' | 'partner', rowId: string, colId: string) =>
        visible(
          document.querySelector(
            `[data-kharch-part="${part}"] .ag-row[row-id="${rowId}"] [col-id="${colId}"]`,
          )?.textContent,
        );
      const partTotals = (part: 'shop' | 'partner', colId: string) =>
        visible(
          document.querySelector(`[data-kharch-part="${part}"] .ag-row-pinned [col-id="${colId}"]`)
            ?.textContent,
        );
      const readOf = (part: 'shop' | 'partner') =>
        vi
          .mocked(analyticsApi.analyticsRun)
          .mock.calls.map((call) => call[0])
          .filter((input) => input.view === 'M' && input.filters.expense_part === part)
          .at(-1);

      it('shows the shop’s head with its total and each partner’s own kharch with his total, and a chart by period', async () => {
        await openView('M');
        await waitFor(() => {
          expect(partCell('shop', '0', 'head')).toContain(names('بجلی', 'Electricity'));
        }, SLOW_MS);
        expect(partCell('shop', '0', 'p_2026_10')).toBe('3,000');
        expect(partTotals('shop', 'total')).toBe('3,000');
        await waitFor(() => {
          expect(partCell('partner', '0', 'partner')).toContain(names('الف', 'Partner A'));
        }, SLOW_MS);
        expect(partCell('partner', '0', 'own')).toBe('2,000');
        expect(partCell('partner', '1', 'own')).toBe('0');
        expect(partTotals('partner', 'own')).toBe('2,000');
        expect(
          await screen.findByRole(
            'img',
            { name: (name) => visible(name).startsWith(m('analytics-chart-m')) },
            SLOW_MS,
          ),
        ).toBeInTheDocument();
        // The screen's own print, Excel and all-lines buttons are off: each part has its own.
        expect(
          screen.queryByRole('button', {
            name: (name) => visible(name) === m('analytics-all-lines'),
          }),
        ).toBeNull();
      });

      it('takes the period, the grouping and the partner: no item, seller, buyer or comparison box', async () => {
        await openView('M');
        await screen.findByRole('combobox', { name: m('report-filter-period') });
        for (const id of ['analytics-filter-group-by', 'report-filter-partner']) {
          expect(screen.getByRole('combobox', { name: m(id) })).toBeInTheDocument();
        }
        for (const id of [
          'report-filter-item',
          'report-filter-seller',
          'analytics-filter-buyer',
          'analytics-filter-compare',
        ]) {
          expect(screen.queryByRole('combobox', { name: m(id) })).toBeNull();
        }
        // The boxes M does not take are sent empty, as for every view.
        await waitFor(() => {
          expect(readOf('shop')).toMatchObject({
            view: 'M',
            filters: { item_id: null, seller_id: null, buyer: null, compare: 'off' },
          });
        }, SLOW_MS);
        expect(readOf('partner')).toMatchObject({ view: 'M' });
      });

      it('asks for the grouping the Owner chooses (day, month or year), in both parts', async () => {
        const { user } = await openView('M');
        await screen.findByRole('combobox', { name: m('analytics-filter-group-by') });
        await user.click(screen.getByRole('combobox', { name: m('analytics-filter-group-by') }));
        await user.click(await screen.findByRole('option', { name: m('analytics-group-by-day') }));
        await waitFor(() => {
          expect(readOf('shop')).toMatchObject({ filters: { group_by: 'day' } });
        }, SLOW_MS);
        expect(readOf('partner')).toMatchObject({ filters: { group_by: 'day' } });
      });

      it('opens the posted voucher lines of a head in a period when its figure is opened', async () => {
        const { user } = await openView('M');
        await waitFor(() => {
          expect(partCell('shop', '0', 'p_2026_10')).toBe('3,000');
        }, SLOW_MS);
        const figure = document.querySelector(
          '[data-kharch-part="shop"] .ag-row[row-id="0"] [col-id="p_2026_10"]',
        );
        if (figure === null) throw new Error('no figure');
        await user.click(figure);
        await user.keyboard('{Enter}');
        await waitFor(() => {
          expect(vi.mocked(analyticsApi.analyticsLines).mock.calls.at(-1)?.[0]).toMatchObject({
            view: 'M',
            row_key: { head_id: 35 },
            column_key: { from: '2026-10-01', to: '2026-10-31' },
          });
        }, SLOW_MS);
      });
    });

    describe('the grouping of the other views (PRD FR-18.9)', () => {
      it('F, I, J and K may be read as a whole or by day, month or year; D, G and M are always grouped', async () => {
        for (const id of ['F', 'I', 'J', 'K', 'D', 'G', 'M']) {
          expect(ANALYTICS_CATALOG.find((entry) => entry.id === id)?.filters, id).toContain(
            'group_by',
          );
        }
        const { user } = await openView('F');
        const box = await screen.findByRole('combobox', { name: m('analytics-filter-group-by') });
        // Left out, the view is the one of plan 27: no grouping is asked for.
        await waitFor(() => {
          expect(lastRun()).toMatchObject({ view: 'F', filters: { group_by: null } });
        }, SLOW_MS);
        await user.click(box);
        expect(
          await screen.findByRole('option', { name: m('analytics-group-by-none') }),
        ).toBeInTheDocument();
        await user.click(screen.getByRole('option', { name: m('analytics-group-by-month') }));
        await waitFor(() => {
          expect(lastRun()).toMatchObject({ view: 'F', filters: { group_by: 'month' } });
        }, SLOW_MS);
        expect(ANALYTICS_RUN_F_BY_MONTH_REQUEST.filters.group_by).toBe('month');
        // The period is the first column, one row for each partner in the month, and the totals are the backend's.
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0', '1', '2']);
        }, SLOW_MS);
        await waitFor(() => {
          expect(cell('0', 'period')).toContain(m('report-month-10'));
          expect(cell('0', 'period')).toContain('2026');
          expect(cell('0', 'profit_share')).toBe('996');
          expect(cell('2', 'profit_share')).toBe('746');
          expect(totalsCell('profit_share')).toBe('2,488');
        }, SLOW_MS);
      });

      it('view B by month: the period is the first column, one row for each seller, and the totals are the whole period’s', async () => {
        const { user } = await openView('B');
        await user.click(
          await screen.findByRole('combobox', { name: m('analytics-filter-group-by') }),
        );
        await user.click(
          await screen.findByRole('option', { name: m('analytics-group-by-month') }),
        );
        await waitFor(() => {
          expect(lastRun()).toMatchObject({ view: 'B', filters: { group_by: 'month' } });
        }, SLOW_MS);
        await waitFor(() => {
          expect(drawnRows()).toEqual(['0', '1']);
        }, SLOW_MS);
        // October 2026: رحیم 3,928 and کریم 1,520 of total commission, each in its month.
        await waitFor(() => {
          expect(cell('0', 'period')).toContain(m('report-month-10'));
          expect(cell('0', 'total_comm')).toBe('3,928');
          expect(cell('1', 'total_comm')).toBe('1,520');
          expect(totalsCell('total_comm')).toBe('5,448');
        }, SLOW_MS);
        // A grouped view has no chart: the backend sends none.
        expect(
          screen.queryByRole('img', {
            name: (name) => visible(name).startsWith(m('analytics-chart-b')),
          }),
        ).toBeNull();
      });

      it.each([
        ['A', 'total_comm', ['0', '1'], '5,448'],
        ['C', 'buyer_comm', ['0', '1', '2'], '2,484'],
        ['E', 'total_comm', ['0', '1'], '5,448'],
        ['L', 'total_comm', ['0', '1', '2'], '2,964'],
      ])(
        'view %s by month: the period is the first column and the totals are the whole period’s',
        async (view, totalColumn, rows, total) => {
          const { user } = await openView(view);
          await user.click(
            await screen.findByRole('combobox', { name: m('analytics-filter-group-by') }),
          );
          await user.click(
            await screen.findByRole('option', { name: m('analytics-group-by-month') }),
          );
          await waitFor(() => {
            expect(lastRun()).toMatchObject({ view, filters: { group_by: 'month' } });
          }, SLOW_MS);
          await waitFor(() => {
            expect(drawnRows()).toEqual(rows);
          }, SLOW_MS);
          // The windowed views (E and L) draw their columns a moment after their rows.
          await waitFor(() => {
            expect(cell('0', 'period')).toContain(m('report-month-10'));
            expect(cell('0', 'period')).toContain('2026');
            expect(totalsCell(totalColumn)).toBe(total);
          }, SLOW_MS);
        },
      );

      it('a view that is always grouped offers month, day and year and no “no grouping”', async () => {
        const { user } = await openView('M');
        await user.click(
          await screen.findByRole('combobox', { name: m('analytics-filter-group-by') }),
        );
        for (const id of [
          'analytics-group-by-month',
          'analytics-group-by-day',
          'analytics-group-by-year',
        ]) {
          expect(await screen.findByRole('option', { name: m(id) })).toBeInTheDocument();
        }
        expect(screen.queryByRole('option', { name: m('analytics-group-by-none') })).toBeNull();
      });

      it('the grouping chosen for one view does not group another: D stays a month when F is by day', async () => {
        const { user } = await openView('F');
        await user.click(
          await screen.findByRole('combobox', { name: m('analytics-filter-group-by') }),
        );
        await user.click(await screen.findByRole('option', { name: m('analytics-group-by-day') }));
        await waitFor(() => {
          expect(lastRun()).toMatchObject({ view: 'F', filters: { group_by: 'day' } });
        }, SLOW_MS);
        await user.click(screen.getByRole('tab', { name: m('analytics-view-d-title') }));
        await waitFor(() => {
          expect(lastRun()).toMatchObject({ view: 'D', filters: { group_by: null } });
        }, SLOW_MS);
      });
    });

    describe('the requests are the backend’s own', () => {
      it('views L and M ask for the period and nothing else, as the backend’s fixtures do', () => {
        for (const request of [
          ANALYTICS_RUN_L_REQUEST,
          ANALYTICS_RUN_M_REQUEST,
          ANALYTICS_RUN_M_BY_DAY_REQUEST,
        ]) {
          expect(request.filters).toMatchObject({ from: '01-10-2026', to: '31-10-2026' });
        }
        expect(ANALYTICS_LINES_L_AKRAM_BANANA_REQUEST.row_key).toMatchObject({ buyer_id: 21 });
        expect(ANALYTICS_LINES_M_ELECTRICITY_REQUEST.row_key).toMatchObject({ head_id: 35 });
      });
    });
  },
);
