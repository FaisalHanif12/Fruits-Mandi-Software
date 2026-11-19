// The printed پچھلی بکری on screen, in the print language (PRD 12.7, FR-7.36, FR-19.13): the shared
// print layout of the reports (the one header of the app, the table with a heading row for today and for
// each day, the totals row under the columns, "Page x of y"). The in-app preview and the hidden print
// window draw this same page. Every figure and date is already text in the print language; nothing
// is worked out here (CLAUDE.md rule 1).

import { useMemo, type Ref } from 'react';
import type { PrintFrame } from '../../../bindings/PrintFrame';
import { ReportPrintDocument } from '../../reports';
import type { BikriListPrintData } from '../../../bindings/BikriListPrintData';
import { pastPrintTable } from './past-print-table';

interface PastPrintDocumentProps {
  data: BikriListPrintData;
  /** The header and footer the backend built for this print. */
  frame: PrintFrame;
  /** True in the hidden print window, where the root of the page takes the print language and direction. */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function PastPrintDocument({ data, frame, applyRoot = false, ref }: PastPrintDocumentProps) {
  // One table for one print data: the document measures itself when its data changes only.
  const table = useMemo(() => pastPrintTable(data), [data]);
  return (
    <ReportPrintDocument
      data={table}
      frame={frame}
      {...(applyRoot ? { applyRoot: true } : {})}
      {...(ref === undefined ? {} : { ref })}
    />
  );
}
