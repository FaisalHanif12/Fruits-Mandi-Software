// All the credit buyers' slips of R5 on screen, one sheet for each buyer, in the print language (PRD
// 19.2 R5 "Print All", FR-7.36, FR-19.13). Each sheet is the firm header, the title and filters, who the
// slip is for, his sale lines written "(item qty . rate)" like the client's own print, his totals row
// and his total نگ; each starts on a new page when printed (print.css). Every figure and date is
// already written by the backend in the print language; this file lays them out and adds up nothing
// (CLAUDE.md rule 1). The totals of all the slips are in the preview only, not on a buyer's slip.

import { useLocalization } from '@fluent/react';
import type { Ref } from 'react';
import type { BuyerBillSlip } from '../../../bindings/BuyerBillSlip';
import type { BuyerBillsPrintData } from '../../../bindings/BuyerBillsPrintData';
import type { PrintReportColumn } from '../../../bindings/PrintReportColumn';
import {
  PrintHeader,
  PrintName,
  PrintRoot,
  PrintScreenFooter,
  type PrintFrame,
} from '../../../shared/print';
import { Code } from '../../../shared/ui/Code';
import { Num } from '../../../shared/ui/Num';
import { Cell, cellClass } from './PrintCell';

/** The class of the cell at a column: a column the data does not name is plain text. */
function columnClass(columns: readonly PrintReportColumn[], position: number): string {
  return cellClass(columns[position]?.kind ?? 'text');
}

interface SlipProps {
  data: BuyerBillsPrintData;
  frame: PrintFrame;
  slip: BuyerBillSlip;
}

function Slip({ data, frame, slip }: SlipProps) {
  const { l10n } = useLocalization();
  const { party } = slip;
  return (
    <section className="print-sheet" data-slip={slip.serial}>
      <PrintHeader header={frame.header} />
      <dl className="print-facts">
        <div className="print-fact">
          <dt>{l10n.getString('report-col-buyer')}</dt>
          <dd>
            <Code>{party.short_code ?? party.code}</Code>{' '}
            <PrintName name={party.name} tag={party.tag ?? undefined} />
            {party.phone === undefined ? null : (
              <>
                {' '}
                <Code>{party.phone}</Code>
              </>
            )}
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
      {/* On paper the footer is the page margin (print.css). These lines are for the preview. */}
      <PrintScreenFooter footer={frame.footer} />
      <p className="print-screen-footer">
        {l10n.getString('report-print-row-count')} <Num>{slip.row_count}</Num>
      </p>
    </section>
  );
}

interface BuyerBillsPrintDocumentProps {
  data: BuyerBillsPrintData;
  /** The header and footer the backend built for this print (`reports_print_header`). */
  frame: PrintFrame;
  /**
   * True in the hidden print window: the root element then takes the print language, direction and
   * footer, so the page margin boxes are right. False in the in-app preview, which must not change
   * the app's own language.
   */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

function BuyerBillsBody({ data, frame }: { data: BuyerBillsPrintData; frame: PrintFrame }) {
  const { l10n } = useLocalization();
  return (
    <div className="print-report">
      {data.slips.map((slip) => (
        <Slip key={slip.serial} data={data} frame={frame} slip={slip} />
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
              <dt>{l10n.getString(data.columns[position]?.title_id ?? data.totals_label_id)}</dt>
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

export function BuyerBillsPrintDocument({
  data,
  frame,
  applyRoot = false,
  ref,
}: BuyerBillsPrintDocumentProps) {
  return (
    <PrintRoot
      frame={frame}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <BuyerBillsBody data={data} frame={frame} />
    </PrintRoot>
  );
}
