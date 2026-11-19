// The public side of the `dashboard` module (Tech Stack 5.2): what other modules may import. The
// attention bell in the top bar and the guide are in `app/`; they use these.
export { AttentionLines } from './components/AttentionLines';
export { commissionSearchOfSource } from './commission-search';
export { useOpenDestination, type Destination } from './destinations';
export { ATTENTION_KEY, DASHBOARD_KEY, useAttention, useDashboard } from './hooks';
// Plan 78: the hidden print window of the buyer dues and the commission page reads these (app/print-kinds).
export {
  dashboardAdvancesOpenPrintData,
  dashboardBuyerDuesPrintData,
  dashboardCashSalesDayPrintData,
  dashboardCashSalesMonthPrintData,
  dashboardCommissionPrintData,
  dashboardLentOpenPrintData,
  dashboardSellerDuesPrintData,
  dashboardTotalDuesPrintData,
} from './api';
