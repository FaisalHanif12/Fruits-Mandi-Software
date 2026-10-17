// The hidden print window's route (plans 14 and 15): `#/print?...`. The address says which
// document it is for: `bikri=<id>` is a document of a saved بکری (the sale book's three), and
// `voucher=<id>` is the receipt slip of a saved voucher. Each page calls its own print-data command
// and nothing else (the capability of the window allows only those commands).

import { useRouterState } from '@tanstack/react-router';
import { BikriPrintPage } from '../features/bikri/print/BikriPrintPage';
import { VoucherPrintPage } from '../features/vouchers/print/VoucherPrintPage';
import { isVoucherAddress } from '../features/vouchers/print/params';

export function PrintRoute() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  return isVoucherAddress(searchStr) ? <VoucherPrintPage /> : <BikriPrintPage />;
}
