// The Excel button of the sales page (plan 83, PRD FR-9.9, FR-19.3) in the Urdu and the English app: it asks where to
// save, then writes every row of the period and the filters on screen in the app language, as the `sales_period` kind
// of `tables_export_excel`, with no paper (the sheet is the print's own table). Against a fake backend.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dashboardApi from '../../src/features/dashboard/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { message, visible } from '../helpers/render';
import { DAY, period } from '../helpers/sales-data';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/api/dialog');
vi.mock('../../src/shared/api/events');

const PICKED = encodeURIComponent(JSON.stringify({ id: 19, name_ur: 'رحیم', name_en: 'Rahim' }));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardSalesPeriod).mockResolvedValue(period());
  vi.mocked(dashboardApi.dashboardSalesDay).mockResolvedValue(DAY);
  // The Excel button listens for the backend's progress and asks where to save (shared/export).
  vi.mocked(events.listenEvent).mockImplementation(() => Promise.resolve(() => undefined));
  vi.mocked(events.emitEvent).mockResolvedValue(undefined);
  vi.mocked(dialog.chooseSavePath).mockResolvedValue('C:\\Reports\\sales');
  vi.mocked(reportsApi.tablesExportExcel).mockResolvedValue({
    path: 'C:\\Reports\\sales.xlsx',
    rows: 1,
    size_bytes: 9000,
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the Excel file of the sales page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const excel = () =>
    screen.findByRole('button', {
      name: (name) => visible(name).includes(message(language, 'export-excel')),
    });

  it('writes the month on screen with no filters, in the app language, with no paper', async () => {
    const user = userEvent.setup();
    await renderApp('/sales');
    await screen.findByRole('treegrid');
    await user.click(await excel());
    await waitFor(() => {
      expect(reportsApi.tablesExportExcel).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.tablesExportExcel).mock.calls[0]?.[0].source).toEqual({
      kind: 'sales_period',
      period: { kind: 'month', month: '2026-10' },
      language,
      paper: null,
    });
  });

  it('carries the party, the item and the goods of the page', async () => {
    const user = userEvent.setup();
    await renderApp(`/sales?side=seller&party=${PICKED}&goods=garden`);
    await screen.findByRole('treegrid');
    await user.click(await excel());
    await waitFor(() => {
      expect(reportsApi.tablesExportExcel).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.tablesExportExcel).mock.calls[0]?.[0].source).toEqual({
      kind: 'sales_period',
      period: { kind: 'month', month: '2026-10' },
      party: { side: 'seller', account_id: 19 },
      goods: 'garden',
      language,
      paper: null,
    });
  });
});
