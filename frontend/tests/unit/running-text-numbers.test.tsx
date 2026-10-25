// Every number in a line of words is in its own isolate in Inter (Code Standards F25, NFR-URD-13, Umar
// 26-10-2026): a date, a count, a quantity and a rate, a code, the number of a trip. A digit left in
// the text of the words around it is drawn in their font, and an Urdu font may draw it in an Eastern
// shape. The test draws the screens of the reports and the analytics in both languages, with their
// lines and sentences, and fails when a digit sits outside a `<Num>`, a `<Code>` or a number cell.
// Titles and tabs are names, not running text ("Top 10"), and are the one exception.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportRow } from '../../src/bindings/ReportRow';
import { useAnalyticsStore } from '../../src/features/analytics/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { bareDigits } from '../helpers/bare-digits';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import * as backupApi from '../../src/features/backup/api';
import * as maintenanceApi from '../../src/features/settings/maintenance-api';
import { fakeAnalytics } from '../helpers/analytics';
import { HEALTH_LAST, UPDATE_NONE } from '../helpers/b5-real';
import { STATUS_AFTER } from '../helpers/backup-real';
import { DASHBOARD_GARDEN, fakeDashboard } from '../helpers/dashboard';
import { fakeGardens } from '../helpers/gardens';
import { paisaForTest } from '../helpers/paisa';
import { fakePartners, partnerRow } from '../helpers/partners';
import { R31_BAY, fakeReports } from '../helpers/reports';
import { fakeTrips } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/features/settings/maintenance-api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners([
    partnerRow(1, 'الف', 'Partner A', 4000),
    partnerRow(2, 'ب', 'Partner B', 3000),
    partnerRow(3, 'ج', 'Partner C', 3000),
  ]);
  fakeTrips();
  fakeGardens();
  fakeDashboard({ dashboard: DASHBOARD_GARDEN });
  vi.mocked(backupApi.backupStatus).mockResolvedValue(STATUS_AFTER);
  vi.mocked(maintenanceApi.healthLast).mockResolvedValue({
    ...HEALTH_LAST,
    checked_at: '2026-10-10T09:05:00Z',
  });
  vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_NONE);
  useAnalyticsStore.getState().reset();
});

afterEach(() => {
  leaveAppWindow();
});

/** The statement of ب with one fruit sentence and one garden sentence, like the backend writes them. */
function statementWithSentences(): ReportPage {
  const base = R31_BAY.rows[0];
  if (base === undefined) throw new Error('The R31 fixture has no row.');
  const sentence = (id: string, args: Record<string, string>): ReportCell => ({
    type: 'message',
    id,
    args: Object.entries(args).map(([name, text]) => ({ name, value: { type: 'text', text } })),
  });
  const line = (heading: string, detail: ReportCell): ReportRow => ({
    ...base,
    cells: base.cells.map((found, index): ReportCell => {
      if (index === 1) return { type: 'label', id: heading };
      if (index === 2) return detail;
      if (index === 3) return { type: 'money', paisa: paisaForTest(-100_000) };
      return found;
    }),
  });
  return {
    ...R31_BAY,
    rows: [
      line(
        'partners-statement-heading-fruit',
        sentence('partners-statement-detail-fruit-taken', {
          for: 'guests',
          subject: 'Apple',
          qty: '3',
          packing: 'Box',
          rate: '1,000',
          value: '3,000',
        }),
      ),
      line(
        'partners-statement-heading-garden',
        sentence('partners-statement-detail-garden-closed', {
          garden: 'G-1',
          cost: '55,000',
          sales: '66,400',
          result: '11,400',
          share: '60',
          part: '6,840',
          back: '39,840',
        }),
      ),
    ],
    total_count: 2,
  };
}

const SCREENS: readonly { path: string; ready: string }[] = [
  { path: '/reports/R5', ready: '.ag-row' },
  { path: '/reports/R37', ready: '.ag-row' },
  { path: '/reports/R31?partner=2', ready: '.ag-row' },
  { path: '/reports/R36', ready: '.ag-row' },
  { path: '/reports/R34', ready: '.ag-row' },
  { path: '/reports/R10', ready: '.ag-row' },
  { path: '/reports/R20', ready: '.ag-row' },
  { path: '/reports/R18', ready: '.ag-row' },
  { path: '/reports/R35', ready: '.ag-row' },
  { path: '/analytics/A', ready: '.ag-row' },
  { path: '/analytics/F', ready: '.ag-row' },
  { path: '/analytics/I', ready: '.ag-row' },
  { path: '/analytics/J', ready: '.ag-row' },
  { path: '/analytics/K', ready: '.ag-row' },
  // The dashboard, with a garden open: its tiles, notes and lists carry counts and dates in words.
  { path: '/', ready: 'main button[data-nav-field]' },
];

describe.each(LANGUAGES)('numbers in running text are in Inter (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it.each(SCREENS)('$path has no digit outside a number', async ({ path, ready }) => {
    fakeReports(
      path.startsWith('/reports/R31') ? { pages: { R31: statementWithSentences() } } : {},
    );
    fakeAnalytics();
    await renderApp(path);
    await screen.findByRole('heading', { level: 1 }, SLOW_MS);
    await waitFor(() => {
      expect(document.querySelector(ready)).not.toBeNull();
    }, SLOW_MS);
    // Let a second answer, a chart or a window line settle before the page is read.
    await new Promise((resolve) => setTimeout(resolve, 500));
    expect(bareDigits(document.body)).toEqual([]);
  });

  it('the guard itself sees a bare digit and a number in its isolate', () => {
    // A box that is not in the page, so it cannot leak into another test.
    const box = document.createElement('div');
    box.innerHTML =
      '<p>From 10-10-2026</p><p>From <bdi class="number-run">10-10-2026</bdi></p><h1>Top 10</h1>';
    const bare = bareDigits(box);
    expect(bare).toHaveLength(1);
    expect(bare[0]?.text).toBe('From 10-10-2026');
  });
});
