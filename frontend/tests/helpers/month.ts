// Test helper: sets the answers of the month close command wrappers (round 7) from the backend's own
// answers in month-real.ts. The wrappers are mocked in the test file with vi.mock. The fake keeps one
// small state, as the backend does: a close makes October closed, a reversal makes it ready again.
// A refusal is a rejected promise (`CommandError`), the way `invoke` fails.

import { vi } from 'vitest';
import type { MonthList } from '../../src/bindings/MonthList';
import type { MonthPreview } from '../../src/bindings/MonthPreview';
import type { MonthRow } from '../../src/bindings/MonthRow';
import type { UiError } from '../../src/bindings/UiError';
import * as monthApi from '../../src/features/month/api';
import { CommandError } from '../../src/shared/api';
import {
  MONTH_LIST_CLOSED,
  MONTH_LIST_READY,
  MONTH_PREVIEW,
  MONTH_PREVIEW_NOT_FINISHED,
  MONTH_REVERSE,
  MONTH_SAVE,
  MONTH_SAVE_DAYS_OPEN,
} from './month-real';

export * from './month-real';

export interface MonthOptions {
  /** The list at the start; the default is the backend's own "ready" list. */
  list?: MonthList | Error;
  /** October starts closed (the backend's "closed" list). */
  closed?: boolean;
  /** Called for every `month_close_save`; return a refusal to refuse that close. */
  refuseSave?: (month: string) => UiError | null;
  /** Called for every `month_close_reverse`; return a refusal to refuse it. */
  refuseReverse?: (month: string, reason: string) => UiError | null;
  /** The preview answers this refusal for a month that is not closed. */
  refusePreview?: (month: string) => UiError | null;
}

export interface FakeMonth {
  /** The list as it is now. */
  rows: () => MonthRow[];
}

/** A month with another state, for the days-open and not-finished screens. */
export function withState(list: MonthList, month: string, state: MonthRow['state']): MonthList {
  return { rows: list.rows.map((row) => (row.month === month ? { ...row, state } : row)) };
}

/** What the preview answers for a month: October's own plan, and the same plan for another month. */
function previewOf(month: string): MonthPreview {
  return { ...MONTH_PREVIEW, month };
}

/** Sets the answers of the month close commands. */
export function fakeMonth(options: MonthOptions = {}): FakeMonth {
  let list: MonthList =
    options.list instanceof Error
      ? MONTH_LIST_READY
      : (options.list ?? (options.closed === true ? MONTH_LIST_CLOSED : MONTH_LIST_READY));
  const setState = (month: string, state: MonthRow['state']) => {
    list = {
      rows: list.rows.map((row) =>
        row.month === month
          ? {
              ...row,
              state,
              closed_on: state === 'closed' ? '2026-11-02' : null,
              closed_entry_id: state === 'closed' ? MONTH_SAVE.entry_id : null,
              reversed_count: state === 'ready' ? row.reversed_count + 1 : row.reversed_count,
            }
          : row,
      ),
    };
  };
  vi.mocked(monthApi.monthList).mockImplementation(() =>
    options.list instanceof Error ? Promise.reject(options.list) : Promise.resolve(list),
  );
  vi.mocked(monthApi.monthClosePreview).mockImplementation((month) => {
    const refused = options.refusePreview?.(month) ?? null;
    if (refused !== null) return Promise.reject(new CommandError(refused));
    const row = list.rows.find((candidate) => candidate.month === month);
    if (row?.state === 'not_finished') {
      return Promise.reject(new CommandError(MONTH_PREVIEW_NOT_FINISHED));
    }
    if (row?.state === 'days_open') {
      return Promise.reject(new CommandError(MONTH_SAVE_DAYS_OPEN));
    }
    return Promise.resolve(previewOf(month));
  });
  vi.mocked(monthApi.monthCloseSave).mockImplementation((month) => {
    const refused = options.refuseSave?.(month) ?? null;
    if (refused !== null) return Promise.reject(new CommandError(refused));
    setState(month, 'closed');
    return Promise.resolve({ ...MONTH_SAVE, month });
  });
  vi.mocked(monthApi.monthCloseReverse).mockImplementation((month, reason) => {
    const refused = options.refuseReverse?.(month, reason) ?? null;
    if (refused !== null) return Promise.reject(new CommandError(refused));
    setState(month, 'ready');
    return Promise.resolve({ ...MONTH_REVERSE, month });
  });
  return { rows: () => list.rows };
}
