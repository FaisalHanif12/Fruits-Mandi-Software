// The fruit taken, view K of the analytics, and the fruit column of view F (round 13, PRD 18.2 K and F,
// 16.7, FR-18.8), in the Urdu and in the English app, against a fake backend that answers with the
// backend's own fixtures. View K is the fruit taken for each partner (or the shop's guests) and item:
// the quantity and the value, and where it came from (our shop, other shops). View F has the fruit of
// each partner in its fruit column, and its garden, trip and fruit numbers open the lines behind them.
// The screen adds up nothing: every figure is one the backend sent.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as analyticsApi from '../../src/features/analytics/api';
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
  ANALYTICS_CATALOG,
  ANALYTICS_LINES_F_FRUIT_ALIF,
  ANALYTICS_LINES_K_GUESTS,
  ANALYTICS_PRINT_DATA_K_EN,
  ANALYTICS_RUN_K_FRUIT,
  fakeAnalytics,
  type AnalyticsOptions,
} from '../helpers/analytics';
import { fakeGardens } from '../helpers/gardens';
import { fakePartners, partnerRow } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeTrips } from '../helpers/trips';

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
// The window of a test has no layout, so the step is the middle one; a test that needs another sets it.
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
  return vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0];
}

function lastLines() {
  return vi.mocked(analyticsApi.analyticsLines).mock.calls.at(-1)?.[0];
}

function runCount() {
  return vi.mocked(analyticsApi.analyticsRun).mock.calls.length;
}

