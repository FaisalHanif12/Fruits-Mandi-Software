// The seller bill (R1, PRD FR-12.12): one sheet per seller, on A4 by default. The first printed
// document of a بکری. It lays out the print data and works out nothing (CLAUDE.md rule 1).

import type { Ref } from 'react';
import { PrintDocument, type PrintData } from '../../../shared/print';
import { sellerBillColumns } from './columns';

interface SellerBillTemplateProps {
  data: PrintData;
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function SellerBillTemplate({ data, applyRoot, ref }: SellerBillTemplateProps) {
  return (
    <PrintDocument
      data={data}
      columnsOf={sellerBillColumns}
      {...(applyRoot === undefined ? {} : { applyRoot })}
      {...(ref === undefined ? {} : { ref })}
    />
  );
}
