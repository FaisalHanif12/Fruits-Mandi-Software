// Typed wrappers around the audit commands (plan 22, PRD FR-21.3). Both only read.

import { callCommand } from '../../shared/api';
import type { AuditListInput, AuditOptions, AuditPage } from './types';

/** One page of the audit log, newest first, with the number of entries of the whole filter. */
export function auditList(input: AuditListInput): Promise<AuditPage> {
  return callCommand<AuditPage>('audit_list', { input });
}

/** The values the filter boxes offer: the users, the actions and the areas that appear in the log. */
export function auditOptions(): Promise<AuditOptions> {
  return callCommand<AuditOptions>('audit_options');
}
