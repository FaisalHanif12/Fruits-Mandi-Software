// Test helper: sets the answers of the report command wrappers (plan 20) from the backend's own
// answers in reports-real.ts. The wrappers are mocked in the test file with vi.mock. A report is
// answered the way the real engine does it: the rows of the whole filter are sliced into the page
// that was asked for, with the same total and totals on every page. A refusal is a rejected promise
// (`CommandError`), the way `invoke` fails.

import { vi } from 'vitest';
import type { BuyerBillsPrintData } from '../../src/bindings/BuyerBillsPrintData';
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

import { PRINT_R36_PARTNER_EN, R36_GARDEN, R36_PARTNER } from './reports-gardens-real';
import { PRINT_R37_EN, R37_FRUIT, R37_GUESTS } from './reports-fruit-real';
import {
  BUYER_BILLS_EN,
  BUYER_BILLS_UR,
  PRINT_R5_EN,
  R1_BIKRI_1,
  R1_NEEDS_A_BIKRI,
  R5_CREDIT_BUYERS,
} from './reports-sales-real';
import {
  PRINT_R35_SLIP_AKRAM_EN,
  R35_SLIP_AKRAM,
  R35_SLIP_RAHIM,
  R35_TABLE,
} from './reports-party-real';
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

export * from './reports-real';
export * from './reports-business-real';
export * from './reports-stock-real';
export * from './reports-party-real';
export * from './reports-gardens-real';
export * from './reports-sales-real';
export * from './reports-fruit-real';

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
  /** What the slips print of R5 answers; the default is the backend's own answer, in the print language asked for. */
  slips?: BuyerBillsPrintData;
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
  // Round 8: stock, the vouchers list with no kind picked, the two registers, the trip profit or
  // loss by trip and the day summary of E2E-36.
  R10: R10_STOCK,
  R15: R15_ALL,
  R23: R23_DAY,
  R25: R25_DAY,
  R33: R33_TRIP,
  R34: R34_DAY,
  // Round 11: the party day sheet, the table of every party (a slip is answered for a party picked).
  R35: R35_TABLE,
  // The garden slice: the gardens report by garden (by partner when the layout box says so).
  R36: R36_GARDEN,
  // Round 12: the sale book of بکری 1 (it needs a بکری picked) and the credit buyers' details.
  R1: R1_BIKRI_1,
  R5: R5_CREDIT_BUYERS,
  // The fruit slice: the fruit register of E2E-42 (the guests' line only when the box says guests).
  R37: R37_FRUIT,
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
  if (id === 'R33') return PRINT_R33_PARTNER_EN;
  if (id === 'R34') return PRINT_R34_EN;
  if (id === 'R35') return PRINT_R35_SLIP_AKRAM_EN;
  if (id === 'R36') return PRINT_R36_PARTNER_EN;
  if (id === 'R5') return PRINT_R5_EN;
  if (id === 'R37') return PRINT_R37_EN;
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
    // The sale book is the lots of one بکری: with none picked the backend refuses it.
    if (input.id === 'R1' && input.filters.bikri_id === null) {
      return Promise.reject(refusedWith(R1_NEEDS_A_BIKRI));
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
    // R15 for one kind of voucher answers with that kind (the payments), with one total.
    if (input.id === 'R15' && input.filters.voucher_type !== undefined) answer = R15_PAYMENTS;
    // R36 answers in the layout asked for (unless the test brought a page of its own).
    if (input.id === 'R36' && options.pages?.R36 === undefined) {
      answer = input.filters.garden_view === 'partner' ? R36_PARTNER : R36_GARDEN;
    }
    // R37 for the guests only answers with their line (unless the test brought a page of its own).
    if (
      input.id === 'R37' &&
      options.pages?.R37 === undefined &&
      input.filters.guests_only === true
    ) {
      answer = R37_GUESTS;
    }
    // R35 for one party answers with his slip: the seller's slip for a seller's account (the worked
    // day's ids: رحیم is 18 and کریم 19), the buyer's slip for any other (unless the test brought a
    // page of its own).
    if (input.id === 'R35' && input.filters.party !== null && options.pages?.R35 === undefined) {
      answer = input.filters.party.account_id === 18 ? R35_SLIP_RAHIM : R35_SLIP_AKRAM;
    }
    // R33 answers in the layout asked for (unless the test brought a page of its own).
    if (input.id === 'R33' && options.pages?.R33 === undefined) {
      const layouts = {
        trip: R33_TRIP,
        item: R33_ITEM,
        month: R33_MONTH,
        partner: R33_PARTNER,
      } as const;
      const view = input.filters.trip_view ?? 'trip';
      answer = view === 'day' ? R33_MONTH : layouts[view];
    }
    return Promise.resolve(slice(answer, input.offset, input.limit));
  });
  vi.mocked(reportsApi.reportsFindPeople).mockImplementation(() =>
    Promise.resolve([...(options.people ?? PEOPLE_SHAHID)]),
  );
  vi.mocked(reportsApi.reportsPrintData).mockImplementation((input) =>
    Promise.resolve(options.print ?? printOf(input.id)),
  );
  vi.mocked(reportsApi.reportsBuyerBillsPrintData).mockImplementation((input) =>
    Promise.resolve(options.slips ?? (input.language === 'ur' ? BUYER_BILLS_UR : BUYER_BILLS_EN)),
  );
  return { pages };
}
