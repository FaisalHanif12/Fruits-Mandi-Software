// The hidden print window's route (plans 14 and 15): `#/print?...`. The address says which
// document it is for: `bikri=<id>` is a document of a saved بکری (the sale book's three),
// `voucher=<id>` is the receipt slip of a saved voucher, and `collection=<YYYY-MM-DD>` is the
// collection list of a day (plan 17e). Each page calls its own print-data command and nothing else
// (the capability of the window allows only those commands).

import { useRouterState } from '@tanstack/react-router';
import { BikriPrintPage } from '../features/bikri/print/BikriPrintPage';
import { CollectionPrintPage } from '../features/vouchers/print/CollectionPrintPage';
import { VoucherPrintPage } from '../features/vouchers/print/VoucherPrintPage';
import { isCollectionAddress } from '../features/vouchers/print/collection-params';
import { isVoucherAddress } from '../features/vouchers/print/params';

export function PrintRoute() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  if (isCollectionAddress(searchStr)) return <CollectionPrintPage />;
  return isVoucherAddress(searchStr) ? <VoucherPrintPage /> : <BikriPrintPage />;
}
