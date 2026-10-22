// One printed report on screen, in the print language (PRD FR-7.36, FR-19.3, FR-19.5, FR-19.13).
// It is the shared print layout: the firm header, the title and the filters (dates and party), the
// lines above the table, every row, the totals row under its columns, the amount in words where the
// backend sent it, and "Page x of y" as the page footer. Every figure and date is already written
// by the backend in the print language; this file lays them out and adds up nothing (CLAUDE.md
// rule 1). The in-app preview and the hidden print window (later) draw this same page.

import { useLocalization } from '@fluent/react';
import { useEffect, type Ref } from 'react';
import type { PrintReportColumn } from '../../../bindings/PrintReportColumn';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { ReportPrintData } from '../../../bindings/ReportPrintData';
import type { ReportPrintDetailPart } from '../../../bindings/ReportPrintDetailPart';
import {
  FOOTER_LABEL_IDS,
  PrintProvider,
  applyPrintRoot,
  footerContent,
} from '../../../shared/print';
import { Code } from '../../../shared/ui/Code';
import { Num } from '../../../shared/ui/Num';
import { ScriptText } from '../../../shared/ui/ScriptText';
import { monthText } from '../cells';
import '../reports.css';

/** The column kinds that hold figures: they sit on the physical right edge (NFR-USE-19). */
const FIGURE_KINDS: ReadonlySet<PrintReportColumn['kind']> = new Set([
  'number',
  'money',
  'balance',
]);

/** The message of a zero balance's side: a nil balance has no side word. */
const ZERO_SIDE_ID = 'report-side-zero';

function DetailLine({ part }: { part: ReportPrintDetailPart }) {
  const { l10n } = useLocalization();
  switch (part.kind) {
    case 'sale':
      return (
        <p>
          {l10n.getString('report-detail-sale', {
            number: part.bikri_number,
            item: part.item,
            qty: part.qty,
            packing: part.packing,
            rate: part.rate,
            value: part.value,
            commission: part.commission,
            laga: part.laga,
          })}
        </p>
      );
    case 'lot':
      return (
        <p>
          {l10n.getString('report-detail-lot', {
            number: part.bikri_number,
            item: part.item,
            qty: part.qty,
            packing: part.packing,
            gross: part.gross,
            deductions: part.deductions,
          })}
        </p>
      );
    case 'cash_lent':
      return <p>{l10n.getString('report-detail-cash-lent', { reason: part.reason })}</p>;
    case 'text':
      return (
        <p>
          <bdi dir="auto">{part.text}</bdi>
        </p>
      );
  }
}

function Cell({ cell }: { cell: ReportPrintCell }) {
  const { l10n } = useLocalization();
  switch (cell.type) {
    case 'empty':
      return null;
    case 'text':
      return <bdi dir="auto">{cell.text}</bdi>;
    case 'label':
      return <>{l10n.getString(cell.id)}</>;
    case 'number':
    case 'money':
    case 'date':
      return <Num>{cell.text}</Num>;
    case 'month':
      return <Num>{monthText(Number(cell.year), cell.month)}</Num>;
    case 'balance':
      return (
        <span className="print-balance">
          <Num>{cell.text}</Num>
          <span className="print-balance-side">
            {cell.side_id === ZERO_SIDE_ID ? null : l10n.getString(cell.side_id)}
          </span>
        </span>
      );
    case 'party': {
      const { party } = cell;
      const name =
        party.tag === null || party.tag === '' ? party.name : `${party.name} ${party.tag}`;
      return (
        <>
          <Code>{party.short_code ?? party.code}</Code> <ScriptText>{name}</ScriptText>
        </>
      );
    }
    case 'names':
      return <ScriptText>{cell.text}</ScriptText>;
    case 'doc': {
      const kind = l10n.getString(cell.type_id);
      return cell.number === null ? (
        <>{kind}</>
      ) : (
        <>
          {kind} <Num>{cell.number}</Num>
        </>
      );
    }
    case 'detail':
      return (
        <div className="print-lines">
          {cell.parts.map((part, index) => (
            <DetailLine key={index} part={part} />
          ))}
        </div>
      );
  }
}

/** Figures sit on the right edge; a date and the kind of an entry stay on one line. */
function cellClass(kind: PrintReportColumn['kind']): string {
  if (FIGURE_KINDS.has(kind)) return 'print-num';
  if (kind === 'date' || kind === 'month') return 'print-col-date print-nowrap';
  if (kind === 'doc') return 'print-nowrap';
  return '';
}

interface ReportPrintDocumentProps {
  data: ReportPrintData;
  /**
   * True in the hidden print window: the root element then takes the print language, direction and
   * footer, so the page margin boxes are right. False in the in-app preview, which must not change
   * the app's own language.
   */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

function ReportPrintBody({ data, applyRoot = false, ref }: ReportPrintDocumentProps) {
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

  const { firm } = data;
  const contact = [firm.address, firm.phone].filter((part) => part.trim() !== '');
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
      <section className="print-sheet">
        <header className="print-header">
          <div className="print-firm">
            <p className="print-firm-name">{firm.name}</p>
            {firm.mandi === '' ? null : <p className="print-firm-line">{firm.mandi}</p>}
            {contact.length === 0 ? null : (
              <p className="print-firm-line">
                {contact.map((part, index) => (
                  <span key={part}>
                    {index > 0 ? ' · ' : null}
                    <bdi dir="auto">{part}</bdi>
                  </span>
                ))}
              </p>
            )}
          </div>
          <h1 className="print-title">{l10n.getString(data.title_id)}</h1>
        </header>
        <dl className="print-facts">
          {data.filters.map((fact) => (
            <div key={fact.label_id} className="print-fact">
              <dt>{l10n.getString(fact.label_id)}</dt>
              <dd>{fact.value === '' ? null : <bdi dir="auto">{fact.value}</bdi>}</dd>
            </div>
          ))}
          <div className="print-fact">
            <dt>{l10n.getString('print-printed-at')}</dt>
            <dd>
              <Num>{data.printed_at}</Num>
            </dd>
          </div>
        </dl>
        {data.summary.length === 0 ? null : (
          <dl className="print-facts print-summary-lines">
            {data.summary.map((line) => (
              <div key={line.label_id} className="print-fact">
                <dt>{l10n.getString(line.label_id)}</dt>
                <dd>
                  <Cell cell={line.cell} />
                </dd>
              </div>
            ))}
          </dl>
        )}
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
            {data.rows.map((row, index) => (
              <tr key={index} data-row={row.kind}>
                {row.cells.map((cell, position) => (
                  <td key={position} className={cellClass(data.columns[position]?.kind ?? 'text')}>
                    <Cell cell={cell} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
          <tfoot>
            <tr>
              {data.totals.map((cell, position) => (
                <td key={position} className={cellClass(data.columns[position]?.kind ?? 'text')}>
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
        {data.words === null ? null : (
          <p className="print-words">
            <span className="print-words-label">{l10n.getString(data.words.label_id)}</span>
            <span className="print-words-text">{data.words.text}</span>
          </p>
        )}
        {/* On paper the footer is the page margin (print.css). This line is for the preview. */}
        <p className="print-screen-footer">
          {l10n.getString('report-print-row-count')} <Num>{data.row_count}</Num>
        </p>
      </section>
    </div>
  );
}

export function ReportPrintDocument(props: ReportPrintDocumentProps) {
  return (
    <PrintProvider language={props.data.language}>
      <ReportPrintBody {...props} />
    </PrintProvider>
  );
}
