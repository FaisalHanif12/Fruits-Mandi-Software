// Typed wrappers around the app-level commands (backend/app/src/commands/app.rs). Code Standards F4.

import type { AppInfo } from '../bindings/AppInfo';
import type { DataStatus } from '../bindings/DataStatus';
import type { DayStatus } from '../bindings/DayStatus';
import { callCommand, hasAppBridge } from '../shared/api';

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

/** Today's date and whether the day is open. Needs a signed-in user. */
export function dayStatus(): Promise<DayStatus> {
  return callCommand<DayStatus>('day_status');
}

/** Closes the app (the sign-in screen's Exit button). Does nothing in a plain browser tab. */
export async function exitApp(): Promise<void> {
  if (!hasAppBridge()) return;
  const { exit } = await import('@tauri-apps/plugin-process');
  await exit(0);
}
