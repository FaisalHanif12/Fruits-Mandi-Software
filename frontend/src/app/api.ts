// Typed wrappers around the app-level commands (backend/app/src/commands/app.rs). Code Standards F4.

import type { AppInfo } from '../bindings/AppInfo';
import { callCommand } from '../shared/api';

/** The running app's version and identifier. Needs no login (plan 01 Section 4.1). */
export function appInfo(): Promise<AppInfo> {
  return callCommand<AppInfo>('app_info');
}
