// The receipt slip (PRD FR-15.2): one slip for a saved voucher, on A5 or the 80 mm roll. It prints
// receipts, payments and advances. One page: who it is with, one total, the amount in words and the
// facts (number, day, how it was paid, the cheque, the reason, the collector, the narration). It
// has no goods table. It lays out the print data and works out nothing (CLAUDE.md rule 1).

import type { Ref } from 'react';
import { PrintDocument, type PrintData } from '../../../shared/print';

interface ReceiptSlipTemplateProps {
  data: PrintData;
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function ReceiptSlipTemplate({ data, applyRoot, ref }: ReceiptSlipTemplateProps) {
  return (
    <PrintDocument
      data={data}
      columnsOf={() => []}
      {...(applyRoot === undefined ? {} : { applyRoot })}
      {...(ref === undefined ? {} : { ref })}
    />
  );
}
