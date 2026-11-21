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
import type { AdvancesReport } from '../../bindings/AdvancesReport';
import type { AdvancesReportInput } from '../../bindings/AdvancesReportInput';
import type { CashSalesDay } from '../../bindings/CashSalesDay';
import type { CashSalesDayInput } from '../../bindings/CashSalesDayInput';
import type { CashSalesMonth } from '../../bindings/CashSalesMonth';
import type { CashSalesMonthInput } from '../../bindings/CashSalesMonthInput';
import type { DeadAmountInput } from '../../bindings/DeadAmountInput';
import type { DeadAmountPage } from '../../bindings/DeadAmountPage';
import type { Dashboard } from '../../bindings/Dashboard';
import type { LentOpenInput } from '../../bindings/LentOpenInput';
import type { LentOpenPage } from '../../bindings/LentOpenPage';
import type { PrintOutcome } from '../../bindings/PrintOutcome';
import type { PrintTo } from '../../bindings/PrintTo';
import type { SellerDuesInput } from '../../bindings/SellerDuesInput';
import type { SellerDuesPage } from '../../bindings/SellerDuesPage';
import type { TablePrintData } from '../../bindings/TablePrintData';
import type { TotalDues } from '../../bindings/TotalDues';
import type { AdvancesReportPrintInput } from '../../bindings/AdvancesReportPrintInput';
import type { CashSalesDayPrintInput } from '../../bindings/CashSalesDayPrintInput';
import type { CashSalesMonthPrintInput } from '../../bindings/CashSalesMonthPrintInput';
import type { LentOpenPrintInput } from '../../bindings/LentOpenPrintInput';
import type { PrintAdvancesOpenInput } from '../../bindings/PrintAdvancesOpenInput';
import type { PrintCashSalesDayInput } from '../../bindings/PrintCashSalesDayInput';
import type { PrintCashSalesMonthInput } from '../../bindings/PrintCashSalesMonthInput';
import type { PrintLentOpenInput } from '../../bindings/PrintLentOpenInput';
import type { PrintSellerDuesInput } from '../../bindings/PrintSellerDuesInput';
import type { PrintTotalDuesInput } from '../../bindings/PrintTotalDuesInput';
import type { SellerDuesPrintInput } from '../../bindings/SellerDuesPrintInput';
import type { TotalDuesPrintInput } from '../../bindings/TotalDuesPrintInput';
import { callCommand } from '../../shared/api';
import type { PrintSalesInput } from '../../bindings/PrintSalesInput';
import type { SalesDay } from '../../bindings/SalesDay';
import type { SalesDayInput } from '../../bindings/SalesDayInput';
import type { SalesPeriod } from '../../bindings/SalesPeriod';
import type { SalesPeriodInput } from '../../bindings/SalesPeriodInput';
import type { SalesPrintInput } from '../../bindings/SalesPrintInput';

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

/**
 * The days (or the months of a year) of a period with sales, for the filters of the sales page, and the totals over
 * the whole filter (PRD FR-9.9, FR-19.11, plan 83). Read only.
 */
export function dashboardSalesPeriod(input: SalesPeriodInput): Promise<SalesPeriod> {
  return callCommand<SalesPeriod>('dashboard_sales_period', { input });
}

/** The بکری posted on one day of the sales page, in the same filters, with the totals of that day (PRD FR-9.9). Read only. */
export function dashboardSalesDay(input: SalesDayInput): Promise<SalesDay> {
  return callCommand<SalesDay>('dashboard_sales_day', { input });
}

/** Every row of the period of sales on screen, written in the print language for the preview (PRD FR-9.9, FR-19.3). */
export function dashboardSalesPrintData(input: SalesPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_sales_print_data', { input });
}

/** Prints the period of sales, or saves it as a PDF: the same period, filters, language and paper as the preview. */
export function dashboardSalesPrint(input: PrintSalesInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_sales_print', { input });
}

/**
 * A page of the sellers with something open, with the count and the three totals of the party and search
 * match: what the shop owes sellers, what sellers owe the shop, and the open advances (PRD FR-9.6, FR-19.11).
 * The side totals are the tiles' numbers. Read only.
 */
export function dashboardSellerDues(input: SellerDuesInput): Promise<SellerDuesPage> {
  return callCommand<SellerDuesPage>('dashboard_seller_dues', { input });
}

/** The four parts of کل اگراہی, each with its count and its total, and the grand total (PRD FR-9.7). Read only. */
export function dashboardTotalDues(): Promise<TotalDues> {
  return callCommand<TotalDues>('dashboard_total_dues');
}

