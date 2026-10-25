// All the credit buyers' slips of R5 on screen, one sheet for each buyer, in the print language (PRD
// 19.2 R5 "Print All", FR-7.36, FR-19.13). Each sheet is the firm header, the title and filters, who the
// slip is for, his sale lines written "(item qty . rate)" like the client's own print, his totals row
// and his total نگ; each starts on a new page when printed (print.css). Every figure and date is
// already written by the backend in the print language; this file lays them out and adds up nothing
// (CLAUDE.md rule 1). The totals of all the slips are in the preview only, not on a buyer's slip.

import { useLocalization } from '@fluent/react';
import { useEffect, type Ref } from 'react';
import type { BuyerBillSlip } from '../../../bindings/BuyerBillSlip';
import type { BuyerBillsPrintData } from '../../../bindings/BuyerBillsPrintData';
import type { PrintReportColumn } from '../../../bindings/PrintReportColumn';
import {
  FOOTER_LABEL_IDS,
  PrintProvider,
  applyPrintRoot,
  footerContent,
} from '../../../shared/print';
import { Code } from '../../../shared/ui/Code';
import { Num } from '../../../shared/ui/Num';
import { ScriptText } from '../../../shared/ui/ScriptText';
import { Cell, cellClass } from './PrintCell';
import { ReportSheetHeader } from './ReportSheetHeader';

/** The class of the cell at a column: a column the data does not name is plain text. */
function columnClass(columns: readonly PrintReportColumn[], position: number): string {
  return cellClass(columns[position]?.kind ?? 'text');
}

interface SlipProps {
  data: BuyerBillsPrintData;
  slip: BuyerBillSlip;
}

function Slip({ data, slip }: SlipProps) {
  const { l10n } = useLocalization();
  const { party } = slip;
  const name = party.tag === null || party.tag === '' ? party.name : `${party.name} ${party.tag}`;
  return (
    <section className="print-sheet" data-slip={slip.serial}>
      <ReportSheetHeader
        firm={data.firm}
        titleId={data.title_id}
        filters={data.filters}
        printedAt={data.printed_at}
      />
      <dl className="print-facts">
        <div className="print-fact">
          <dt>{l10n.getString('report-col-buyer')}</dt>
          <dd>
            <Code>{party.short_code ?? party.code}</Code> <ScriptText>{name}</ScriptText>
          </dd>
        </div>
      </dl>
      <table className="print-table">
        <thead>
          <tr>
            {data.columns.map((column) => (
              <th key={column.key} scope="col" className={cellClass(column.kind)}>
                {l10n.getString(column.title_id)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {slip.rows.map((row, index) => (
            <tr key={index} data-row={row.kind}>
              {row.cells.map((cell, position) => (
                <td key={position} className={columnClass(data.columns, position)}>
                  <Cell cell={cell} shortSale />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr>
            {slip.totals.map((cell, position) => (
              <td key={position} className={columnClass(data.columns, position)}>
                {position === 0 && cell.type === 'empty' ? (
                  l10n.getString(data.totals_label_id)
                ) : (
                  <Cell cell={cell} />
                )}
              </td>
            ))}
          </tr>
        </tfoot>
      </table>
      <dl className="print-facts print-summary-lines">
        <div className="print-fact">
          <dt>{l10n.getString(data.qty_label_id)}</dt>
          <dd>
            <Num>{slip.qty}</Num>
          </dd>
        </div>
      </dl>
      {/* On paper the footer is the page margin (print.css). This line is for the preview. */}
      <p className="print-screen-footer">
        {l10n.getString('report-print-row-count')} <Num>{slip.row_count}</Num>
      </p>
    </section>
  );
}

interface BuyerBillsPrintDocumentProps {
  data: BuyerBillsPrintData;
  /**
   * True in the hidden print window: the root element then takes the print language, direction and
   * footer, so the page margin boxes are right. False in the in-app preview, which must not change
   * the app's own language.
   */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

function BuyerBillsBody({ data, applyRoot = false, ref }: BuyerBillsPrintDocumentProps) {
  const { l10n } = useLocalization();
  const words = {
    printed: l10n.getString(FOOTER_LABEL_IDS.printed),
    page: l10n.getString(FOOTER_LABEL_IDS.page),
    of: l10n.getString(FOOTER_LABEL_IDS.of),
  };
  const footer = footerContent(words, data.printed_at, data.urdu_digits, data.paper !== 'mm80');
  useEffect(() => {
    if (!applyRoot) return undefined;
    return applyPrintRoot({ language: data.language, dir: data.dir, footer });
  }, [applyRoot, data.language, data.dir, footer]);
  return (
    <div
      ref={ref}
      className="print-doc print-report"
      lang={data.language}
      dir={data.dir}
      data-paper={data.paper}
      data-digits={data.urdu_digits ? 'urdu' : 'latin'}
      data-window={applyRoot ? 'true' : undefined}
    >
      {data.slips.map((slip) => (
        <Slip key={slip.serial} data={data} slip={slip} />
      ))}
      {/* What all the slips come to is for the one who prints them: it is not on a buyer's slip. */}
      <dl className="print-facts print-screen-footer" data-slips-total="">
        <div className="print-fact">
          <dt>{l10n.getString('report-buyer-bills-slips')}</dt>
          <dd>
            <Num>{data.slip_count}</Num>
          </dd>
        </div>
        <div className="print-fact">
          <dt>{l10n.getString(data.qty_label_id)}</dt>
          <dd>
            <Num>{data.qty}</Num>
          </dd>
        </div>
        {data.totals.map((cell, position) =>
          cell.type === 'empty' ? null : (
            <div key={position} className="print-fact">
              <dt>
                {l10n.getString(data.columns[position]?.title_id ?? data.totals_label_id)}
              </dt>
              <dd>
                <Cell cell={cell} />
              </dd>
            </div>
          ),
        )}
      </dl>
    </div>
  );
}

export function BuyerBillsPrintDocument(props: BuyerBillsPrintDocumentProps) {
  return (
    <PrintProvider language={props.data.language}>
      <BuyerBillsBody {...props} />
    </PrintProvider>
  );
}
