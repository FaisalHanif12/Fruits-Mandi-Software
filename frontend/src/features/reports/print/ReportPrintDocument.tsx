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
import type { PrintReportRow } from '../../../bindings/PrintReportRow';
import type { PrintReportTable } from '../../../bindings/PrintReportTable';
import type { ReportPrintSummaryLine } from '../../../bindings/ReportPrintSummaryLine';
import type { ReportRowKind } from '../../../bindings/ReportRowKind';
import { PrintHeader, PrintRoot, PrintScreenFooter, type PrintFrame } from '../../../shared/print';
import { Num } from '../../../shared/ui/Num';
import { ScriptText } from '../../../shared/ui/ScriptText';
import { emptyNameMessageId, monthMessageId } from '../cells';
import { daySheetSegments } from './daySheet';
import { Cell, alignOf, cellClass } from './PrintCell';
import { fitZoom } from './fit';
import { SlipGoods } from './SlipGoods';
import { SlipLines, isSlip } from './SlipLines';

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
function columnClass(
  data: Pick<PrintTable, 'report'>,
  columns: PrintTable['columns'],
  position: number,
): string {
  const base = cellClass(columns[position]?.kind ?? 'text');
  // The code column of the party day sheet (R35) and of its short tables in the day total sheet (R34)
  // holds a short code or an eight-digit code of a one-time person: it never wraps.
  return (data.report === 'R35' || data.report === 'R34') && columns[position]?.key === 'code'
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
  /** More tables under the main one, with every day of the filter (the day total sheet R34, plan 42). */
  tables?: PrintReportTable[];
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
 * What a sheet leaves for a table: its width less its own padding (the page margin on paper is not
 * padding, so there it is the whole width).
 */
function roomOf(sheet: HTMLElement): number {
  const style = getComputedStyle(sheet);
  const lengthOf = (px: string) => Number(px.replace('px', '')) || 0;
  return (
    sheet.clientWidth - (lengthOf(style.paddingInlineStart) + lengthOf(style.paddingInlineEnd))
  );
}

/**
 * Brings the tables inside `scope` that are wider than their sheet back inside it (`fit.ts`). It
 * measures after the fonts are loaded, because the Urdu and English words have different widths, and
 * again when the data, the language or the paper change. All the tables of one scope get the SAME zoom,
 * the smallest one any of them needs, so the buyers' list and the sellers' list of a day are in one size.
 */
function useFitToSheet<E extends HTMLElement>(key: unknown) {
  const ref = useRef<E>(null);
  useLayoutEffect(() => {
    const scope = ref.current;
    const sheet = scope?.closest<HTMLElement>('.print-sheet') ?? null;
    if (scope === null || sheet === null) return undefined;
    let current = true;
    const fit = () => {
      if (!current) return;
      const tables =
        scope instanceof HTMLTableElement ? [scope] : [...scope.querySelectorAll('table')];
      for (const table of tables) table.style.zoom = '';
      const room = roomOf(sheet);
      const zooms = tables
        .map((table) => fitZoom(table.offsetWidth, room))
        .filter((zoom): zoom is number => zoom !== null);
      if (zooms.length === 0) return;
      const zoom = Math.min(...zooms);
      for (const table of tables) table.style.zoom = String(zoom);
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

/** What one printed table is drawn from: the main table, or one of the short tables of a day (R34). */
interface TableBlockData {
  columns: PrintTable['columns'];
  headings?: PrintTable['headings'];
  rows: PrintTable['rows'];
  totals_label_id: string;
  totals: ReportPrintCell[];
}

interface TableBlockProps {
  data: PrintTable;
  block: TableBlockData;
  /** The head row of the columns. A part of a day that goes on after its short tables has none. */
  showHead?: boolean;
  /** The smaller print size and tighter cells, for a short table that must share a narrow sheet. */
  dense?: boolean;
  ref?: Ref<HTMLTableElement>;
}

/**
 * One table of the print: its head, its rows and, when it has figures to total, its totals row. The
 * main table, a part of it (the day total sheet is cut after its sellers' lines) and each short table
 * of a day are drawn by this one function, so a figure, a name and a balance look the same in all.
 */
function TableBlock({ data, block, showHead = true, dense = false, ref }: TableBlockProps) {
  const { l10n } = useLocalization();
  // The buyers' list (R5) writes each sale line as "(item qty . rate)", like the client's own print.
  const buyersList = data.report === 'R5';
  // The party day sheet (R35) is the client's own slip: its lines first, what the day comes to underneath.
  const sheet = data.report === 'R35';
  // The word "Total" stands in the first column of words that has no total of its own, never in a
  // column of figures such as a date (plan 39 R6). In a list of parties it stands under the party.
  const partyColumn = block.columns.findIndex((column) => column.kind === 'party');
  const labelColumn =
    partyColumn !== -1
      ? partyColumn
      : block.columns.findIndex(
          (column, position) =>
            alignOf(column.kind) !== 'num' && (block.totals[position]?.type ?? 'empty') === 'empty',
        );
  const totalsLabelAt = labelColumn === -1 ? 0 : labelColumn;
  const classOf = (position: number) => columnClass(data, block.columns, position);
  // A slip on the 80 mm roll has no room for five columns: each line is two rows of text (plan 42).
  if (data.paper === 'mm80' && isSlip(data)) {
    return <SlipGoods columns={block.columns} rows={block.rows} />;
  }
  return (
    <table
      ref={ref}
      className={
        dense || block.columns.length > DENSE_COLUMNS
          ? 'print-table print-table-dense'
          : 'print-table'
      }
    >
      {showHead ? (
        <thead>
          <tr>
            {block.columns.map((column, position) => (
              <th
                key={column.key}
                scope="col"
                data-align={alignOf(column.kind)}
                className={cellClass(column.kind)}
              >
                <ColumnHeading
                  heading={block.headings?.[position] ?? null}
                  titleId={column.title_id}
                />
              </th>
            ))}
          </tr>
        </thead>
      ) : null}
      <tbody>
        {block.rows.map((row, index) => (
          <tr key={index} data-row={row.kind}>
            {row.cells.map((cell, position) => {
              const span = groupSpanAt(row, position);
              // The empty cells a heading's words spread over are not drawn (see `groupSpanAt`).
              if (span === 0) return null;
              // A name left empty on purpose (no city, no collector, no اڈا) is said in words.
              const none =
                position === 0 && cell.type === 'empty' && data.report !== undefined
                  ? emptyNameMessageId(data.report, row.kind)
                  : null;
              return (
                <td
                  key={position}
                  {...(span > 1 ? { colSpan: span } : {})}
                  data-align={alignOf(block.columns[position]?.kind ?? 'text')}
                  className={classOf(position)}
                >
                  {none === null ? (
                    <Cell
                      cell={cell}
                      shortSale={buyersList}
                      nameOnly={
                        (sheet || data.report === 'R34') &&
                        block.columns[position]?.kind === 'party'
                      }
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
      {/* A table with nothing to total (the day summary R34, a slip) has no totals row. */}
      {block.totals.every((cell) => cell.type === 'empty') ? null : (
        <tfoot>
          <tr>
            {block.totals.map((cell, position) => (
              <td
                key={position}
                data-align={alignOf(block.columns[position]?.kind ?? 'text')}
                className={classOf(position)}
              >
                {position === totalsLabelAt && cell.type === 'empty' ? (
                  l10n.getString(block.totals_label_id)
                ) : (
                  <Cell cell={cell} />
                )}
              </td>
            ))}
          </tr>
        </tfoot>
      )}
    </table>
  );
}

/**
 * The lines of a day of the day total sheet on A4 (plan 42, Umar 31-10-2026): the label and value lines
 * stand in two columns side by side, the first half of the lines in the first column and the rest in
 * the second, so the sheet is half as long. Nothing is left out and no line changes its words or its
 * place in the order of reading, column after column. The short tables of the day stay full width, and
 * A5 keeps one column.
 */
function TwoColumnRows({
  data,
  block,
  showHead,
}: {
  data: PrintTable;
  block: TableBlockData;
  showHead: boolean;
}) {
  const half = Math.ceil(block.rows.length / 2);
  const columns = [block.rows.slice(0, half), block.rows.slice(half)].filter(
    (rows) => rows.length > 0,
  );
  return (
    <div className="print-two-columns">
      {columns.map((rows, index) => (
        <TableBlock key={index} data={data} block={{ ...block, rows }} showHead={showHead} dense />
      ))}
    </div>
  );
}

/**
 * The two short tables of one day of the day total sheet (R34): the buyers' and the sellers'. Each has
 * a title; when the print holds more than one day the title says which day it is.
 */
function DayTables({
  data,
  tables,
  withDay,
}: {
  data: PrintTable;
  tables: readonly PrintReportTable[];
  withDay: boolean;
}) {
  const { l10n } = useLocalization();
  // Both tables of the day are brought inside the sheet by the same zoom.
  const ref = useFitToSheet<HTMLDivElement>(tables);
  return (
    <div ref={ref} className="print-day-tables">
      {tables.map((table) => (
        <section key={`${table.day_iso}-${table.key}`} className="print-day-table">
          <h2 className="print-day-table-title">
            {l10n.getString(table.title_id)}
            {withDay ? (
              <>
                {' '}
                <Num>{table.day}</Num>
              </>
            ) : null}
          </h2>
          <TableBlock data={data} block={table} dense />
        </section>
      ))}
    </div>
  );
}

function ReportPrintBody({ data, frame }: Pick<ReportPrintDocumentProps, 'data' | 'frame'>) {
  const { l10n } = useLocalization();
  const tableRef = useFitToSheet<HTMLTableElement>(data);
  // The party day sheet (R35) is the client's own slip: its lines first, what the day comes to
  // underneath (PRD FR-19.12). The other reports have their figures above the table.
  const sheet = data.report === 'R35';
  // One party's slip (a buyer's or a seller's, plan 42): its labelled lines are a card under the goods.
  const slip = isSlip(data);
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
  // The day total sheet (R34) is cut after the sellers' lines of each day, and the short buyers' and
  // sellers' tables of that day stand there (PRD R34, v1.12). Every other print is one piece.
  const segments = daySheetSegments(data.rows, data.tables ?? []);
  const days = new Set((data.tables ?? []).map((table) => table.day_iso));
  // The first piece of rows is the one that is brought inside the sheet and has the head.
  const firstRows = segments.findIndex((segment) => !('tables' in segment));
  // The day total sheet on A4 has its lines in two columns (the short tables decide it is R34's).
  const twoColumns = data.paper === 'a4' && data.report === 'R34' && (data.tables ?? []).length > 0;
  return (
    <div className="print-report">
      <section className="print-sheet">
        <PrintHeader header={frame.header} />
        {sheet || slip ? null : summaryLines}
        {segments.map((segment, index) => {
          if ('tables' in segment) {
            return (
              <DayTables key={index} data={data} tables={segment.tables} withDay={days.size > 1} />
            );
          }
          // The first piece has the head; the piece that goes on after the short tables of a day does
          // not repeat it, and the next day starts with its own.
          const showHead = index === firstRows || segment.startsDay;
          const first = index === firstRows;
          const block: TableBlockData = {
            columns: data.columns,
            ...(data.headings === undefined ? {} : { headings: data.headings }),
            rows: segment.rows,
            totals_label_id: data.totals_label_id,
            totals: data.totals,
          };
          if (twoColumns) {
            return <TwoColumnRows key={index} data={data} block={block} showHead={showHead} />;
          }
          return (
            <TableBlock
              key={index}
              data={data}
              block={block}
              showHead={showHead}
              {...(first ? { ref: tableRef } : {})}
            />
          );
        })}
        {slip ? <SlipLines lines={data.summary ?? []} /> : sheet ? summaryLines : null}
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

/**
 * How many columns the cell at `position` spreads over, or 0 when an earlier cell spreads over it. Only a
 * heading row (`group`) with a sentence in it does this: the sentence takes the empty cells after it, up to
 * the next cell with something in it, so "آج کی بکری، 10-10-2026: 1" is on one line and is not broken in a
 * narrow column (plan 72 P8, the Urdu check of the print). Every other row is one cell for each column.
 */
function groupSpanAt(row: PrintReportRow, position: number): number {
  if (row.kind !== 'group') return 1;
  const cells = row.cells;
  // A cell that an earlier sentence spreads over.
  for (let before = position - 1; before >= 0; before -= 1) {
    const earlier = cells[before];
    if (earlier === undefined) break;
    if (earlier.type === 'empty') continue;
    if (earlier.type !== 'message') break;
    // Everything between that sentence and this cell is empty (the loop went over it), so it spreads here.
    if (cells[position]?.type === 'empty') return 0;
    break;
  }
  const own = cells[position];
  if (own?.type !== 'message') return 1;
  let span = 1;
  while (cells[position + span]?.type === 'empty') span += 1;
  return span;
}
