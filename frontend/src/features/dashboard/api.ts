// Typed wrappers around the dashboard and attention commands (plan 19). Both are read only: they
// write nothing and work out nothing the screens could not read (Code Standards B29).

import type { AttentionList } from '../../bindings/AttentionList';
import type { BuyerDuesInput } from '../../bindings/BuyerDuesInput';
import type { BuyerDuesPage } from '../../bindings/BuyerDuesPage';
import type { BuyerDuesPrintInput } from '../../bindings/BuyerDuesPrintInput';
import type { CommissionDay } from '../../bindings/CommissionDay';
import type { CommissionDayInput } from '../../bindings/CommissionDayInput';
import type { CommissionMonth } from '../../bindings/CommissionMonth';
import type { CommissionMonthInput } from '../../bindings/CommissionMonthInput';
import type { CommissionPrintInput } from '../../bindings/CommissionPrintInput';
import type { Dashboard } from '../../bindings/Dashboard';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import type { PrintTo } from '../../bindings/PrintTo';
import type { TablePrintData } from '../../bindings/TablePrintData';
import { callCommand } from '../../shared/api';

/** Everything the dashboard shows (PRD 9), in one call. One failure fails the whole answer. */
export function dashboardGet(): Promise<Dashboard> {
  return callCommand<Dashboard>('dashboard_get');
}

/** The attention list and the number on the bell (PRD FR-7.44). */
export function assistAttention(): Promise<AttentionList> {
  return callCommand<AttentionList>('assist_attention');
}

/**
 * A page of the buyers who owe, with the count and the total of all that match (PRD FR-9.4, FR-19.11). The
 * total is the tile's number and the R20 total. Read only.
 */
export function dashboardBuyerDues(input: BuyerDuesInput): Promise<BuyerDuesPage> {
  return callCommand<BuyerDuesPage>('dashboard_buyer_dues', { input });
}

/**
 * One month of the commission page: a row for each day that has a figure, and the totals of the month
 * (PRD FR-9.3). The partner columns are empty while the partner section is locked. Read only.
 */
export function dashboardCommissionMonth(input: CommissionMonthInput): Promise<CommissionMonth> {
  return callCommand<CommissionMonth>('dashboard_commission_month', { input });
}

/** The postings behind one day's commission (PRD FR-9.3). Read only. */
export function dashboardCommissionDay(input: CommissionDayInput): Promise<CommissionDay> {
  return callCommand<CommissionDay>('dashboard_commission_day', { input });
}

/** Every buyer who owes, written in the print language for the preview and the print (PRD FR-9.4, FR-19.13). */
export function dashboardBuyerDuesPrintData(input: BuyerDuesPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_buyer_dues_print_data', { input });
}

/** Prints the buyer dues, or saves them as a PDF: the same search, order, language and paper as the preview. */
export function dashboardBuyerDuesPrint(input: {
  dues: BuyerDuesPrintInput;
  to: PrintTo;
  copies: string | null;
}): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_buyer_dues_print', { input });
}

/** One month of commission, every day of it, in the print language (PRD FR-9.3, FR-19.3). */
export function dashboardCommissionPrintData(input: CommissionPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_commission_print_data', { input });
}

/** Prints the month of commission, or saves it as a PDF: the same month, language and paper as the preview. */
export function dashboardCommissionPrint(input: {
  commission: CommissionPrintInput;
  to: PrintTo;
  copies: string | null;
}): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_commission_print', { input });
}
