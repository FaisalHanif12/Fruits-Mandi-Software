// The buyer slips and the seller bills on screen, in the print language (PRD FR-19.15, FR-12.12, FR-19.13,
// plan 76 point 3, plan 78 A2, A3). Both are made of party slips, the R35 slip of each party for one day, which
// the backend sends ready (`DaySlipsPrintData`): the title with the party's name, its facts (the party's code,
// the day), the goods (item, packing, quantity, rate, amount), the labelled lines under them and the
// previous balance, the opening balance when it was made that day, and the new balance.
//
// - The buyer slips (`variant="parchi"`) have the buyers only. Several stand on one A4 page, two columns
//   (about six slips), each in a box with dashed cut lines, never cut across a page; each is short, is titled
//   "<name> کی روزانہ پرچی" and ends with an empty line for the collector to write the وصولی on. On A5 they
//   stand one under another.
// - The seller bills (`variant="seller-bills"`) have the sellers only, one bill to a page with the header of
//   every print, titled "<name> کا بل": the items with their lot names, each deduction on its own line, the net,
//   the previous balance and the new total.
// - A balance on a slip is written for the person who reads it: آپ کے ذمے (you owe), دکان کے ذمے (the shop owes
//   you) (plan 78 U-7 yes), in SlipLines.
//
// The whole sale print is its own document (SalePrintDocument).
//
// Every figure and date is written by the backend in the print language; this file lays them out and adds
// up nothing (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import type { Ref } from 'react';
import type { DaySlipsPrintData } from '../../../bindings/DaySlipsPrintData';
import type { PrintReportParty } from '../../../bindings/PrintReportParty';
import type { ReportColumnKind } from '../../../bindings/ReportColumnKind';
import type { ReportPrintData } from '../../../bindings/ReportPrintData';
import { PrintHeader, PrintRoot, PrintScreenFooter, PrintText } from '../../../shared/print';
import { Code } from '../../../shared/ui/Code';
import { Cell, alignOf, cellClass } from './PrintCell';
import { SlipLines } from './SlipLines';

export type DaySlipsVariant = 'parchi' | 'seller-bills';

/** The kind of the column at a position: a column the data does not name is plain text. */
function kindAt(columns: ReportPrintData['columns'], position: number): ReportColumnKind {
  return columns[position]?.kind ?? 'text';
}

/** The class of the cell at a column. */
function columnClass(columns: ReportPrintData['columns'], position: number): string {
  return cellClass(kindAt(columns, position));
}

/** The party's name as the title writes it: with his tag after it. */
function nameOf(party: PrintReportParty): string {
  return party.tag === null ? party.name : `${party.name} ${party.tag}`;
}

/** The id of the fact that holds the party (his code and name); a titled slip keeps only his code there. */
const PARTY_FACT_ID = 'report-filter-party';

/** The title of a slip that names the party, as words, or null when the slip is not titled by a name. */
export function slipTitle(
  slip: Pick<ReportPrintData, 'named_title_id' | 'party'>,
  getString: (id: string, vars: Record<string, string>) => string,
): string | null {
  if (slip.named_title_id === null || slip.party === null) return null;
  return getString(slip.named_title_id, { name: nameOf(slip.party) });
}

/** The facts of a slip (the party and the day) as one line of label and value pairs. */
function SlipFacts({ slip }: { slip: ReportPrintData }) {
  const { l10n } = useLocalization();
  // The title already has his name, so the line says his code only (plan 78 A6).
  const titled = slip.named_title_id !== null && slip.party !== null;
  return (
    <dl className="print-parchi-facts">
      {slip.filters.map((fact) => (
        <div key={fact.label_id} className="print-fact">
          <dt>{l10n.getString(fact.label_id)}</dt>
          <dd>
            {titled && fact.label_id === PARTY_FACT_ID && slip.party !== null ? (
              <Code>{slip.party.short_code ?? slip.party.code}</Code>
            ) : /\p{Script=Arabic}/u.test(fact.value) ? (
              <PrintText text={fact.value} />
            ) : (
              <Code>{fact.value}</Code>
            )}
          </dd>
        </div>
      ))}
    </dl>
  );
}

/** One party's slip: the facts, the goods, the labelled lines, and for a parchi the line to write on. */
function PartySlip({
  slip,
  parchi,
  firm,
  showTitle = true,
}: {
  slip: ReportPrintData;
  parchi: boolean;
  /** The firm's name: it heads a parchi, which stands alone on a page with other slips. */
  firm: string;
  /** False when the page already has the title in its header (a seller bill). */
  showTitle?: boolean;
}) {
  const { l10n } = useLocalization();
  const title = slipTitle(slip, (id, vars) => l10n.getString(id, vars));
  return (
    <article className="print-parchi-slip" data-slip="">
      {parchi ? <p className="print-parchi-firm">{firm}</p> : null}
      {showTitle && title !== null ? (
        <h2 className="print-slip-title">
          <PrintText text={title} />
        </h2>
      ) : null}
      <SlipFacts slip={slip} />
      <table className="print-table print-table-dense">
        <thead>
          <tr>
            {slip.columns.map((column) => (
              <th
                key={column.key}
                scope="col"
                data-align={alignOf(column.kind)}
                className={cellClass(column.kind)}
              >
                {l10n.getString(column.title_id)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {slip.rows.map((row, index) => (
            <tr key={index} data-row={row.kind}>
              {row.cells.map((cell, position) => (
                <td
                  key={position}
                  data-align={alignOf(kindAt(slip.columns, position))}
                  className={columnClass(slip.columns, position)}
                >
                  <Cell cell={cell} />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <SlipLines lines={slip.summary} />
      {parchi ? (
        <p className="print-parchi-collect">
          {l10n.getString('print-parchi-collect')} <span className="print-write-line" />
        </p>
      ) : null}
    </article>
  );
}

interface DaySlipsDocumentProps {
  data: DaySlipsPrintData;
  variant: DaySlipsVariant;
  /**
   * True in the hidden print window: the root element then takes the print language, direction and
   * footer. False in the in-app preview, which must not change the app's own language.
   */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

function DaySlipsBody({ data, variant }: Pick<DaySlipsDocumentProps, 'data' | 'variant'>) {
  const { l10n } = useLocalization();
  const { frame } = data;
  if (variant === 'parchi') {
    return (
      <div className="print-report print-parchi-page" data-variant="parchi">
        <div className="print-parchi-grid">
          {data.buyers.map((slip, index) => (
            <PartySlip key={index} slip={slip} parchi firm={frame.header.firm.name} />
          ))}
        </div>
        <PrintScreenFooter footer={frame.footer} />
      </div>
    );
  }
  // Each seller's bill is a sheet of its own, with the header of every print and his name in the title.
  return (
    <div className="print-report print-parchi-page" data-variant="seller-bills">
      {data.sellers.map((slip, index) => (
        <section key={index} className="print-sheet">
          <PrintHeader
            header={frame.header}
            title={slipTitle(slip, (id, vars) => l10n.getString(id, vars)) ?? undefined}
          />
          <PartySlip slip={slip} parchi={false} firm={frame.header.firm.name} showTitle={false} />
          <PrintScreenFooter footer={frame.footer} />
        </section>
      ))}
    </div>
  );
}

export function DaySlipsDocument({ data, variant, applyRoot = false, ref }: DaySlipsDocumentProps) {
  return (
    <PrintRoot
      frame={data.frame}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <DaySlipsBody data={data} variant={variant} />
    </PrintRoot>
  );
}
