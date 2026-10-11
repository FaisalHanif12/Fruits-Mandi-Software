// Typed wrappers around the app-level commands (backend/app/src/commands/app.rs). Code Standards F4.

import type { AppInfo } from '../bindings/AppInfo';
import type { DataStatus } from '../bindings/DataStatus';
import { callCommand } from '../shared/api';

/** The running app's version and identifier. Needs no login (plan 01 Section 4.1). */
export function appInfo(): Promise<AppInfo> {
  return callCommand<AppInfo>('app_info');
}

/**
 * Where the start-up check of the data file stands. Needs no login and no input, so the shell
 * can ask before the login screen (plan 03 Section 4.2b).
 */
export function appDataStatus(): Promise<DataStatus> {
  return callCommand<DataStatus>('app_data_status');
}
