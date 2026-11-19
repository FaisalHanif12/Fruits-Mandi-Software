// The hidden print window's route (plans 14, 15 and 17e): `#/print?...`. The address says which
// document it is for: `bikri=<id>` is a document of a saved بکری (the sale book's three),
// `voucher=<id>` is the receipt slip of a saved voucher, `collection=<YYYY-MM-DD>` is the
// collection list of a day, `report=<id>` is a report (or the slips of R5 with `slips=1`) and
// `analytics=<view>` is a view of the analytics, `print=<kind>` is one of the prints of plan 78 (app/print-kinds), and `bikrilist=1` is پچھلی بکری (plan 72 P8) (the party's statement is `print=statement-rows`). Each page calls its own print-data command and the
// print header, and nothing else (the capability of the window allows only those commands).

import { useRouterState } from '@tanstack/react-router';
import { AnalyticsPrintPage, isAnalyticsAddress } from '../features/analytics';
import { BikriPrintPage } from '../features/bikri/print/BikriPrintPage';
import { PastPrintPage } from '../features/bikri/print/PastPrintPage';
import { isPastListAddress } from '../features/bikri/print/past-print-params';
import { ReportPrintPage, isReportAddress } from '../features/reports';
import { CollectionPrintPage } from '../features/vouchers/print/CollectionPrintPage';
import { VoucherPrintPage } from '../features/vouchers/print/VoucherPrintPage';
import { KindPrintPage } from './print-kinds/KindPrintPage';
import { isKindAddress } from './print-kinds/params';
import { isCollectionAddress } from '../features/vouchers/print/collection-params';
import { isVoucherAddress } from '../features/vouchers/print/params';

export function PrintRoute() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  // The prints of plan 78 (`print=<kind>`): the buyer slips, the seller bills, the whole sale, ادھاری تفصیل and the rest.
  if (isKindAddress(searchStr)) return <KindPrintPage />;
  if (isReportAddress(searchStr)) return <ReportPrintPage />;
  if (isAnalyticsAddress(searchStr)) return <AnalyticsPrintPage />;
  if (isPastListAddress(searchStr)) return <PastPrintPage />;
  if (isCollectionAddress(searchStr)) return <CollectionPrintPage />;
  return isVoucherAddress(searchStr) ? <VoucherPrintPage /> : <BikriPrintPage />;
}
