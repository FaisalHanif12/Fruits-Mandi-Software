// The report screen of plan 38 (R4): a title of a column sorts the whole report by the backend, the
// reports in date order ask for a heading row above each month (`group_by_month`), a month heading
// is drawn like a group row and opens nothing, a ledger sorted by another column says that its
// balance shows in date order only, and the print and the Excel sheet carry the order that is on
// screen. The figures are the backend's. Urdu and English.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportPage } from '../../src/bindings/ReportPage';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { R12_DAY, R2_ALL, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  window.localStorage.clear();
});

afterEach(() => {
  leaveAppWindow();
});

/** R2 with a month heading above its two lines, as the backend sends it with `group_by_month`. */
function withMonthRow(page: ReportPage): ReportPage {
  const first = page.rows[0];
  if (first === undefined) throw new Error('no row');
  const heading = {
    cells: first.cells.map((_, index) =>
      index === 0 ? { type: 'month' as const, year: 2026, month: 10 } : { type: 'empty' as const },
    ),
    source: null,
    kind: 'month' as const,
  };
  return { ...page, rows: [heading, ...page.rows], total_count: page.total_count };
}

function lastRun() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
}

describe.each(LANGUAGES)('a report with sorting and month headings (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);

  async function open(id: string, pages: Parameters<typeof fakeReports>[0] = {}) {
    fakeReports(pages);
    await renderApp(`/reports/${id}`);
    return userEvent.setup();
  }

  it('asks for the month headings of a report in date order, and draws one as a heading row', async () => {
    await open('R2', { pages: { R2: withMonthRow(R2_ALL) } });
    await waitFor(() => {
      expect(document.querySelector('.ag-row[row-id="0"].row-group')).not.toBeNull();
    }, SLOW_MS);
    expect(lastRun()).toMatchObject({ id: 'R2', sort: null, group_by_month: true });
  });

  it('a click on a title sorts the whole report by the backend and turns the headings off', async () => {
    const user = await open('R2');
    const title = await screen.findByText((text) => visible(text) === m('report-col-qty'));
    await user.click(title);
    await waitFor(() => {
      expect(lastRun()).toMatchObject({
        id: 'R2',
        sort: { column: 'qty', direction: 'asc', language },
        group_by_month: false,
      });
    }, SLOW_MS);
  });

  it('says the balance shows in date order only, when a ledger is sorted by another column', async () => {
    const user = await open('R12');
    const other = R12_DAY.columns.find((column) => column.kind === 'money');
    if (other === undefined) throw new Error('no money column');
    await waitFor(() => {
      expect(document.querySelector(`.ag-header-cell[col-id="${other.key}"]`)).not.toBeNull();
    }, SLOW_MS);
    expect(
      screen.queryByText((text) => visible(text) === m('report-balance-date-order-only')),
    ).toBeNull();
    const header = document.querySelector(
      `.ag-header-cell[col-id="${other.key}"] .ag-header-cell-text`,
    );
    if (header === null) throw new Error('no header');
    await user.click(header);
    expect(
      await screen.findByText(
        (text) => visible(text) === m('report-balance-date-order-only'),
        undefined,
        SLOW_MS,
      ),
    ).toBeInTheDocument();
  });
});
