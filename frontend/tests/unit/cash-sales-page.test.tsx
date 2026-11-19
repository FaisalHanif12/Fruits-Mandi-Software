// The cash sales page (plan 79 item 2c, PRD FR-9.8) in the Urdu and the English app, against a fake backend: the
// tile آج کی نقدی opens it; one month, a row for each day with the number of buyers and the نقدی; the month
// total is the backend's; a click on a day opens its buyers in place (a چکتا buyer marked, the بکری, the bill,
// the نقدی and a total row) and the بکری number opens it; the month buttons move the month and keep it in
// the address; and it has its four states.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { CASH_DAY, CASH_MONTH } from '../helpers/dues-pages';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardCashSalesMonth).mockImplementation((input) =>
    Promise.resolve(
      input.month === '2026-09'
        ? {
            ...CASH_MONTH,
            month: '2026-09',
            rows: [],
            buyer_days_total: 0,
            total_paisa: paisaForTest(0),
          }
        : CASH_MONTH,
    ),
  );
  vi.mocked(dashboardApi.dashboardCashSalesDay).mockResolvedValue(CASH_DAY);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the cash sales page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const open = () => renderApp('/cash-sales');
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('cash-sales-title'),
    });
  const button = (id: string) =>
    screen.getByRole('button', { name: (name) => visible(name).includes(m(id)) });

  it('opens on the current month: a row for each day, with its buyers and نقدی, and the backend’s month total', async () => {
    await open();
    await heading();
    const tree = await screen.findByRole('treegrid');
    const text = visible(tree.textContent);
    expect(text).toContain('10-10-2026');
    expect(text).toContain('12,800');
    // The first ask is the current month (no month in the address).
    expect(vi.mocked(dashboardApi.dashboardCashSalesMonth).mock.calls[0]?.[0]).toEqual({
      month: null,
    });
    // The month's own total is the backend's: it is on the facts, and under the column.
    expect(
      visible(screen.getByRole('main').textContent).split('12,800').length - 1,
    ).toBeGreaterThanOrEqual(2);
  });

  it('a click on a day opens its buyers in place: the چکتا buyer, his بکری, the bill and the نقدی, with a total row; the بکری opens', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await heading();
    const tree = await screen.findByRole('treegrid');
    const day = within(tree).getAllByRole('row')[1];
    if (day === undefined) throw new Error('no day row');
    await user.click(day);
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCashSalesDay).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-10',
        offset: 0,
      });
    });
    const table = await screen.findByRole('table', {
      name: (name) => visible(name) === m('cash-sales-day-label'),
    });
    const text = visible(table.textContent);
    expect(text).toContain(language === 'ur' ? 'شاہد' : 'Shahid');
    expect(text).toContain(m('cash-sales-chukta'));
    // The total row says how many different buyers paid: the backend's buyer_count.
    expect(text).toContain(m('cash-sales-day-total', { count: CASH_DAY.buyer_count }));
    expect(text.split('12,800').length - 1).toBeGreaterThanOrEqual(4);
    await user.click(within(table).getByRole('button', { name: (name) => visible(name) === '2' }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 2 });
  });

  it('Enter on a day opens it, by keyboard only', async () => {
    const user = userEvent.setup();
    await open();
    await heading();
    const tree = await screen.findByRole('treegrid');
    tree.focus();
    await user.keyboard('{Enter}');
    expect(
      await screen.findByRole('table', {
        name: (name) => visible(name) === m('cash-sales-day-label'),
      }),
    ).toBeInTheDocument();
  });

  it('the month buttons go to the month before and keep it in the address; the later button is off on this month', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await heading();
    await screen.findByRole('treegrid');
    expect((button('common-month-later') as HTMLButtonElement).disabled).toBe(true);
    await user.click(button('common-month-earlier'));
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCashSalesMonth).mock.calls.at(-1)?.[0]).toEqual({
        month: '2026-09',
      });
    });
    expect(router.state.location.search).toMatchObject({ month: '2026-09' });
    expect(
      await screen.findByText((text) => visible(text) === m('cash-sales-empty')),
    ).toBeInTheDocument();
  });

  it('an error is told with a retry that asks again', async () => {
    vi.mocked(dashboardApi.dashboardCashSalesMonth).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await open();
    const retry = await screen.findByRole('button', {
      name: (name) => visible(name) === m('common-retry'),
    });
    await user.click(retry);
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });

  it('while the month is on its way: a loading state', async () => {
    vi.mocked(dashboardApi.dashboardCashSalesMonth).mockReturnValue(new Promise(() => undefined));
    await open();
    await heading();
    expect(
      screen.getAllByRole('status').some((one) => one.getAttribute('aria-busy') === 'true'),
    ).toBe(true);
  });
});
