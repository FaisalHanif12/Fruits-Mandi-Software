// A stand-in for the Tauri bridge for the development pages /dev/reports (plan 20) only. In a plain
// browser tab there is no backend, so the report screens cannot be looked at. This answers the
// report commands (and the people search) with the backend's own fixtures (reports-real.ts), so the screens can be
// opened in a browser and photographed in both languages and at every window size. It answers
// nothing else: any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only
// the development route imports it.
//
// Flags in the hash, for example `#/dev/reports/R18?many`:
//   empty   the report has no rows        many   230 rows (page by page)
//   down    the backend fails             rahim  the ledger of a seller (party 18)

import type { ReportCatalogEntry } from '../../src/bindings/ReportCatalogEntry';
import type { ReportId } from '../../src/bindings/ReportId';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { devFlag } from './dev-bridge';
import { devExport, exportPluginCommand, transformCallback } from './dev-bridge-export';
import { rokarHandlers } from './dev-bridge-rokar';
import {
  PRINT_R33_PARTNER_EN,
  PRINT_R34_EN,
  R10_STOCK,
  R15_ALL,
  R15_PAYMENTS,
  R23_DAY,
  R25_DAY,
  R33_ITEM,
  R33_MONTH,
  R33_PARTNER,
  R33_TRIP,
  R34_DAY,
} from './reports-stock-real';
import {
  PRINT_R31_BAY_EN,
  R12_DAY,
  R29_ALIF,
  R29_DAY,
  R31_BAY,
  R31_PARTNER_REQUIRED,
  R32_E2E_30B,
} from './reports-business-real';
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
  PEOPLE_SHAHID,
  PRINT_R20_UR,
  PRINT_R28_MONTH_EN,
  PRINT_R7_EN,
  PRINT_R9_DUE_EN,
  R2_ALL,
  R20_DUES,
  R21_BY_CITY,
  R22_BY_COLLECTOR,
  R24_PAYABLES,
  R26_ADVANCES,
  R27_SERIES,
  R28_BY_DAY,
  R28_BY_MONTH,
  R3_CASH,
  R30_PROFIT,
  R4_CREDIT,
  R6_LOTS,
  R7_ITEMS,
  R8_ADDA,
  R9_ALL,
  R9_DUE,
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
  R2: R2_ALL,
  R3: R3_CASH,
  R4: R4_CREDIT,
  R6: R6_LOTS,
  R7: R7_ITEMS,
  R8: R8_ADDA,
  R9: R9_ALL,
  R20: R20_DUES,
  R21: R21_BY_CITY,
  R22: R22_BY_COLLECTOR,
  R24: R24_PAYABLES,
  R26: R26_ADVANCES,
  R27: R27_SERIES,
  R28: R28_BY_MONTH,
  R30: R30_PROFIT,
  R12: R12_DAY,
  R29: R29_DAY,
  R31: R31_BAY,
  R32: R32_E2E_30B,
  R10: R10_STOCK,
  R15: R15_ALL,
  R23: R23_DAY,
  R25: R25_DAY,
  R33: R33_TRIP,
  R34: R34_DAY,
};

const PRINTS: Partial<Record<ReportId, ReportPrintData>> = {
  R7: PRINT_R7_EN,
  R9: PRINT_R9_DUE_EN,
  R13: PRINT_R13_AKRAM_UR,
  R20: PRINT_R20_UR,
  R28: PRINT_R28_MONTH_EN,
  R31: PRINT_R31_BAY_EN,
  R33: PRINT_R33_PARTNER_EN,
  R34: PRINT_R34_EN,
};

function catalog(): ReportCatalogEntry[] {
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
  if (id === 'R9' && filters['status'] === 'due') base = R9_DUE;
  if (id === 'R28' && filters['group_by'] === 'day') base = R28_BY_DAY;
  // The partner statement cannot run without a partner, as the backend refuses it.
  if (id === 'R31' && (filters['partner_id'] ?? null) === null) {
    throw Object.assign(new Error('ERR_REQUIRED'), R31_PARTNER_REQUIRED);
  }
  if (id === 'R29' && (filters['partner_id'] ?? null) !== null) base = R29_ALIF;
  // R15 for one kind of voucher answers with that kind, with one total.
  if (id === 'R15' && (filters['voucher_type'] ?? null) !== null) base = R15_PAYMENTS;
  // R33 answers in the layout asked for (the day layout has no fixture of its own: the month one).
  if (id === 'R33') {
    const view = filters['trip_view'];
    if (view === 'item') base = R33_ITEM;
    else if (view === 'partner') base = R33_PARTNER;
    else if (view === 'month' || view === 'day') base = R33_MONTH;
  }
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
  reports_export_excel: (input) => devExport(input),
  reports_catalog: () => catalog(),
  reports_run: (input) => page(input),
  reports_print_data: (input) => PRINTS[input['id'] as ReportId] ?? PRINT_R18_EN,
  reports_find_people: () => PEOPLE_SHAHID,
  // The partner box (R29, R31) and the person box of R32 (the people who can borrow) read these
  // from the cash book stand-in, so the filter boxes of the new reports work in the browser.
  partners_list: (input) => rokarHandlers['partners_list']?.(input),
  coa_controls_list: (input) => rokarHandlers['coa_controls_list']?.(input),
  search_accounts: (input) => rokarHandlers['search_accounts']?.(input),
  search_accounts_recent: (input) => rokarHandlers['search_accounts_recent']?.(input),
  search_accounts_touch: () => null,
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
      transformCallback,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const plugin = exportPluginCommand(command, args);
            if (plugin.handled) {
              resolve(plugin.value);
              return;
            }
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