/** A page of the people who borrowed and still owe, with the count and the totals (PRD FR-9.1, FR-16.5). Read only. */
export function dashboardLentOpen(input: LentOpenInput): Promise<LentOpenPage> {
  return callCommand<LentOpenPage>('dashboard_lent_open', { input });
}

/** One month of نقدی: a row for each day with money paid on the spot, and the month's total (PRD FR-9.8). Read only. */
export function dashboardCashSalesMonth(input: CashSalesMonthInput): Promise<CashSalesMonth> {
  return callCommand<CashSalesMonth>('dashboard_cash_sales_month', { input });
}

/** The buyers who paid something on the spot on one day, with the day's totals (PRD FR-9.8). Read only. */
export function dashboardCashSalesDay(input: CashSalesDayInput): Promise<CashSalesDay> {
  return callCommand<CashSalesDay>('dashboard_cash_sales_day', { input });
}

/**
 * A page of the sellers with an open advance (the page behind the tile پیشگی باقی, PRD FR-9.1): what was given,
 * returned, adjusted and what is open, with the totals of the match. `only_open` asks for the sellers whose
 * open amount is more than 0, so the total is the tile's. Read only.
 */
export function dashboardAdvancesOpen(
  input: AdvancesReportInput & { only_open: boolean },
): Promise<AdvancesReport> {
  return callCommand<AdvancesReport>('advances_report', { input });
}

/** A page of the dead amount (R38) with the totals of the filter, for the dead section of کل اگراہی (PRD FR-9.7). Read only. */
export function dashboardDeadAmount(input: DeadAmountInput): Promise<DeadAmountPage> {
  return callCommand<DeadAmountPage>('reports_dead_amount', { input });
}

// Print, PDF and Excel of the new pages (PRD FR-19.10, FR-19.3, plan 79). Each page has the same trio as the buyer dues:
// `..._print_data` writes every row of the filter in the print language (the preview and the hidden print window
// read it), `..._print` prints it or saves it as a PDF, and the Excel file is made by `tables_export_excel`.

/** Every seller of the filter, written in the print language (PRD FR-9.6, FR-19.13). */
export function dashboardSellerDuesPrintData(input: SellerDuesPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_seller_dues_print_data', { input });
}

/** Prints the seller dues, or saves them as a PDF: the same filter, order, language and paper as the preview. */
export function dashboardSellerDuesPrint(input: PrintSellerDuesInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_seller_dues_print', { input });
}

/** The four parts of کل اگراہی and the grand total, in the print language (PRD FR-9.7). */
export function dashboardTotalDuesPrintData(input: TotalDuesPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_total_dues_print_data', { input });
}

export function dashboardTotalDuesPrint(input: PrintTotalDuesInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_total_dues_print', { input });
}

/** Every person who borrowed and still owes, in the print language (PRD FR-9.1, FR-16.5). */
export function dashboardLentOpenPrintData(input: LentOpenPrintInput): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_lent_open_print_data', { input });
}

export function dashboardLentOpenPrint(input: PrintLentOpenInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_lent_open_print', { input });
}

/** Every seller with an open advance, in the print language (PRD FR-9.1, FR-15.8). */
export function dashboardAdvancesOpenPrintData(
  input: AdvancesReportPrintInput,
): Promise<TablePrintData> {
  return callCommand<TablePrintData>('advances_report_print_data', { input });
}

export function dashboardAdvancesOpenPrint(input: PrintAdvancesOpenInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('advances_report_print', { input });
}

/** One month of نقدی, every day of it, in the print language (PRD FR-9.8). */
export function dashboardCashSalesMonthPrintData(
  input: CashSalesMonthPrintInput,
): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_cash_sales_month_print_data', { input });
}

export function dashboardCashSalesMonthPrint(
  input: PrintCashSalesMonthInput,
): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_cash_sales_month_print', { input });
}

/** Every buyer of one day of نقدی, in the print language (PRD FR-9.8). */
export function dashboardCashSalesDayPrintData(
  input: CashSalesDayPrintInput,
): Promise<TablePrintData> {
  return callCommand<TablePrintData>('dashboard_cash_sales_day_print_data', { input });
}

export function dashboardCashSalesDayPrint(input: PrintCashSalesDayInput): Promise<PrintOutcome> {
  return callCommand<PrintOutcome>('dashboard_cash_sales_day_print', { input });
}
