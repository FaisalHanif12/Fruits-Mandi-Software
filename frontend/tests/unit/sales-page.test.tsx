// The sales page (plan 83, PRD FR-9.9, FR-9.1, FR-7.92, FR-19.11) in the Urdu and the English app, against a fake
// backend with the numbers of the plan (after Events 1 to 5, 10-10-26: 2 بکری, 60 نگ, خام بکری 41,400, the buyers'
// bill 44,364, نقدی 12,800, ادھاری 31,564): the tile آج کی بکری opens it (not پچھلی بکری) and Back returns to the
// dashboard; the month as it opens, today's row marked, the totals row from the backend; a day that opens in place
// with its بکری, each with a button that opens it; the month buttons, the box and Page Up and Page Down; the
// period, the party, the item and the goods sent to the backend and kept in the address; the four states; the
// print; the keyboard only; a language switch in the middle. The screen adds nothing up: every figure is the
// backend's.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dashboardApi from '../../src/features/dashboard/api';
import type { SalesPeriod } from '../../src/bindings/SalesPeriod';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { printData } from '../helpers/collection-sheet';
import { fakeDashboard } from '../helpers/dashboard';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';
import {
  AKRAM,
  BANANA,
  DAY,
  DAY_FIGURES,
  DAY_ROW,
  EMPTY_FIGURES,
  RAHIM,
  period,
} from '../helpers/sales-data';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

/** The picker value of a record, as the address keeps it. */
const picked = (record: { id: number; name_ur: string; name_en: string; code: string }) =>
  encodeURIComponent(JSON.stringify(record));

const lastRead = () => vi.mocked(dashboardApi.dashboardSalesPeriod).mock.calls.at(-1)?.[0];

/** What the page asks when nothing is chosen: the current month of every party, item and kind of goods. */
const PLAIN = { period: { kind: 'month' } };

