// The buyer accounts of a saved بکری, for the print box ("which buyer's slip"). One entry for each
// account, in the order the buyers first appear. The چکتا buyers are one entry, the account چکتا
// کھاتہ: choosing it prints a slip for each of them, by name (PRD FR-12.12, plan 64). A line that was
// replaced by an edit does not count.

import type { BikriDetail } from '../../../bindings/BikriDetail';
import type { RecordRef } from '../../../bindings/RecordRef';

export function buyersOf(detail: BikriDetail): RecordRef[] {
  const buyers: RecordRef[] = [];
  for (const lot of detail.lots) {
    for (const sale of lot.sales) {
      if (!sale.is_active) continue;
      if (!buyers.some((buyer) => buyer.id === sale.buyer.id)) buyers.push(sale.buyer);
    }
  }
  return buyers;
}
