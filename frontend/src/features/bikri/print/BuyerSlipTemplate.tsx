// The buyer slips (R5, PRD FR-12.12): one slip per buyer, each on a new sheet ("print all"), on A5
// or on the 80 mm roll. It lays out the print data and works out nothing (CLAUDE.md rule 1).

import type { Ref } from 'react';
import { PrintDocument, type PrintData } from '../../../shared/print';
import { slipColumns } from './columns';

interface BuyerSlipTemplateProps {
  data: PrintData;
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function BuyerSlipTemplate({ data, applyRoot, ref }: BuyerSlipTemplateProps) {
  return (
    <PrintDocument
      data={data}
      columnsOf={slipColumns}
      {...(applyRoot === undefined ? {} : { applyRoot })}
      {...(ref === undefined ? {} : { ref })}
    />
  );
}
