// A stand-in for the Tauri bridge for the development page /dev/kharch (plan 77) only. In a plain browser tab there is
// no backend, so the kharch page cannot be looked at. This answers its commands with the backend's own answers of the
// analytics view M (worked day, October 2026: the shop's بجلی 3,000; the partners' own kharch الف 2,000, ب 0, ج 0),
// the entries behind a figure, and the partner list of the partner filter. Any other command is refused with
// ERR_NOT_IMPLEMENTED. Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/kharch?locked`:
//   locked   the partner part is refused (the partner code is not entered)   empty   no kharch at all   down   it fails

import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import {
  ANALYTICS_LINES_M_ELECTRICITY,
  ANALYTICS_LINES_M_PARTNER_ALIF,
  ANALYTICS_RUN_M,
  ANALYTICS_RUN_M_BY_DAY,
  ANALYTICS_RUN_M_PARTNERS,
} from './analytics-plan77';
import { dashboardHandlers, inputOf, refusal } from './dev-bridge-dashboard';
import { devFlag } from './dev-bridge';
import { rokarHandlers } from './dev-bridge-rokar';

type Input = Record<string, unknown>;

function filtersOf(input: Input): Input {
  const filters = input['filters'];
  return typeof filters === 'object' && filters !== null ? (filters as Input) : {};
}

function run(input: Input): AnalyticsPage {
  if (devFlag('down')) throw refusal('ERR_INTERNAL');
  const filters = filtersOf(input);
  const partner = filters['expense_part'] === 'partner';
  if (partner && devFlag('locked')) throw refusal('ERR_PARTNER_LOCKED');
  let base = partner
    ? ANALYTICS_RUN_M_PARTNERS
    : filters['group_by'] === 'day'
      ? ANALYTICS_RUN_M_BY_DAY
      : ANALYTICS_RUN_M;
  // A partner chosen narrows the partner part to his row (the backend does the same; plan 98). The stand-in only
  // picks the row and its figure: it adds nothing up.
  const wanted = filters['partner_id'];
  if (partner && typeof wanted === 'number') {
    const rows = base.rows.filter((row) => row.key?.partner_id === wanted);
    const own = rows[0]?.cells[1];
    base = {
      ...base,
      rows,
      total_count: rows.length,
      totals: base.totals.map((cell, index) => (index === 1 && own !== undefined ? own : cell)),
    };
  }
  return devFlag('empty') ? { ...base, rows: [], total_count: 0 } : base;
}

const handlers: Record<string, (input: Input) => unknown> = {
  ...dashboardHandlers,
  analytics_run: run,
  analytics_lines: (input) =>
    filtersOf(input)['expense_part'] === 'partner'
      ? ANALYTICS_LINES_M_PARTNER_ALIF
      : ANALYTICS_LINES_M_ELECTRICITY,
  // The partner box reads the partners of the Rokar stand-in.
  partners_list: (input) => rokarHandlers['partners_list']?.(input),
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installKharchDevBridge(): void {
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
