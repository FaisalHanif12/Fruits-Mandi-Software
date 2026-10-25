// The hidden print window's route (plans 14, 15 and 17e): `#/print?...`. The address says which
// document it is for: `bikri=<id>` is a document of a saved بکری (the sale book's three),
// `voucher=<id>` is the receipt slip of a saved voucher, `collection=<YYYY-MM-DD>` is the
// collection list of a day, `report=<id>` is a report (or the slips of R5 with `slips=1`) and
// `analytics=<view>` is a view of the analytics. Each page calls its own print-data command and the
// print header, and nothing else (the capability of the window allows only those commands).

import { useRouterState } from '@tanstack/react-router';
import { AnalyticsPrintPage, isAnalyticsAddress } from '../features/analytics';
import { BikriPrintPage } from '../features/bikri/print/BikriPrintPage';
import { ReportPrintPage, isReportAddress } from '../features/reports';
import { CollectionPrintPage } from '../features/vouchers/print/CollectionPrintPage';
import { VoucherPrintPage } from '../features/vouchers/print/VoucherPrintPage';
import { isCollectionAddress } from '../features/vouchers/print/collection-params';
import { isVoucherAddress } from '../features/vouchers/print/params';

export function PrintRoute() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  if (isReportAddress(searchStr)) return <ReportPrintPage />;
  if (isAnalyticsAddress(searchStr)) return <AnalyticsPrintPage />;
  if (isCollectionAddress(searchStr)) return <CollectionPrintPage />;
  return isVoucherAddress(searchStr) ? <VoucherPrintPage /> : <BikriPrintPage />;
}