describe.each(LANGUAGES)('تجزیات, view K and the fruit of view F (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openView(view: string, options: AnalyticsOptions = {}) {
    const fake = fakeAnalytics(options);
    const result = await renderApp(`/analytics/${view}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(`analytics-view-${view.toLowerCase()}-title`)),
    });
    return { ...result, fake, user: userEvent.setup() };
  }

  describe('view K, the fruit taken', () => {
    it('is the last tab, with the boxes of the PRD: the days, the item, the partner, the garden and the goods', () => {
      expect(ANALYTICS_CATALOG.at(-1)).toMatchObject({
        id: 'K',
        title_id: 'analytics-view-k-title',
        filters: ['period', 'item', 'partner', 'garden', 'goods'],
      });
    });

    it('asks for no partner and no comparison, and shows each partner and the guests with the backend’s figures and totals', async () => {
      await openView('K');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      expect(lastRun()).toMatchObject({
        view: 'K',
        filters: { partner_id: null, compare: 'off', goods: 'all', garden_id: null },
      });
      expect(cell('0', 'for')).toBe(names('الف', 'Partner A'));
      expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
      expect(cell('0', 'qty')).toBe('2');
      expect(cell('0', 'value')).toBe('1,160');
      expect(cell('0', 'own_shop')).toBe('1,160');
      expect(cell('0', 'other_shops')).toBe('0');
      // The guests are not a partner: the row says "guests", with their item and where it came from.
      expect(cell('1', 'for')).toBe(m('fruit-for-guests'));
      expect(cell('1', 'item')).toBe(names('سیب', 'Apple'));
      expect(cell('1', 'qty')).toBe('3');
      expect(cell('1', 'value')).toBe('3,000');
      expect(cell('1', 'own_shop')).toBe('0');
      expect(cell('1', 'other_shops')).toBe('3,000');
      // The totals are the backend's.
      expect(totalsCell('qty')).toBe('5');
      expect(totalsCell('value')).toBe('4,160');
      expect(totalsCell('own_shop')).toBe('1,160');
      expect(totalsCell('other_shops')).toBe('3,000');
    });

    it('names the row by who it was for and the item, and heads the columns with the words of the fruit screen', async () => {
      await openView('K');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      expect(drawnColumns()).toEqual(['for', 'item', 'qty', 'value', 'own_shop', 'other_shops']);
      const heading = (key: string) =>
        visible(document.querySelector(`.ag-header-cell[col-id="${key}"]`)?.textContent)
          .replace(/\s+/g, ' ')
          .trim();
      expect(heading('for')).toBe(m('fruit-filter-for'));
      expect(heading('own_shop')).toBe(m('fruit-source-own'));
      expect(heading('other_shops')).toBe(m('fruit-source-other'));
    });

    it('draws a chart labelled for a screen reader, with the value of each row from our shop and from other shops', async () => {
      await openView('K');
      const chart = await screen.findByRole(
        'img',
        { name: named(m('analytics-chart-k')) },
        SLOW_MS,
      );
      expect(chart).toBeVisible();
    });

    it('offers no comparison, no side and no trip goods, because the backend has none of them for the fruit', async () => {
      const { user } = await openView('K');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-compare') })).toBeNull();
      expect(screen.queryByRole('combobox', { name: m('analytics-filter-side') })).toBeNull();
      // The goods box holds all, consigned and the gardens' goods only.
      await user.click(screen.getByRole('combobox', { name: m('analytics-filter-goods') }));
      const options = (await screen.findAllByRole('option', undefined, SLOW_MS)).map((option) =>
        visible(option.textContent),
      );
      expect(options).toEqual([
        m('analytics-goods-all'),
        m('analytics-goods-consigned'),
        m('analytics-goods-garden'),
      ]);
    });

    it('takes a partner from its box and asks again with him', async () => {
      const { user } = await openView('K');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(
        await screen.findByRole('option', { name: names('الف', 'Partner A') }, SLOW_MS),
      );
      await waitFor(() => {
        expect(lastRun()?.filters.partner_id).toBe(1);
      }, SLOW_MS);
    });

    it('Enter on a row opens the fruit lines behind it, and the totals open the whole filter', async () => {
      const { user } = await openView('K');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="1"] [col-id="for"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      // The guests' row: the key says guests, with the item, and no column.
      expect(lastLines()).toMatchObject({
        view: 'K',
        row_key: { guests: true, item_id: 2 },
        column_key: null,
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(ANALYTICS_LINES_K_GUESTS.rows.length);
      }, SLOW_MS);
      // The lines are the lines of the fruit register: the document, who it was for, where from.
      expect(drawnColumns()).toEqual(['date', 'doc', 'for', 'item', 'qty', 'rate', 'value']);
    });

    it('shows the print with the columns of the backend, and the title of the view', async () => {
      const { user } = await openView('K', { print: ANALYTICS_PRINT_DATA_K_EN });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByText(ANALYTICS_PRINT_DATA_K_EN.firm.name, undefined, SLOW_MS);
      expect(vi.mocked(analyticsApi.analyticsPrintData).mock.calls[0]?.[0]).toMatchObject({
        view: 'K',
        language,
        paper: 'a4',
      });
      const heads = Array.from(document.querySelectorAll('.print-report thead th')).map((th) =>
        visible(th.textContent),
      );
      expect(heads).toHaveLength(ANALYTICS_PRINT_DATA_K_EN.columns.length);
      expect(heads.some((head) => /analytics-col-|report-col-|fruit-/.test(head))).toBe(false);
    });

    it('keeps the rows across a language switch, writing the partner and the guests in the other language', async () => {
      await openView('K');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      const before = runCount();
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(cell('0', 'for')).toBe(other === 'ur' ? 'الف' : 'Partner A');
      }, SLOW_MS);
      expect(cell('1', 'for')).toBe(message(other, 'fruit-for-guests'));
      expect(runCount()).toBe(before);
    });

    it('says there is nothing when no fruit was taken', async () => {
      await openView('K', { pages: { K: { ...ANALYTICS_RUN_K_FRUIT, rows: [], total_count: 0 } } });
      await screen.findByText(named(m('analytics-empty')), undefined, SLOW_MS);
    });
  });

  describe('view F, the fruit of each partner', () => {
    it('has the fruit of E2E-42 in its fruit column, a minus for what he took, and a total', async () => {
      await openView('F', { fruitInF: true });
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1', '2']);
      }, SLOW_MS);
      // الف took 2 پیٹی (1,160) and has his third of the guests' fruit (1,000): 2,160 off his total.
      expect(cell('0', 'fruit')).toBe('-2,160');
      expect(cell('1', 'fruit')).toBe('-1,000');
      expect(cell('2', 'fruit')).toBe('-1,000');
      expect(totalsCell('fruit')).toBe('-4,160');
    });

    it('opens the lines behind a fruit number with that money and the row’s partner', async () => {
      const { user } = await openView('F', { fruitInF: true });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="0"] [col-id="fruit"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'F',
        row_key: { partner_id: 1 },
        column_key: { money: 'fruit' },
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(ANALYTICS_LINES_F_FRUIT_ALIF.rows.length);
      }, SLOW_MS);
      expect(ANALYTICS_LINES_F_FRUIT_ALIF.rows.length).toBeGreaterThan(0);
      expect(drawnColumns()).toEqual(['date', 'doc', 'partner', 'amount']);
    });

    it('opens the lines of the whole filter for the totals of a money column, without a partner', async () => {
      const { user } = await openView('F', { fruitInF: true });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.click(pick('.ag-row-pinned [col-id="fruit"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        view: 'F',
        row_key: { partner_id: null },
        column_key: { money: 'fruit' },
      });
    });

    it('opens the garden and trip lines of a partner the same way, and a plain column as before', async () => {
      const { user } = await openView('F', { fruitInF: true });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="1"] [col-id="trips"]'));
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('analytics-lines-title')) },
        SLOW_MS,
      );
      expect(lastLines()).toMatchObject({
        row_key: { partner_id: 2 },
        column_key: { money: 'trips' },
      });
    });
  });
});
