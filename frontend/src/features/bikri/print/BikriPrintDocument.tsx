// Picks the template of a printed بکری document: the seller bill, the buyer slips or the پختہ
// slip. The print data says which one it is (`doc`).

import type { Ref } from 'react';
import type { PrintData } from '../../../shared/print';
import { BuyerSlipTemplate } from './BuyerSlipTemplate';
import { PukhtaSlipTemplate } from './PukhtaSlipTemplate';
import { SellerBillTemplate } from './SellerBillTemplate';

interface BikriPrintDocumentProps {
  data: PrintData;
  /** True in the hidden print window (see PrintDocument). */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function BikriPrintDocument({ data, applyRoot, ref }: BikriPrintDocumentProps) {
  const rest = {
    data,
    ...(applyRoot === undefined ? {} : { applyRoot }),
    ...(ref === undefined ? {} : { ref }),
  };
  switch (data.doc) {
    case 'seller_bill':
      return <SellerBillTemplate {...rest} />;
    case 'buyer_slips':
      return <BuyerSlipTemplate {...rest} />;
    case 'pukhta_slip':
      return <PukhtaSlipTemplate {...rest} />;
    case 'receipt_slip':
    case 'collection_list':
      // The receipt slip belongs to a voucher (plan 15) and the collection list to the collection
      // screen (plan 17e); each comes from its own command, never from `reports_bikri_print_data`,
      // so the sale book has no template for them.
      return null;
  }
}
