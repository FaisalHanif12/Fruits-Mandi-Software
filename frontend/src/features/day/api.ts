// Typed wrappers around the day commands (plan 10 contract). Components never call invoke()
// (Code Standards F4). The only file that knows the wire shape. The date is the text typed.

import { callCommand } from '../../shared/api';
import type { DayClosePreview, DayStatus } from './types';

/**
 * What closing every day up to `upTo` would do. An empty `upTo` asks for the oldest open day, which
 * the answer names (PRD FR-21.1). Read only; closes nothing.
 */
export function dayClosePreview(upTo: string): Promise<DayClosePreview> {
  return callCommand<DayClosePreview>('day_close_preview', { input: { up_to: upTo } });
}

/**
 * Closes the days in order, from the oldest open one up to `upTo` (PRD FR-21.1). Refused out of
 * order and while the trial balance does not balance. Answers with the new closed-up-to date.
 */
export function dayClose(upTo: string): Promise<DayStatus> {
  return callCommand<DayStatus>('day_close', { input: { up_to: upTo } });
}

/**
 * Reopens a date and every later date (PRD FR-21.2). The reason is required and goes to the audit
 * log, and so is the Owner's sign, as when a day is reopened to fix an entry (FR-21.4; plan 80
 * Q-R2-2): ERR_SIGN_REQUIRED and ERR_SIGN_WRONG on `sign`. Answers with the new closed-up-to date.
 */
export function dayReopen(date: string, reason: string, sign: string): Promise<DayStatus> {
  return callCommand<DayStatus>('day_reopen', { input: { date, reason, sign } });
}
