// One printed report on screen, in the print language (PRD FR-7.36, FR-19.3, FR-19.5, FR-19.13).
// It is the shared print layout: the ONE header of the app (`PrintHeader`: the firm, its contacts and
// trade mark, the title and the filters, built by the backend's `reports_print_header`), the lines
// above the table, every row, the totals row under its columns, the amount in words where the backend
// sent it, and "Page x of y" as the page footer (the `PrintRoot` frame). Every figure and date is
// already written by the backend in the print language; this file lays them out and adds up nothing
// (CLAUDE.md rule 1). The in-app preview and the hidden print window draw this same page.

import { useLocalization } from '@fluent/react';
import { useLayoutEffect, useRef, type Ref } from 'react';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { PrintFact } from '../../../bindings/PrintFact';
import type { ReportColumnKind } from '../../../bindings/ReportColumnKind';
import type { ReportId } from '../../../bindings/ReportId';
import type { ReportPrintSummaryLine } from '../../../bindings/ReportPrintSummaryLine';
import type { ReportRowKind } from '../../../bindings/ReportRowKind';
import { PrintHeader, PrintRoot, PrintScreenFooter, type PrintFrame } from '../../../shared/print';
import { Num } from '../../../shared/ui/Num';
import { ScriptText } from '../../../shared/ui/ScriptText';
import { emptyNameMessageId, monthMessageId } from '../cells';
import { Cell, cellClass } from './PrintCell';
import { fitZoom } from './fit';

/** A table of more columns than this is set in the smaller print size (reports.css). */
const DENSE_COLUMNS = 8;

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
  const base = cellClass(data.columns[position]?.kind ?? 'text');
  // The code column of the party day sheet holds an eight-digit code of a one-time person: it never wraps.
  return data.report === 'R35' && data.columns[position]?.key === 'code'
    ? `${base} print-nowrap`
    : base;
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
  paper: Paper;
  /** The title and the filters are what the backend turns into the shared header (`PrintFrameInput`). */
  title_id: string;
  filters: PrintFact[];
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

/**
 * Brings a table that is wider than its sheet back inside it (`fit.ts`). It measures after the fonts
 * are loaded, because the Urdu and English words have different widths, and again when the data, the
 * language or the paper change.
 */
function useFitToSheet(key: unknown) {
  const ref = useRef<HTMLTableElement>(null);
  useLayoutEffect(() => {
    const table = ref.current;
    const sheet = table?.closest<HTMLElement>('.print-sheet') ?? null;
    if (table === null || sheet === null) return undefined;
    let current = true;
    const fit = () => {
      if (!current) return;
      table.style.zoom = '';
      // What the sheet leaves for the table: its width less its own padding (the page margin on
      // paper is not padding, so there it is the whole width).
      const style = getComputedStyle(sheet);
      const lengthOf = (px: string) => Number(px.replace('px', '')) || 0;
      const padding = lengthOf(style.paddingInlineStart) + lengthOf(style.paddingInlineEnd);
      const zoom = fitZoom(table.offsetWidth, sheet.clientWidth - padding);
      if (zoom !== null) table.style.zoom = String(zoom);
    };
    fit();
    // A window without the font API (a test page) has nothing to wait for.
    if ('fonts' in document) void document.fonts.ready.then(fit);
    return () => {
      current = false;
    };
  }, [key]);
  return ref;
}

function ReportPrintBody({ data, frame }: Pick<ReportPrintDocumentProps, 'data' | 'frame'>) {
  const { l10n } = useLocalization();
  const tableRef = useFitToSheet(data);
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
  return (
    <div className="print-report">
      <section className="print-sheet">
        <PrintHeader header={frame.header} />
        {sheet ? null : summaryLines}
        <table
          ref={tableRef}
          className={
            data.columns.length > DENSE_COLUMNS ? 'print-table print-table-dense' : 'print-table'
          }
        >
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
                        <Cell
                          cell={cell}
                          shortSale={buyersList}
                          nameOnly={sheet && data.columns[position]?.key === 'party'}
                        />
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
        {/* On paper the footer is the page margin (print.css). These lines are for the preview. */}
        <PrintScreenFooter footer={frame.footer} />
        <p className="print-screen-footer">
          {l10n.getString('report-print-row-count')} <Num>{data.row_count}</Num>
        </p>
      </section>
    </div>
  );
}

export function ReportPrintDocument({
  data,
  frame,
  applyRoot = false,
  ref,
}: ReportPrintDocumentProps) {
  return (
    <PrintRoot
      frame={frame}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <ReportPrintBody data={data} frame={frame} />
    </PrintRoot>
  );
}
