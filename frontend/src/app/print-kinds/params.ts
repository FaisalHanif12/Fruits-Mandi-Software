// The address of the hidden print window of the prints of plan 78: `#/print?print=<kind>&language=<ur|en>&paper=
// <a4|a5|mm80>&input=<percent-encoded JSON>&job=<job>`. The backend opens the window with this address, waits
// for `print-ready-<job>` and then prints it or saves it as a PDF. `input` is the exact JSON input of the
// matching print-data command (it carries the language and the paper too); the command checks it again (Code
// Standards V0), so this only checks that it is a JSON object. A value that is not right is a failed print,
// never a guess. The job name is checked by the same function as every other print window's.

import { z } from 'zod';
import type { AdhariSheetPrintInput } from '../../bindings/AdhariSheetPrintInput';
import type { AdvancesReportPrintInput } from '../../bindings/AdvancesReportPrintInput';
import type { CapitalPrintInput } from '../../bindings/CapitalPrintInput';
import type { BuyerDuesPrintInput } from '../../bindings/BuyerDuesPrintInput';
import type { CashSalesDayPrintInput } from '../../bindings/CashSalesDayPrintInput';
import type { CashSalesMonthPrintInput } from '../../bindings/CashSalesMonthPrintInput';
import type { CollectionSheetPrintInput } from '../../bindings/CollectionSheetPrintInput';
import type { CommissionPrintInput } from '../../bindings/CommissionPrintInput';
import type { DaySlipsInput } from '../../bindings/DaySlipsInput';
import type { LentOpenPrintInput } from '../../bindings/LentOpenPrintInput';
import type { PartnersStatementMonthPrintInput } from '../../bindings/PartnersStatementMonthPrintInput';
import type { SellerDuesPrintInput } from '../../bindings/SellerDuesPrintInput';
import type { SalesPrintInput } from '../../bindings/SalesPrintInput';
import type { StatementRowsPrintInput } from '../../bindings/StatementRowsPrintInput';
import type { TotalDuesPrintInput } from '../../bindings/TotalDuesPrintInput';
import { jobOf } from '../../shared/print';

/** The kinds of print the window knows, each with its own print-data command (the table in KindPrintPage). */
export const PRINT_KINDS = [
  'parchi',
  'seller-bills',
  'sale',
  'adhari',
  'collection-sheet',
  'statement-rows',
  'buyer-dues',
  'commission',
  'partner-month',
  // Plan 79: the pages behind the dashboard tiles.
  'seller-dues',
  'total-dues',
  'lent-open',
  'advances-open',
  'cash-sales-month',
  'cash-sales-day',
  // Plan 83: the sales page behind the tile آج کی بکری.
  'sales',
  // Plan 87: the capital card of the روکڑ page.
  'capital',
] as const;

export type PrintKind = (typeof PRINT_KINDS)[number];

const isObject = (value: unknown): boolean =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

// The input of each kind is the JSON of its own command, a JSON object. The command checks every field again
// (Code Standards V0), so a wrong value is refused there and the print fails, never prints something else.
const daySlipsInput = z.custom<DaySlipsInput>(isObject);
const adhariInput = z.custom<AdhariSheetPrintInput>(isObject);
const collectionSheetInput = z.custom<CollectionSheetPrintInput>(isObject);
const statementRowsInput = z.custom<StatementRowsPrintInput>(isObject);
const buyerDuesInput = z.custom<BuyerDuesPrintInput>(isObject);
const commissionInput = z.custom<CommissionPrintInput>(isObject);
const partnerMonthInput = z.custom<PartnersStatementMonthPrintInput>(isObject);
const sellerDuesInput = z.custom<SellerDuesPrintInput>(isObject);
const totalDuesInput = z.custom<TotalDuesPrintInput>(isObject);
const lentOpenInput = z.custom<LentOpenPrintInput>(isObject);
const advancesOpenInput = z.custom<AdvancesReportPrintInput>(isObject);
const cashSalesMonthInput = z.custom<CashSalesMonthPrintInput>(isObject);
const cashSalesDayInput = z.custom<CashSalesDayPrintInput>(isObject);
const salesPeriodInput = z.custom<SalesPrintInput>(isObject);
const capitalInput = z.custom<CapitalPrintInput>(isObject);

/** The parameters of the print window: the kind and the input of its command, typed. */
export type KindPrintParams = {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
} & (
  | { kind: 'parchi' | 'seller-bills' | 'sale'; input: DaySlipsInput }
  | { kind: 'adhari'; input: AdhariSheetPrintInput }
  | { kind: 'collection-sheet'; input: CollectionSheetPrintInput }
  | { kind: 'statement-rows'; input: StatementRowsPrintInput }
  | { kind: 'buyer-dues'; input: BuyerDuesPrintInput }
  | { kind: 'commission'; input: CommissionPrintInput }
  | { kind: 'partner-month'; input: PartnersStatementMonthPrintInput }
  | { kind: 'seller-dues'; input: SellerDuesPrintInput }
  | { kind: 'total-dues'; input: TotalDuesPrintInput }
  | { kind: 'lent-open'; input: LentOpenPrintInput }
  | { kind: 'advances-open'; input: AdvancesReportPrintInput }
  | { kind: 'cash-sales-month'; input: CashSalesMonthPrintInput }
  | { kind: 'cash-sales-day'; input: CashSalesDayPrintInput }
  | { kind: 'sales'; input: SalesPrintInput }
  | { kind: 'capital'; input: CapitalPrintInput }
);

/** True when the address is for one of these prints (it has a `print` value), even if the rest is wrong. */
export function isKindAddress(searchStr: string): boolean {
  return new URLSearchParams(searchStr).has('print');
}

/** The parameters of the print window for a kind, or null when the address is not valid. */
export function parseKindPrintParams(searchStr: string): KindPrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const kind = PRINT_KINDS.find((candidate) => candidate === search.get('print'));
  const text = search.get('input');
  if (job === null || kind === undefined || text === null) return null;
  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    return null;
  }
  switch (kind) {
    case 'parchi':
    case 'seller-bills':
    case 'sale': {
      const input = daySlipsInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'adhari': {
      const input = adhariInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'collection-sheet': {
      const input = collectionSheetInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'statement-rows': {
      const input = statementRowsInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'buyer-dues': {
      const input = buyerDuesInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'commission': {
      const input = commissionInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'partner-month': {
      const input = partnerMonthInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'seller-dues': {
      const input = sellerDuesInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'total-dues': {
      const input = totalDuesInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'lent-open': {
      const input = lentOpenInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'advances-open': {
      const input = advancesOpenInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'cash-sales-month': {
      const input = cashSalesMonthInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'cash-sales-day': {
      const input = cashSalesDayInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'sales': {
      const input = salesPeriodInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
    case 'capital': {
      const input = capitalInput.safeParse(json);
      return input.success ? { job, kind, input: input.data } : null;
    }
  }
}
