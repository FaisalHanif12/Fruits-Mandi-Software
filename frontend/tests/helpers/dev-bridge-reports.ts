// A stand-in for the Tauri bridge for the development pages /dev/reports (plan 20) only. In a plain
// browser tab there is no backend, so the report screens cannot be looked at. This answers the
// report commands with the backend's own fixtures (reports-real.ts), so the screens can be
// opened in a browser and photographed in both languages and at every window size. It answers
// nothing else: any other command is refused with ERR_NOT_IMPLEMENTED. Never part of a build: only
// the development route imports it.
//
// Flags in the hash, for example `#/dev/reports/R18?many`:
//   empty   the report has no rows        many   230 rows (page by page)
//   down    the backend fails             rahim  the ledger of a seller (party 18)
//   grade   R2 with the quality column    threecols  R2 to R4 with the quality, packing and seller's rate
//   R35 asks for the table of every party; a party picked (or Enter on a row) gives his slip.

import type { ReportCatalogEntry } from '../../src/bindings/ReportCatalogEntry';
import type { ReportId } from '../../src/bindings/ReportId';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import type { GardensListInput } from '../../src/bindings/GardensListInput';
import type { PrintFrameInput } from '../../src/bindings/PrintFrameInput';
import { devFlag } from './dev-bridge';
import { printFrameFor } from './print-frame-real';
import { devExport, exportPluginCommand, transformCallback } from './dev-bridge-export';
import { rokarHandlers } from './dev-bridge-rokar';
import { listRow } from './bikri-saved';
import { createGardenBooks } from './gardens-data';
import { PRINT_R37_EN, R10_FRUIT, R34_FRUIT, R37_FRUIT, R37_GUESTS } from './reports-fruit-real';
import {
  DAY_SHEET,
  PRINT_DAY_SHEET_EN,
  PRINT_DAY_SHEET_UR,
  PRINT_SLIP_BUYER_EN,
  PRINT_SLIP_BUYER_UR,
  PRINT_SLIP_SELLER_EN,
  PRINT_SLIP_SELLER_UR,
  PRINT_SLIP_WEIGHED_EN,
  PRINT_SLIP_WEIGHED_UR,
  SLIP_BUYER_AKRAM,
  SLIP_BUYER_WEIGHED,
  SLIP_SELLER_RAHIM,
} from './day-slips-real';
import {
  BUYER_BILLS_EN,
  BUYER_BILLS_UR,
  PRINT_R5_EN,
  R1_BIKRI_1,
  R1_NEEDS_A_BIKRI,
  R5_CREDIT_BUYERS,
} from './reports-sales-real';
import {
  PRINT_R36_PARTNER_EN,
  R34_GARDEN_DAY,
  R36_GARDEN,
  R36_PARTNER,
} from './reports-gardens-real';
import { PRINT_R35_TABLE_EN, PRINT_R35_TABLE_UR, R35_TABLE } from './reports-party-real';
import { R2_THREE_COLUMNS } from './register-three-columns';
import { GRADE_R2, GRADE_R35_AKRAM } from './sale-grade-real';
import {
  PRINT_R33_PARTNER_EN,
  R10_STOCK,
  R15_ALL,
  R15_PAYMENTS,
  R23_DAY,
  R25_DAY,
  R33_ITEM,
  R33_MONTH,
  R33_PARTNER,
  R33_TRIP,
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
  PRINT_R20_UR,
  PRINT_R28_MONTH_EN,
  PRINT_R7_EN,
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
} from './reports-real';

type Input = Record<string, unknown>;

/** The gardens of the garden box of R36: the garden world of the gardens module. */
const gardenBooks = createGardenBooks();

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
  // The day total sheet and the day slips are the plan 42 answers (PRD v1.12): the buyers' and the
  // sellers' table under the lines of the day, and a slip with its labelled lines.
  R34: DAY_SHEET,
  R35: R35_TABLE,
  R36: R36_GARDEN,
  R1: R1_BIKRI_1,
  R5: R5_CREDIT_BUYERS,
  R37: R37_FRUIT,
};

const PRINTS: Partial<Record<ReportId, ReportPrintData>> = {
  R7: PRINT_R7_EN,
  R13: PRINT_R13_AKRAM_UR,
  R20: PRINT_R20_UR,
  R28: PRINT_R28_MONTH_EN,
  R31: PRINT_R31_BAY_EN,
  R33: PRINT_R33_PARTNER_EN,
  R34: PRINT_DAY_SHEET_EN,
  R36: PRINT_R36_PARTNER_EN,
  R5: PRINT_R5_EN,
  R37: PRINT_R37_EN,
};

/**
 * With the `blank` flag a slip has nothing paid yet: the backend sends the paid-today line EMPTY, and
 * the page leaves a line for the collector to write on (PRD FR-19.12).
 */
function blankPaid<T extends { summary: { label_id: string; cell: unknown }[] }>(slip: T): T {
  if (!devFlag('blank')) return slip;
  return {
    ...slip,
    summary: slip.summary.map((line) =>
      line.label_id === 'report-r35-paid-today' ? { ...line, cell: { type: 'empty' } } : line,
    ),
  };
}

