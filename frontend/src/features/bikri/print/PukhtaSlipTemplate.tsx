// The پختہ کھاتہ slip (PRD FR-13.1 to FR-13.3): one slip for each one-time buyer, with his name
// and phone. The same layout as a buyer slip, because a one-time buyer is charged like any buyer.

import type { Ref } from 'react';
import { PrintDocument, type PrintData } from '../../../shared/print';
import { slipColumns } from './columns';

interface PukhtaSlipTemplateProps {
  data: PrintData;
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function PukhtaSlipTemplate({ data, applyRoot, ref }: PukhtaSlipTemplateProps) {
  return (
    <PrintDocument
      data={data}
      columnsOf={slipColumns}
      {...(applyRoot === undefined ? {} : { applyRoot })}
      {...(ref === undefined ? {} : { ref })}
    />
  );
}
