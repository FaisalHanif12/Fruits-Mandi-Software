// Print, PDF and Excel of the pages behind the dashboard tiles (plan 79, PRD FR-19.10, FR-19.3) in the Urdu and the English
// app, against a fake backend: on each page the Print button shows the print preview of the filter on screen in
// the print layout (every row, read from the backend in the print language, never added up on screen), Back
// returns to the page, and the Excel button is there. The cash sales page prints the month, and one day from
// its open row.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { printData } from '../helpers/collection-sheet';
import { fakeDashboard } from '../helpers/dashboard';
import {
  ADVANCES_OPEN,
  CASH_DAY,
  CASH_MONTH,
  LENT_OPEN,
  SELLERS_ALL,
  TOTAL_DUES,
} from '../helpers/dues-pages';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

type PrintRead = (input: { language: 'ur' | 'en'; paper: 'a4' | 'a5' | 'mm80' | null }) => unknown;

/** Every print-data command answers with the same small sheet, in the language and on the paper asked. */
function answerPrints(): void {
  const answer: PrintRead = (input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4'));
  vi.mocked(dashboardApi.dashboardSellerDuesPrintData).mockImplementation(answer as never);
  vi.mocked(dashboardApi.dashboardTotalDuesPrintData).mockImplementation(answer as never);
  vi.mocked(dashboardApi.dashboardLentOpenPrintData).mockImplementation(answer as never);
  vi.mocked(dashboardApi.dashboardAdvancesOpenPrintData).mockImplementation(answer as never);
  vi.mocked(dashboardApi.dashboardCashSalesMonthPrintData).mockImplementation(answer as never);
  vi.mocked(dashboardApi.dashboardCashSalesDayPrintData).mockImplementation(answer as never);
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardSellerDues).mockResolvedValue(SELLERS_ALL);
  vi.mocked(dashboardApi.dashboardTotalDues).mockResolvedValue(TOTAL_DUES);
  vi.mocked(dashboardApi.dashboardLentOpen).mockResolvedValue(LENT_OPEN);
  vi.mocked(dashboardApi.dashboardAdvancesOpen).mockResolvedValue(ADVANCES_OPEN);
  vi.mocked(dashboardApi.dashboardCashSalesMonth).mockResolvedValue(CASH_MONTH);
  vi.mocked(dashboardApi.dashboardCashSalesDay).mockResolvedValue(CASH_DAY);
  answerPrints();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the print of the dashboard pages (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const printButton = () =>
    screen.findByRole('button', { name: (name) => visible(name).includes(m('report-preview')) });
  const back = () =>
    within(screen.getByRole('main')).findByRole('button', {
      name: (name) => visible(name) === m('common-back'),
    });

  /** Opens the page, presses Print, and checks the preview is read with the filter on screen and Back returns. */
  async function printFrom(
    path: string,
    calls: () => readonly (readonly unknown[])[],
    expected: Record<string, unknown>,
  ) {
    const user = userEvent.setup();
    await renderApp(path);
    await user.click(await printButton());
    await waitFor(() => {
      expect(calls()[0]?.[0]).toEqual({
        ...expected,
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    });
    // The Excel button of the page is in the preview frame as well.
    expect(
      screen.getAllByRole('button', { name: (name) => visible(name).includes(m('export-excel')) })
        .length,
    ).toBeGreaterThan(0);
    await user.click(await back());
    // The preview is gone and the page is back. These pages are opened from a tile, so they keep the
    // Back button of the top bar (PRD FR-7.92); what went is the preview's own Back button, which was
    // inside the page area.
    await waitFor(() => {
      expect(document.querySelector('.print-sheet')).toBeNull();
    });
    expect(
      within(screen.getByRole('main')).queryByRole('button', { name: m('common-back') }),
    ).toBeNull();
    expect(screen.getAllByRole('button', { name: m('common-back') })).toHaveLength(1);
  }

  it('the seller dues print holds the side and every seller, in the order on screen', async () => {
    await printFrom(
      '/dues/sellers?side=shop_owes',
      () => vi.mocked(dashboardApi.dashboardSellerDuesPrintData).mock.calls,
      {
        side: 'shop_owes',
        party_id: null,
        search: null,
        sort: null,
      },
    );
  });

  it('the total dues print is the four parts and the grand total', async () => {
    await printFrom(
      '/dues/total',
      () => vi.mocked(dashboardApi.dashboardTotalDuesPrintData).mock.calls,
      {},
    );
  });

  it('the money lent print holds every person who owes', async () => {
    await printFrom(
      '/dues/lent',
      () => vi.mocked(dashboardApi.dashboardLentOpenPrintData).mock.calls,
      {
        search: null,
        sort: null,
      },
    );
  });

  it('the advances print asks for the open advances as of today', async () => {
    await printFrom(
      '/dues/advances',
      () => vi.mocked(dashboardApi.dashboardAdvancesOpenPrintData).mock.calls,
      {
        seller_id: null,
        as_of: '',
        search: null,
        only_open: true,
      },
    );
  });

  it('the cash sales print is the month on screen', async () => {
    await printFrom(
      '/cash-sales',
      () => vi.mocked(dashboardApi.dashboardCashSalesMonthPrintData).mock.calls,
      {
        month: '2026-10',
      },
    );
  });

  it('a day of the cash sales prints from its open row: every buyer of that day', async () => {
    const user = userEvent.setup();
    await renderApp('/cash-sales');
    const tree = await screen.findByRole('treegrid');
    const day = within(tree).getAllByRole('row')[1];
    if (day === undefined) throw new Error('no day row');
    await user.click(day);
    // The open day has its own Print button, next to the page's (which prints the whole month).
    const detail = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[data-cash-sales-day]');
      if (found === null) throw new Error('no day yet');
      return found;
    });
    await user.click(
      within(detail).getByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCashSalesDayPrintData).mock.calls[0]?.[0]).toEqual({
        date: '2026-10-10',
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    });
    await user.click(await back());
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });
});