/** R35 prints in the language asked for: the table for no party, a slip for one (a seller's for رحیم). */
function partyDaySheetPrint(input: Input): ReportPrintData {
  const filters = (input['filters'] ?? {}) as Record<string, unknown>;
  const party = filters['party'] as { account_id?: number } | null;
  const urdu = input['language'] === 'ur';
  if (party?.account_id === undefined) return urdu ? PRINT_R35_TABLE_UR : PRINT_R35_TABLE_EN;
  if (party.account_id === 18) {
    return blankPaid(urdu ? PRINT_SLIP_SELLER_UR : PRINT_SLIP_SELLER_EN);
  }
  // With the `weighed` flag the buyer's slip has a line of an item sold by weight (PRD A73).
  if (devFlag('weighed')) return urdu ? PRINT_SLIP_WEIGHED_UR : PRINT_SLIP_WEIGHED_EN;
  return blankPaid(urdu ? PRINT_SLIP_BUYER_UR : PRINT_SLIP_BUYER_EN);
}

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
  // The sale book needs a بکری picked, as the backend refuses it.
  if (id === 'R1' && (filters['bikri_id'] ?? null) === null) {
    throw Object.assign(new Error('ERR_REQUIRED'), R1_NEEDS_A_BIKRI);
  }
  // R36 answers in the layout asked for; the day summary shows the garden world with the `garden` flag.
  if (id === 'R36' && filters['garden_view'] === 'partner') base = R36_PARTNER;
  if (id === 'R34' && devFlag('garden')) base = R34_GARDEN_DAY;
  // The stock and the day summary show the fruit world of E2E-42 with the `fruit` flag.
  if (id === 'R10' && devFlag('fruit')) base = R10_FRUIT;
  if (id === 'R34' && devFlag('fruit')) base = R34_FRUIT;
  // The fruit register for the guests only answers with their line.
  if (id === 'R37' && filters['guests_only'] === true) base = R37_GUESTS;
  // R35 for one party answers with his slip: the seller's for a seller (رحیم is 18), the buyer's
  // for any other account.
  if (id === 'R35' && party?.account_id !== undefined) {
    base =
      devFlag('weighed') && party.account_id !== 18
        ? SLIP_BUYER_WEIGHED
        : blankPaid(party.account_id === 18 ? SLIP_SELLER_RAHIM : SLIP_BUYER_AKRAM);
  }
  // `?grade` shows the real answers of plan 58: lines with a quality, a grade column in R2 and R35.
  if (devFlag('grade') && id === 'R2') base = GRADE_R2;
  if (devFlag('grade') && id === 'R35' && party?.account_id !== undefined) base = GRADE_R35_AKRAM;
  // `?threecols` shows R2, R3 and R4 with the quality, the packing and the seller's rate together (plan 59).
  if (devFlag('threecols') && (id === 'R2' || id === 'R3' || id === 'R4')) {
    base = R2_THREE_COLUMNS;
  }
  // The real fixtures written before plan 42 have no `tables`; the backend now always sends the field
  // (empty for every report but the day total sheet). Until the backend writes them again, say so here.
  base = { ...base, tables: (base as Partial<ReportPage>).tables ?? [] };
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
  reports_print_data: (input) => {
    // The backend echoes the paper it was asked for, so the preview is drawn on that paper.
    const onPaper = (data: ReportPrintData): ReportPrintData => ({
      ...data,
      paper: (input['paper'] as ReportPrintData['paper'] | undefined) ?? data.paper,
    });
    if (input['id'] === 'R35') return onPaper(partyDaySheetPrint(input));
    if (input['id'] === 'R34' && !devFlag('garden') && !devFlag('fruit')) {
      return onPaper(input['language'] === 'ur' ? PRINT_DAY_SHEET_UR : PRINT_DAY_SHEET_EN);
    }
    return PRINTS[input['id'] as ReportId] ?? PRINT_R18_EN;
  },
  // The shared header and footer of every report print: the firm's own, the title and facts asked for.
  reports_print_header: (input) => printFrameFor(input as unknown as PrintFrameInput),
  reports_buyer_bills_print_data: (input) =>
    input['language'] === 'ur' ? BUYER_BILLS_UR : BUYER_BILLS_EN,
  // The partner box (R29, R31) and the person box of R32 (the people who can borrow) read these
  // from the cash book stand-in, so the filter boxes of the new reports work in the browser.
  partners_list: (input) => rokarHandlers['partners_list']?.(input),
  gardens_list: (input) => gardenBooks.list(input as GardensListInput),
  // The box of the sale book report (R1) finds a بکری by its number: the worked day's بکری 1.
  bikri_find: () => [listRow({ id: 1, number: 1 })],
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
export function installReportsDevBridge(
  /** More commands for a page of another module that borrows this stand-in (the party ledgers, plan 66). */
  extra: Readonly<Record<string, (input: Input) => unknown>> = {},
): void {
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
            const handler = extra[command] ?? handlers[command];
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
