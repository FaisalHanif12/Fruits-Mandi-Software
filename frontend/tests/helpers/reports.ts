// Test helper: sets the answers of the report command wrappers (plan 20) from the backend's own
// answers in reports-real.ts. The wrappers are mocked in the test file with vi.mock. A report is
// answered the way the real engine does it: the rows of the whole filter are sliced into the page
// that was asked for, with the same total and totals on every page. A refusal is a rejected promise
// (`CommandError`), the way `invoke` fails.

import { vi } from 'vitest';
import type { PukhtaPersonHit } from '../../src/bindings/PukhtaPersonHit';
import type { ReportCatalogEntry } from '../../src/bindings/ReportCatalogEntry';
import type { ReportId } from '../../src/bindings/ReportId';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { ReportRow } from '../../src/bindings/ReportRow';
import type { UiError } from '../../src/bindings/UiError';
import * as reportsApi from '../../src/features/reports/api';
import { CommandError } from '../../src/shared/api';
import {
  CATALOG,
  PRINT_R13_AKRAM_UR,
  PRINT_R18_EN,
  R11_TODAY,
  R13_AKRAM,
  R14_BALANCE,
  R14_DETAIL,
  R16_PAGE1,
  R17,
  R18_AS_OF,
  R18_FULL,
  R19,
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
  PEOPLE_SHAHID,
} from './reports-real';
import {
  PRINT_R31_BAY_EN,
  R12_DAY,
  R29_ALIF,
  R29_DAY,
  R31_BAY,
  R32_E2E_30B,
} from './reports-business-real';

export * from './reports-real';
export * from './reports-business-real';

/** A backend refusal, as a rejected promise. */
export function refusedWith(error: UiError): CommandError {
  return new CommandError(error);
}

export interface ReportsOptions {
  catalog?: readonly ReportCatalogEntry[] | Error;
  /** Pages by report id; the default is the backend's own answer for each. */
  pages?: Partial<Record<ReportId, ReportPage>>;
  /** Called for every `reports_run`; return a refusal to refuse that run. */
  refuse?: (input: Parameters<typeof reportsApi.reportsRun>[0]) => UiError | null;
  print?: ReportPrintData;
  /** What the people search answers; the default is the backend's own answer for "shahid". */
  people?: readonly PukhtaPersonHit[];
}

const DEFAULT_PAGES: Partial<Record<ReportId, ReportPage>> = {
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
  // Round 7: the cash book, the expense report (all partners, one day), the statement of partner ب
  // and the money lent report.
  R12: R12_DAY,
  R29: R29_DAY,
  R31: R31_BAY,
  R32: R32_E2E_30B,
};

function slice(page: ReportPage, offset: number, limit: number): ReportPage {
  const size = limit > 0 ? limit : 100;
  const rows: ReportRow[] = page.rows.slice(offset, offset + size);
  return { ...page, rows, offset, limit };
}

/** The print the backend wrote for a report; the one of R18 for any report it wrote none for. */
function printOf(id: ReportId): ReportPrintData {
  if (id === 'R13') return PRINT_R13_AKRAM_UR;
  if (id === 'R31') return PRINT_R31_BAY_EN;
  return PRINT_R18_EN;
}

export interface FakeReports {
  pages: Partial<Record<ReportId, ReportPage>>;
}

/** Sets the answers of the report commands, and returns the pages behind them. */
export function fakeReports(options: ReportsOptions = {}): FakeReports {
  const pages: Partial<Record<ReportId, ReportPage>> = { ...DEFAULT_PAGES, ...options.pages };
  vi.mocked(reportsApi.reportsCatalog).mockImplementation(() =>
    options.catalog instanceof Error
      ? Promise.reject(options.catalog)
      : Promise.resolve([...(options.catalog ?? CATALOG)]),
  );
  vi.mocked(reportsApi.reportsRun).mockImplementation((input) => {
    const refused = options.refuse?.(input) ?? null;
    if (refused !== null) return Promise.reject(refusedWith(refused));
    const page = pages[input.id];
    // A report with no canned answer is not found, the way the backend answers an unknown one.
    if (page === undefined) {
      return Promise.reject(
        refusedWith({
          ok: false,
          errors: [{ code: 'ERR_NOT_FOUND', field: 'id', values: {} }],
          ref: 'E-1',
        }),
      );
    }
    // The as-of day of the backend's fixture is the 9th: a filled as-of box answers with it.
    const base = input.id === 'R18' && input.filters.as_of !== '' ? R18_AS_OF : page;
    // R14 answers with its lines only when the detail box is ticked.
    let answer = input.id === 'R14' && input.filters.detail ? R14_DETAIL : base;
    // R9 answers with the due lines only when asked for them, and R28 by day when asked by day.
    if (input.id === 'R9' && input.filters.status === 'due') answer = R9_DUE;
    if (input.id === 'R28' && input.filters.group_by === 'day') answer = R28_BY_DAY;
    // R29 for one partner answers with his lines only.
    if (input.id === 'R29' && input.filters.partner_id !== null) answer = R29_ALIF;
    return Promise.resolve(slice(answer, input.offset, input.limit));
  });
  vi.mocked(reportsApi.reportsFindPeople).mockImplementation(() =>
    Promise.resolve([...(options.people ?? PEOPLE_SHAHID)]),
  );
  vi.mocked(reportsApi.reportsPrintData).mockImplementation((input) =>
    Promise.resolve(options.print ?? printOf(input.id)),
  );
  return { pages };
}
