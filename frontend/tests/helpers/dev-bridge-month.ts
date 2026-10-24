// A stand-in for the Tauri bridge for the development page /dev/month (round 7) only. In a plain
// browser tab there is no backend, so the month close screen cannot be looked at. This answers the
// four month commands with the backend's own fixtures (month-real.ts), and keeps the small state a
// close and a reversal change, so the screen can be opened in a browser, used, and photographed in
// both languages and at every window size. It answers nothing else: any other command is refused
// with ERR_NOT_IMPLEMENTED. Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/month?closed`:
//   closed     October starts closed        daysopen   September has days that are still open
//   empty      there is no month yet        down       the backend fails
//   twice      a close is refused (the month was closed meanwhile)

import type { MonthList } from '../../src/bindings/MonthList';
import type { MonthRow } from '../../src/bindings/MonthRow';
import { devFlag } from './dev-bridge';
import {
  MONTH_LIST_CLOSED,
  MONTH_LIST_READY,
  MONTH_PREVIEW,
  MONTH_PREVIEW_NOT_FINISHED,
  MONTH_REVERSE,
  MONTH_SAVE,
  MONTH_SAVE_DAYS_OPEN,
  MONTH_SAVE_TWICE,
} from './month-real';

type Input = Record<string, unknown>;

let list: MonthList | null = null;

function current(): MonthList {
  if (list !== null) return list;
  if (devFlag('empty')) {
    list = { rows: [] };
    return list;
  }
  const base = devFlag('closed') ? MONTH_LIST_CLOSED : MONTH_LIST_READY;
  list = devFlag('daysopen')
    ? {
        rows: base.rows.map((row) =>
          row.month === '2026-09' ? { ...row, state: 'days_open' as const } : row,
        ),
      }
    : base;
  return list;
}

function setState(month: string, state: MonthRow['state']): void {
  list = {
    rows: current().rows.map((row) =>
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
}

function monthOf(input: Input): string {
  return typeof input['month'] === 'string' ? input['month'] : '';
}

const handlers: Record<string, (input: Input) => unknown> = {
  month_list: () => current(),
  month_close_preview: (input) => {
    const month = monthOf(input);
    const row = current().rows.find((candidate) => candidate.month === month);
    if (row?.state === 'not_finished') throw refusalOf(MONTH_PREVIEW_NOT_FINISHED);
    if (row?.state === 'days_open') throw refusalOf(MONTH_SAVE_DAYS_OPEN);
    return { ...MONTH_PREVIEW, month };
  },
  month_close_save: (input) => {
    if (devFlag('twice')) throw refusalOf(MONTH_SAVE_TWICE);
    const month = monthOf(input);
    setState(month, 'closed');
    return { ...MONTH_SAVE, month };
  },
  month_close_reverse: (input) => {
    const month = monthOf(input);
    setState(month, 'ready');
    return { ...MONTH_REVERSE, month };
  },
};

/** The error Tauri rejects with for a refusal the backend wrote. */
function refusalOf(body: { errors: unknown; ref: string }): Error {
  return Object.assign(new Error('refused'), { ok: false, errors: body.errors, ref: body.ref });
}

/** What Tauri rejects with for a command this page does not answer, or for the `down` flag. */
function refusal(code: string): Error {
  return Object.assign(new Error(code), {
    ok: false,
    errors: [{ code, field: null, values: {} }],
    ref: 'E-20261010-0001',
  });
}

function inputOf(args: unknown): Input {
  if (typeof args !== 'object' || args === null) return {};
  const inner = (args as Input)['input'];
  return typeof inner === 'object' && inner !== null ? (inner as Input) : {};
}

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installMonthDevBridge(): void {
  list = null;
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      transformCallback: () => 0,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
              return;
            }
            if (devFlag('down')) {
              reject(refusal('ERR_INTERNAL'));
              return;
            }
            try {
              resolve(handler(inputOf(args)));
            } catch (error) {
              reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
            }
          }, 20);
        }),
    },
  });
}
