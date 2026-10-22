// A stand-in for the Tauri bridge for the development pages /dev/reports (plan 20) only. In a plain
// browser tab there is no backend, so the report screens cannot be looked at. This answers the
// three report commands with the backend's own fixtures (reports-real.ts), so the screens can be
// opened in a browser and photographed in both languages and at every window size. It answers
// nothing else: any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only
// the development route imports it.
//
// Flags in the hash, for example `#/dev/reports/R18?many`:
//   empty   the report has no rows        many   230 rows (page by page)
//   down    the backend fails             rahim  the ledger of a seller (party 18)
//   catalog the catalogue also lists a Sales report (shows the grouping)

import type { ReportCatalogEntry } from '../../src/bindings/ReportCatalogEntry';
import type { ReportId } from '../../src/bindings/ReportId';
import type { ReportPage } from '../../src/bindings/ReportPage';
import { devFlag } from './dev-bridge';
import {
  CATALOG,
  PRINT_R13_AKRAM_UR,
  PRINT_R18_EN,
  R11_TODAY,
  R13_AKRAM,
  R13_RAHIM,
  R14_BALANCE,
  R14_DETAIL,
  R16_PAGE1,
  R17,
  R18_AS_OF,
  R18_FULL,
  R19,
} from './reports-real';

type Input = Record<string, unknown>;

const PAGES: Partial<Record<ReportId, ReportPage>> = {
  R11: R11_TODAY,
  R13: R13_AKRAM,
  R14: R14_BALANCE,
  R16: R16_PAGE1,
  R17,
  R18: R18_FULL,
  R19,
};

function catalog(): ReportCatalogEntry[] {
  const first = CATALOG[0];
  if (devFlag('catalog') && first !== undefined) {
    return [
      ...CATALOG,
      { ...first, id: 'R2', group: 'sales', group_title_id: 'report-group-sales' },
    ];
  }
  return CATALOG;
}

function page(input: Input): ReportPage {
  const id = input['id'] as ReportId;
  const filters = (input['filters'] ?? {}) as Record<string, unknown>;
  const party = filters['party'] as { account_id?: number } | null;
  let base = PAGES[id];
  if (base === undefined) throw refusal('ERR_NOT_FOUND');
  if (id === 'R13' && party?.account_id === 18) base = R13_RAHIM;
  if (id === 'R14' && filters['detail'] === true) base = R14_DETAIL;
  if (id === 'R18' && filters['as_of'] !== '') base = R18_AS_OF;
  const firstRow = base.rows[0];
  if (devFlag('many') && firstRow !== undefined) {
    base = { ...base, rows: Array.from({ length: 230 }, () => firstRow), total_count: 230 };
  }
  if (devFlag('empty')) base = { ...base, rows: [], total_count: 0, summary: [] };
  const offset = Number(input['offset'] ?? 0);
  const limit = Number(input['limit'] ?? 100) || 100;
  return { ...base, rows: base.rows.slice(offset, offset + limit), offset, limit };
}

const handlers: Record<string, (input: Input) => unknown> = {
  reports_catalog: () => catalog(),
  reports_run: (input) => page(input),
  reports_print_data: (input) => (input['id'] === 'R13' ? PRINT_R13_AKRAM_UR : PRINT_R18_EN),
};

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
export function installReportsDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
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
            resolve(handler(inputOf(args)));
          }, 20);
        }),
    },
  });
}
