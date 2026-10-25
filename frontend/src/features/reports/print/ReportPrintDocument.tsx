// One printed report on screen, in the print language (PRD FR-7.36, FR-19.3, FR-19.5, FR-19.13).
// It is the shared print layout: the firm header, the title and the filters (dates and party), the
// lines above the table, every row, the totals row under its columns, the amount in words where the
// backend sent it, and "Page x of y" as the page footer. Every figure and date is already written
// by the backend in the print language; this file lays them out and adds up nothing (CLAUDE.md
// rule 1). The in-app preview and the hidden print window (later) draw this same page.

import { useLocalization } from '@fluent/react';
import { useEffect, type Ref } from 'react';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { PrintFact } from '../../../bindings/PrintFact';
import type { PrintFirm } from '../../../bindings/PrintFirm';
import type { ReportColumnKind } from '../../../bindings/ReportColumnKind';
import type { ReportId } from '../../../bindings/ReportId';
import type { ReportPrintSummaryLine } from '../../../bindings/ReportPrintSummaryLine';
import type { ReportRowKind } from '../../../bindings/ReportRowKind';
import type { TextDir } from '../../../bindings/TextDir';
import {
  FOOTER_LABEL_IDS,
  PrintProvider,
  applyPrintRoot,
  footerContent,
} from '../../../shared/print';
import { Num } from '../../../shared/ui/Num';
import { ScriptText } from '../../../shared/ui/ScriptText';
import { emptyNameMessageId, monthMessageId } from '../cells';
import { Cell, cellClass } from './PrintCell';
import { ReportSheetHeader } from './ReportSheetHeader';

/** The heading of a column: its message, or the day, month or item it stands for. */
function ColumnHeading({ heading, titleId }: { heading: PrintHeading | null; titleId: string }) {
  const { l10n } = useLocalization();
  if (heading === null) return <>{l10n.getString(titleId)}</>;
  switch (heading.kind) {
    case 'text':
      return <ScriptText>{heading.text}</ScriptText>;
    case 'month':
      return (
        <>
          {l10n.getString(monthMessageId(heading.month))} <Num>{heading.year}</Num>
        </>
      );
    case 'other_items':
      return <>{l10n.getString('analytics-col-other-items')}</>;
    case 'partner':
      // A trip has the same two columns for each partner: whose they are, then what they hold.
      return (
        <>
          {l10n.getString('analytics-col-partner-heading', {
            name: heading.text,
            title: l10n.getString(titleId),
          })}
        </>
      );
  }
}

/** The class of the cell at a column: a column the data does not name is plain text. */
function columnClass(data: PrintTable, position: number): string {
  return cellClass(data.columns[position]?.kind ?? 'text');
}

/** The heading of a column that is not one message: a day, a month or an item (analytics, plan 21). */
export type PrintHeading =
  | { kind: 'text'; text: string }
  | { kind: 'month'; year: string; month: number }
  | { kind: 'other_items' }
  | { kind: 'partner'; text: string };

/**
 * What the printed table is drawn from: the print data of a report, or of an analytics view (which
 * has no lines above the table and no amount in words, and headed columns). Every figure and date is
 * already written by the backend in the print language; the page adds up nothing.
 */
export interface PrintTable {
  language: Language;
  dir: TextDir;
  paper: Paper;
  urdu_digits: boolean;
  title_id: string;
  firm: PrintFirm;
  filters: PrintFact[];
  printed_at: string;
  columns: { key: string; title_id: string; kind: ReportColumnKind }[];
  /** The heading of each column when it is not its message (same order as `columns`). */
  headings?: readonly (PrintHeading | null)[];
  rows: { cells: ReportPrintCell[]; kind: ReportRowKind }[];
  row_count: string;
  totals_label_id: string;
  totals: ReportPrintCell[];
  summary?: ReportPrintSummaryLine[];
  words?: { label_id: string; text: string } | null;
  /** Which report it is, for the words of an empty name in a few reports. */
  report?: ReportId;
}

interface ReportPrintDocumentProps {
  data: PrintTable;
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

  // The party day sheet (R35) is the client's own slip: its lines first, what the day comes to
  // underneath (PRD FR-19.12). The other reports have their figures above the table.
  const sheet = data.report === 'R35';
  // The buyers' list (R5) writes each sale line as "(item qty . rate)", like the client's own print.
  const buyersList = data.report === 'R5';
  const summaryLines =
    (data.summary ?? []).length === 0 ? null : (
      <dl className="print-facts print-summary-lines">
        {(data.summary ?? []).map((line) => (
          <div key={line.label_id} className="print-fact">
            <dt>{l10n.getString(line.label_id)}</dt>
            <dd>
              <Cell cell={line.cell} />
            </dd>
          </div>
        ))}
      </dl>
    );
  const { firm } = data;
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
        <ReportSheetHeader
          firm={firm}
          titleId={data.title_id}
          filters={data.filters}
          printedAt={data.printed_at}
          dayFilter={sheet}
        />
        {sheet ? null : summaryLines}
        <table className="print-table">
          <thead>
            <tr>
              {data.columns.map((column, position) => (
                <th key={column.key} scope="col" className={cellClass(column.kind)}>
                  <ColumnHeading
                    heading={data.headings?.[position] ?? null}
                    titleId={column.title_id}
                  />
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {data.rows.map((row, index) => (
              <tr key={index} data-row={row.kind}>
                {row.cells.map((cell, position) => {
                  // A name left empty on purpose (no city, no collector, no اڈا) is said in words.
                  const none =
                    position === 0 && cell.type === 'empty' && data.report !== undefined
                      ? emptyNameMessageId(data.report, row.kind)
                      : null;
                  return (
                    <td key={position} className={columnClass(data, position)}>
                      {none === null ? (
                        <Cell cell={cell} shortSale={buyersList} />
                      ) : (
                        l10n.getString(none)
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
          {/* A report with nothing to total (the day summary) has no totals row. */}
          {data.totals.every((cell) => cell.type === 'empty') ? null : (
            <tfoot>
              <tr>
                {data.totals.map((cell, position) => (
                  <td key={position} className={columnClass(data, position)}>
                    {position === 0 && cell.type === 'empty' ? (
                      l10n.getString(data.totals_label_id)
                    ) : (
                      <Cell cell={cell} />
                    )}
                  </td>
                ))}
              </tr>
            </tfoot>
          )}
        </table>
        {sheet ? summaryLines : null}
        {data.words === null || data.words === undefined ? null : (
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
