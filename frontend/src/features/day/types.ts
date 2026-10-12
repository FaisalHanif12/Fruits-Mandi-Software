// INTERIM (plan 10 T3): the shape of the day commands as the plan's contract and the proposal sent
// to backend describe them (`day_close_preview`, `day_close`, `day_reopen`). When backend sends the
// generated bindings, `api.ts` is the only file that changes. Code Standards F2: types of backend
// data come from bindings. Dates the user types are text; amounts the backend sends are `Balance`.

import type { Balance } from '../../shared/accounts';

/**
 * What closing the days up to a date would do, as the backend sees it now (PRD FR-21.1). Later
 * plans add lines (بکری count, stock); a screen shows only the fields it knows.
 */
export interface DayClosePreview {
  /** The oldest open day, `YYYY-MM-DD`. Null when every day up to today is closed already. */
  first_day: string | null;
  /** The date asked for, `YYYY-MM-DD`. */
  up_to: string;
  /** How many days would be closed. 0 when there is nothing to close. */
  days: number;
  /** Debit total minus credit total of the books. It must be zero before a day is closed. */
  trial_difference: Balance;
  cash_in_hand: Balance;
}
