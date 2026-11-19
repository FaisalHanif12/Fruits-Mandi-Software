// The whole sale print on screen, in the print language (PRD FR-19.15 "Whole sale print", plan 78 A4): the
// header of every print, then the sellers table (the R6 columns: one row for each lot, and a totals row), then
// the buyers table (buyer, what he bought in short, bill, نقد, ادھار, and a totals row), one after the other and
// never a page for a party. The shop sees everything sold on one or two pages. The backend sends it ready
// (`SalePrintData`): every figure, date and name is already written in the print language; this lays the
// two tables out and adds up nothing (CLAUDE.md rule 1). The in-app preview and the hidden print window draw
// this same page.

import { useLocalization } from '@fluent/react';
import type { Ref } from 'react';
import type { SalePrintData } from '../../../bindings/SalePrintData';
import { PrintHeader, PrintRoot, PrintScreenFooter } from '../../../shared/print';
import { TableBlock, useFitToSheet, type PrintTable } from './ReportPrintDocument';

interface SalePrintDocumentProps {
  data: SalePrintData;
  /** True in the hidden print window: the root element then takes the print language, direction and footer. */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

/** The title of the sellers table: the glossary word for a seller, as the old overall print had it. */
const SELLERS_TITLE_ID = 'print-sale-sellers';

function SaleBody({ data }: { data: SalePrintData }) {
  const { l10n } = useLocalization();
  // Both tables are brought inside the sheet by the same zoom, so the sellers and the buyers are one size.
  const scope = useFitToSheet<HTMLDivElement>(data);
  const sellers: PrintTable = data.sellers;
  // The buyers table writes what he bought as "(item qty . rate)", like ادھاری تفصیل (R5).
  const buyers: PrintTable = { ...sellers, report: 'R5' };
  return (
    <div className="print-report" data-variant="sale-print">
      <section className="print-sheet">
        <PrintHeader header={data.frame.header} />
        <div ref={scope} className="print-day-tables">
          <section className="print-day-table">
            <h2 className="print-day-table-title">{l10n.getString(SELLERS_TITLE_ID)}</h2>
            <TableBlock data={sellers} block={sellers} dense />
          </section>
          <section className="print-day-table">
            <h2 className="print-day-table-title">{l10n.getString(data.buyers.title_id)}</h2>
            <TableBlock data={buyers} block={data.buyers} dense />
          </section>
        </div>
        <PrintScreenFooter footer={data.frame.footer} />
      </section>
    </div>
  );
}

export function SalePrintDocument({ data, applyRoot = false, ref }: SalePrintDocumentProps) {
  return (
    <PrintRoot
      frame={data.frame}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <SaleBody data={data} />
    </PrintRoot>
  );
}
