// Test helper: sets the answers of the dashboard commands from the backend's own answers in
// dashboard-real.ts. The wrappers are mocked in the test file with vi.mock. An error is a rejected
// promise (`CommandError`), the way `invoke` fails.

import { vi } from 'vitest';
import type { AttentionPanel } from '../../src/bindings/AttentionPanel';
import type { Dashboard } from '../../src/bindings/Dashboard';
import * as dashboardApi from '../../src/features/dashboard/api';
import { PANEL_WORKED_DAY } from './attention-panel';
import { DASHBOARD_WORKED_DAY } from './dashboard-real';

export * from './dashboard-real';

export interface DashboardOptions {
  dashboard?: Dashboard | Error;
  /** The bell's panel (plan 100 J). */
  panel?: AttentionPanel | Error;
}

/** Sets the dashboard commands. The dashboard's own attention list is the one it holds. */
export function fakeDashboard(options: DashboardOptions = {}) {
  vi.mocked(dashboardApi.dashboardGet).mockImplementation(() =>
    options.dashboard instanceof Error
      ? Promise.reject(options.dashboard)
      : Promise.resolve(options.dashboard ?? DASHBOARD_WORKED_DAY),
  );
  vi.mocked(dashboardApi.assistAttention).mockImplementation(() =>
    options.panel instanceof Error
      ? Promise.reject(options.panel)
      : Promise.resolve(options.panel ?? PANEL_WORKED_DAY),
  );
}
