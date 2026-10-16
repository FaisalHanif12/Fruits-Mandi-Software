// The buyer accounts of a saved بکری, for the print box ("which buyer's slip"). One entry for each
// account, in the order the buyers first appear. A one-time buyer is not here: he has the پختہ
// slip, not a buyer slip. A line that was replaced by an edit does not count.

import type { BikriDetail } from '../../../bindings/BikriDetail';
import type { RecordRef } from '../../../bindings/RecordRef';

export function buyersOf(detail: BikriDetail): RecordRef[] {
  const buyers: RecordRef[] = [];
  for (const lot of detail.lots) {
    for (const sale of lot.sales) {
      if (!sale.is_active || sale.pukhta !== null) continue;
      if (!buyers.some((buyer) => buyer.id === sale.buyer.id)) buyers.push(sale.buyer);
    }
  }
  return buyers;
}