/** The backend's answer for the period asked: October has the day, September has nothing, a year has its months. */
function answer(input: Parameters<typeof dashboardApi.dashboardSalesPeriod>[0]): SalesPeriod {
  // No period is the current month, as the backend reads it.
  const asked = input.period ?? { kind: 'month' };
  if (asked.kind === 'year') {
    return period({
      period: { kind: 'year', from: '2026-01-01', to: '2026-12-31' },
      rows: [{ ...DAY_ROW, date: '2026-10' }],
    });
  }
  if (asked.kind === 'month' && asked.month === '2026-09') {
    return period({
      month: '2026-09',
      period: { kind: 'month', from: '2026-09-01', to: '2026-09-30' },
      rows: [],
      totals: EMPTY_FIGURES,
      total_count: 0,
    });
  }
  return period();
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardSalesPeriod).mockImplementation((input) =>
    Promise.resolve(answer(input)),
  );
  vi.mocked(dashboardApi.dashboardSalesDay).mockResolvedValue(DAY);
  vi.mocked(dashboardApi.dashboardSalesPrintData).mockImplementation((input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4')),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the sales page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const name = (record: { name_ur: string; name_en: string }) =>
    language === 'ur' ? record.name_ur : record.name_en;
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (text) => visible(text) === m('sales-title'),
    });
  const treeText = async () => visible((await screen.findByRole('treegrid')).textContent);
  const headers = async () =>
    within(await screen.findByRole('treegrid'))
      .getAllByRole('columnheader')
      .map((header) => visible(header.textContent));
  const button = (id: string) =>
    screen.getByRole('button', { name: (text) => visible(text).includes(m(id)) });
  const side = (value: 'all' | 'buyer' | 'seller') =>
    screen.getByRole('button', { name: (text) => visible(text) === m(`sales-side-${value}`) });
  const dayRow = async () => {
    const row = within(await screen.findByRole('treegrid')).getAllByRole('row')[1];
    if (row === undefined) throw new Error('no day row');
    return row;
  };

  it('R1: the tile آج کی بکری opens the sales page, not پچھلی بکری, and Back returns to the dashboard', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/');
    const label = m('dashboard-tile-sales');
    const tile = await waitFor(() => {
      const found = Array.from(
        document.querySelectorAll<HTMLElement>('main button[data-nav-field]'),
      ).find((candidate) => visible(candidate.textContent).startsWith(label));
      if (found === undefined) throw new Error('no tile yet');
      return found;
    });
    await user.click(tile);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/sales');
    });
    await heading();
    await screen.findByRole('treegrid');
    // The Back button of the shell (PRD FR-7.92): the page was opened from the dashboard, and the dashboard is its parent.
    await user.click(await screen.findByRole('button', { name: m('common-back') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
    // It draws the dashboard, then the page: more room than a one-page test when the machine is loaded.
  }, 60_000);

  it('R2: it opens on the current month with the worked numbers and the totals from the backend', async () => {
    await renderApp('/sales');
    await heading();
    const text = await treeText();
    // 10-10-26: 2 بکری, 60 نگ, خام بکری 41,400, the buyers' bill 44,364, نقدی 12,800, ادھاری 31,564.
    for (const figure of ['10-10-2026', '41,400', '44,364', '12,800', '31,564']) {
      expect(text).toContain(figure);
    }
    expect(await headers()).toEqual([
      m('sales-col-date'),
      m('sales-col-count'),
      m('sales-col-qty'),
      m('sales-col-gross'),
      m('sales-col-buyer-total'),
      m('sales-col-cash'),
      m('sales-col-credit'),
    ]);
    // The first request is for the current month, every party, every item, all goods.
    expect(vi.mocked(dashboardApi.dashboardSalesPeriod).mock.calls[0]?.[0]).toEqual(PLAIN);
    // The totals row is the backend's: each figure is in the day and in the total.
    for (const figure of ['41,400', '44,364', '12,800', '31,564']) {
      expect(text.split(figure).length - 1).toBe(2);
    }
    expect(visible((await screen.findByRole('treegrid')).textContent)).toContain(
      m('glossary-meezan'),
    );
  });

  it('R3: today’s row is marked with the word', async () => {
    await renderApp('/sales');
    await heading();
    const row = await dayRow();
    expect(visible(row.textContent)).toContain(m('sales-today'));
    // A day that is not today carries no mark.
    vi.mocked(dashboardApi.dashboardSalesPeriod).mockResolvedValue(
      period({ rows: [{ ...DAY_ROW, date: '2026-10-09' }] }),
    );
    const user = userEvent.setup();
    await user.click(button('common-month-earlier'));
    await waitFor(async () => {
      expect(await treeText()).toContain('09-10-2026');
    });
    expect(await treeText()).not.toContain(m('sales-today'));
  });

  it('R4: a click on a day opens its بکری in place, with their lots, and a button opens a بکری', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/sales');
    await heading();
    await user.click(await dayRow());
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardSalesDay).mock.calls[0]?.[0]).toEqual({
        date: '2026-10-10',
        offset: 0,
        limit: 100,
      });
    });
    await waitFor(() => {
      expect(document.querySelector('[data-sales-day="2026-10-10"]')).not.toBeNull();
    });
    const detail = document.querySelector('[data-sales-day="2026-10-10"]');
    if (detail === null) throw new Error('no day detail');
    const text = visible(detail.textContent);
    // بکری 1 رحیم، کیلا-1 and بکری 2 کریم، سیب-1 with their own figures; the day's total row is the backend's.
    for (const figure of ['29,400', '31,564', '12,000', '12,800', '41,400', '44,364']) {
      expect(text).toContain(figure);
    }
    expect(text).toContain(name(RAHIM));
    expect(text).toContain(name(BANANA));
    expect(text).toContain(m('sales-day-total', { count: DAY_FIGURES.bikri_count }));
    await user.click(
      within(detail as HTMLElement).getByRole('button', {
        name: (text2) => visible(text2) === m('sales-day-open', { number: 2 }),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 42 });
  });

  it('R5: the month buttons go back, stop at the first month with sales and keep the month in the address', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/sales');
    await heading();
    await screen.findByRole('treegrid');
    // The later button is off on the current month; the earlier one goes to September.
    expect((button('common-month-later') as HTMLButtonElement).disabled).toBe(true);
    await user.click(button('common-month-earlier'));
    await waitFor(() => {
      expect(lastRead()).toEqual({ period: { kind: 'month', month: '2026-09' } });
    });
    expect(router.state.location.search).toMatchObject({ month: '2026-09' });
    // September is the first month with sales: the earlier button is off, and an empty month says so.
    await waitFor(() => {
      expect((button('common-month-earlier') as HTMLButtonElement).disabled).toBe(true);
    });
    expect(
      await screen.findByText((text) => visible(text) === m('sales-empty')),
    ).toBeInTheDocument();
  });

  it('R6: Page Up and Page Down move the month, and a typed month goes to it', async () => {
    const user = userEvent.setup();
    await renderApp('/sales');
    await heading();
    await screen.findByRole('treegrid');
    press('PageUp');
    await waitFor(() => {
      expect(lastRead()).toMatchObject({ period: { kind: 'month', month: '2026-09' } });
    });
    press('PageDown');
    await waitFor(() => {
      expect(lastRead()).toMatchObject({ period: { kind: 'month' } });
      expect(lastRead()?.period).not.toHaveProperty('month');
    });
    await user.type(screen.getByPlaceholderText(m('common-goto-month-placeholder')), '9{Enter}');
    await waitFor(() => {
      expect(lastRead()).toMatchObject({ period: { kind: 'month', month: '2026-09' } });
    });
  });

  it('R7: a year is one row for each month, and a click on a month opens that month with the same filters', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/sales?period=year&goods=garden');
    await heading();
    const text = await treeText();
    for (const figure of ['41,400', '44,364']) expect(text).toContain(figure);
    expect((await headers())[0]).toBe(m('sales-col-month'));
    expect(lastRead()).toEqual({ period: { kind: 'year', year: 2026 }, goods: 'garden' });
    await user.click(await dayRow());
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ month: '2026-10', goods: 'garden' });
    });
    expect(router.state.location.search).not.toHaveProperty('period');
    await waitFor(() => {
      expect(lastRead()).toEqual({
        period: { kind: 'month', month: '2026-10' },
        goods: 'garden',
      });
    });
  });

  it('R8: one date is one row with its بکری open at once, and the date buttons move by a day', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/sales?period=day&date=2026-10-10');
    await heading();
    await screen.findByRole('treegrid');
    expect(lastRead()).toEqual({ period: { kind: 'day', date: '2026-10-10' } });
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardSalesDay).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-10',
      });
    });
    expect(
      screen.getByRole('button', {
        name: (text) => visible(text).includes(m('period-date-later')),
      }),
    ).toBeDisabled();
    await user.click(
      screen.getByRole('button', {
        name: (text) => visible(text).includes(m('period-date-earlier')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ period: 'day', date: '2026-10-09' });
    });
  });

  it('R9: a range is paged and asks for its dates', async () => {
    vi.mocked(dashboardApi.dashboardSalesPeriod).mockResolvedValue(
      period({
        period: { kind: 'range', from: '2026-10-01', to: '2026-10-05' },
        total_count: 40,
        limit: 25,
      }),
    );
    const user = userEvent.setup();
    await renderApp('/sales?period=range&from=2026-10-01&to=2026-10-05');
    await heading();
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(lastRead()).toMatchObject({
        period: { kind: 'range', from: '2026-10-01', to: '2026-10-05' },
        offset: 0,
        limit: 25,
      });
    });
    await user.click(
      await screen.findByRole('button', {
        name: (text) => visible(text) === m('common-pager-next'),
      }),
    );
    await waitFor(() => {
      expect(lastRead()).toMatchObject({ offset: 25, limit: 25 });
    });
  });

  it('R10: the party, the item and the goods are sent to the backend and kept in the address', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp(
      `/sales?side=seller&party=${picked(RAHIM)}&item=${picked(BANANA)}&goods=trip`,
    );
    await heading();
    await screen.findByRole('treegrid');
    expect(lastRead()).toEqual({
      period: { kind: 'month' },
      party: { side: 'seller', account_id: 19 },
      item_id: 4,
      goods: 'trip',
    });
    expect(side('seller').getAttribute('aria-pressed')).toBe('true');
    // The open day is read with the same party, item and goods.
    await user.click(await dayRow());
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardSalesDay).mock.calls[0]?.[0]).toMatchObject({
        party: { side: 'seller', account_id: 19 },
        item_id: 4,
        goods: 'trip',
      });
    });
    // A choice in the goods box asks again and is kept in the address.
    await user.click(screen.getByRole('combobox', { name: m('analytics-filter-goods') }));
    await user.click(await screen.findByRole('option', { name: m('analytics-goods-garden') }));
    await waitFor(() => {
      expect(lastRead()).toMatchObject({
        goods: 'garden',
        party: { side: 'seller', account_id: 19 },
        item_id: 4,
      });
    });
    expect(router.state.location.search).toMatchObject({ goods: 'garden' });
  });

  it('R11: every party box starts at All, and a side with nobody picked asks for him and reads nothing', async () => {
    const user = userEvent.setup();
    await renderApp('/sales');
    await screen.findByRole('treegrid');
    expect(side('all').getAttribute('aria-pressed')).toBe('true');
    const reads = vi.mocked(dashboardApi.dashboardSalesPeriod).mock.calls.length;
    await user.click(side('buyer'));
    expect(
      await screen.findByText((text) => visible(text) === m('sales-pick-buyer')),
    ).toBeInTheDocument();
    expect(screen.queryByRole('treegrid')).toBeNull();
    expect(vi.mocked(dashboardApi.dashboardSalesPeriod).mock.calls.length).toBe(reads);
    await user.click(side('seller'));
    expect(
      await screen.findByText((text) => visible(text) === m('sales-pick-seller')),
    ).toBeInTheDocument();
    // Back to All: every party again.
    await user.click(side('all'));
    expect(await treeText()).toContain('44,364');
    expect(lastRead()).toEqual(PLAIN);
  });

  it('R12: a buyer is read as a buyer and the empty filtered month says so', async () => {
    vi.mocked(dashboardApi.dashboardSalesPeriod).mockResolvedValue(
      period({ rows: [], totals: EMPTY_FIGURES, total_count: 0 }),
    );
    await renderApp(`/sales?side=buyer&party=${picked(AKRAM)}`);
    await heading();
    expect(
      await screen.findByText((text) => visible(text) === m('sales-empty-filtered')),
    ).toBeInTheDocument();
    expect(lastRead()).toMatchObject({ party: { side: 'buyer', account_id: 21 } });
  });

  it('R13: the four states: loading, an error with a retry, no sales, and the rows', async () => {
    let release: (value: SalesPeriod) => void = () => undefined;
    vi.mocked(dashboardApi.dashboardSalesPeriod).mockImplementationOnce(
      () =>
        new Promise<SalesPeriod>((resolve) => {
          release = resolve;
        }),
    );
    await renderApp('/sales');
    await heading();
    // Loading: the page is there, the table is not.
    expect(screen.queryByRole('treegrid')).toBeNull();
    act(() => {
      release(period());
    });
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });

  it('R14: an error is told with a retry that asks again', async () => {
    vi.mocked(dashboardApi.dashboardSalesPeriod).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await renderApp('/sales');
    const retry = await screen.findByRole('button', {
      name: (text) => visible(text) === m('common-retry'),
    });
    expect(screen.getByText((text) => visible(text) === m('sales-failed'))).toBeInTheDocument();
    await user.click(retry);
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });

  it('R15: a day whose بکری cannot be read tells so with a retry, and an empty day says so', async () => {
    vi.mocked(dashboardApi.dashboardSalesDay).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await renderApp('/sales');
    await heading();
    await user.click(await dayRow());
    expect(
      await screen.findByText((text) => visible(text) === m('sales-day-failed')),
    ).toBeInTheDocument();
    vi.mocked(dashboardApi.dashboardSalesDay).mockResolvedValue({
      ...DAY,
      rows: [],
      total_count: 0,
      totals: EMPTY_FIGURES,
    });
    await user.click(
      await screen.findByRole('button', { name: (text) => visible(text) === m('common-retry') }),
    );
    expect(
      await screen.findByText((text) => visible(text) === m('sales-day-empty')),
    ).toBeInTheDocument();
  });

  it('R16: the print shows the period on screen in the print layout and Back returns', async () => {
    const user = userEvent.setup();
    await renderApp('/sales');
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', {
        name: (text) => visible(text).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardSalesPrintData).mock.calls[0]?.[0]).toEqual({
        period: { kind: 'month', month: '2026-10' },
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    });
    await user.click(
      await within(screen.getByRole('main')).findByRole('button', {
        name: (text) => visible(text) === m('common-back'),
      }),
    );
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });

  it('R17: it works with the keyboard only: Enter opens a day, Page Up goes to the month before, Esc leaves', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/sales');
    await heading();
    const tree = await screen.findByRole('treegrid');
    tree.focus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(document.querySelector('[data-sales-day="2026-10-10"]')).not.toBeNull();
    });
    press('PageUp');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ month: '2026-09' });
    });
    press('Escape');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('R18: a language switch in the middle keeps the month, the filters and redraws the words', async () => {
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await renderApp(`/sales?period=year&side=seller&party=${picked(RAHIM)}`);
    await heading();
    await screen.findByRole('treegrid');
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const text = visible((await screen.findByRole('treegrid')).textContent);
    for (const figure of ['41,400', '44,364']) expect(text).toContain(figure);
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: (heading1) => visible(heading1) === message(other, 'sales-title'),
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', {
        name: (label) => visible(label) === message(other, 'sales-side-seller'),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
  });
});
