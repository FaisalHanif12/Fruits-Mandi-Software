// Typed wrappers around the month close commands (round 7 contract, PRD FR-17.6, FR-21.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape. A
// month is the text `YYYY-MM`; the backend reads it and refuses what is not a month.

import { callCommand } from '../../shared/api';
import type { MonthClosed, MonthList, MonthPreview, MonthReversed } from './types';

/** The months from the first entry to this month, newest first, each with where it stands. Read only. */
export function monthList(): Promise<MonthList> {
  return callCommand<MonthList>('month_list');
}

/**
 * What closing the month would post, or the reason it cannot be closed now (the same refusals as
 * the close itself). Read only; posts nothing.
 */
export function monthClosePreview(month: string): Promise<MonthPreview> {
  return callCommand<MonthPreview>('month_close_preview', { input: { month } });
}

/**
 * Closes a finished month: its income and shop expenses move into the partners' capital accounts,
 * by one entry dated today (PRD FR-17.6). No partner's total changes.
 */
export function monthCloseSave(month: string): Promise<MonthClosed> {
  return callCommand<MonthClosed>('month_close_save', { input: { month } });
}

/** Reverses the close in force for a month. The reason is required and goes to the audit log. */
export function monthCloseReverse(month: string, reason: string): Promise<MonthReversed> {
  return callCommand<MonthReversed>('month_close_reverse', { input: { month, reason } });
}
