// Test helper: sets the answers of the dashboard commands from the backend's own answers in
// dashboard-real.ts. The wrappers are mocked in the test file with vi.mock. An error is a rejected
// promise (`CommandError`), the way `invoke` fails.

import { vi } from 'vitest';
import type { AttentionList } from '../../src/bindings/AttentionList';
import type { Dashboard } from '../../src/bindings/Dashboard';
import * as dashboardApi from '../../src/features/dashboard/api';
import { ATTENTION_18_10, DASHBOARD_WORKED_DAY } from './dashboard-real';

export * from './dashboard-real';

export interface DashboardOptions {
  dashboard?: Dashboard | Error;
  attention?: AttentionList | Error;
}

/** Sets the dashboard commands. The dashboard's own attention list is the one it holds. */
export function fakeDashboard(options: DashboardOptions = {}) {
  vi.mocked(dashboardApi.dashboardGet).mockImplementation(() =>
    options.dashboard instanceof Error
      ? Promise.reject(options.dashboard)
      : Promise.resolve(options.dashboard ?? DASHBOARD_WORKED_DAY),
  );
  vi.mocked(dashboardApi.assistAttention).mockImplementation(() =>
    options.attention instanceof Error
      ? Promise.reject(options.attention)
      : Promise.resolve(options.attention ?? ATTENTION_18_10),
  );
}
