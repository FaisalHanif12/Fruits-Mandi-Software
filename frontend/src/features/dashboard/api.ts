// Typed wrappers around the dashboard and attention commands (plan 19). Both are read only: they
// write nothing and work out nothing the screens could not read (Code Standards B29).

import type { AttentionList } from '../../bindings/AttentionList';
import type { Dashboard } from '../../bindings/Dashboard';
import { callCommand } from '../../shared/api';

/** Everything the dashboard shows (PRD 9), in one call. One failure fails the whole answer. */
export function dashboardGet(): Promise<Dashboard> {
  return callCommand<Dashboard>('dashboard_get');
}

/** The attention list and the number on the bell (PRD FR-7.44). */
export function assistAttention(): Promise<AttentionList> {
  return callCommand<AttentionList>('assist_attention');
}
