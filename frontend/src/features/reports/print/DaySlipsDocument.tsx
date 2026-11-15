// The buyer parchi and the overall sale print on screen, in the print language (PRD FR-19.15, FR-12.12,
// FR-19.13, plan 76 point 3). Both are made of party slips, the R35 slip of each party for one day, which
// the backend sends ready (`DaySlipsPrintData`): its facts (the party, the day), the goods (item, packing,
// quantity, rate, amount), the labelled lines under them and the previous and new balance.
//
// - The parchi (`variant="parchi"`) has the buyers only. Several slips stand on one A4 page, two columns
//   (about six slips), each in a box with dashed cut lines, never cut across a page; each is short and ends
//   with an empty line for the collector to write the وصولی on. On A5 they stand one under another.
// - The overall sale print (`variant="sale"`) has the one header of every print, then the sellers' slips
//   (the items with their lot names, each deduction on its own line, the net, the previous and the new
//   balance), then the buyers' slips (his purchases, paid on the spot, previous, paid that day, total
//   now). A slip is never cut across a page.
//
// Every figure and date is written by the backend in the print language; this file lays them out and adds
// up nothing (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import type { Ref } from 'react';
import type { DaySlipsPrintData } from '../../../bindings/DaySlipsPrintData';
import type { ReportColumnKind } from '../../../bindings/ReportColumnKind';
import type { ReportPrintData } from '../../../bindings/ReportPrintData';
import { PrintHeader, PrintRoot, PrintScreenFooter, PrintText } from '../../../shared/print';
import { Code } from '../../../shared/ui/Code';
import { Cell, alignOf, cellClass } from './PrintCell';
import { SlipLines } from './SlipLines';

export type DaySlipsVariant = 'parchi' | 'sale';

/** The kind of the column at a position: a column the data does not name is plain text. */
function kindAt(columns: ReportPrintData['columns'], position: number): ReportColumnKind {
  return columns[position]?.kind ?? 'text';
}

/** The class of the cell at a column. */
function columnClass(columns: ReportPrintData['columns'], position: number): string {
  return cellClass(kindAt(columns, position));
}

/** The facts of a slip (the party and the day) as one line of label and value pairs. */
function SlipFacts({ slip }: { slip: ReportPrintData }) {
  const { l10n } = useLocalization();
  return (
    <dl className="print-parchi-facts">
      {slip.filters.map((fact) => (
        <div key={fact.label_id} className="print-fact">
          <dt>{l10n.getString(fact.label_id)}</dt>
          <dd>
            {/\p{Script=Arabic}/u.test(fact.value) ? (
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
}: {
  slip: ReportPrintData;
  parchi: boolean;
  /** The firm's name: it heads a parchi, which stands alone on a page with other slips. */
  firm: string;
}) {
  const { l10n } = useLocalization();
  return (
    <article className="print-parchi-slip" data-slip="">
      {parchi ? <p className="print-parchi-firm">{firm}</p> : null}
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
  return (
    <div className="print-report print-parchi-page" data-variant="sale">
      <section className="print-sheet">
        <PrintHeader header={frame.header} />
        {data.sellers.length === 0 ? null : (
          <section className="print-parchi-part" data-part="sellers">
            <h2 className="print-day-table-title">{l10n.getString('print-sale-sellers')}</h2>
            <div className="print-parchi-grid">
              {data.sellers.map((slip, index) => (
                <PartySlip key={index} slip={slip} parchi={false} firm={frame.header.firm.name} />
              ))}
            </div>
          </section>
        )}
        {data.buyers.length === 0 ? null : (
          <section className="print-parchi-part" data-part="buyers">
            <h2 className="print-day-table-title">{l10n.getString('print-sale-buyers')}</h2>
            <div className="print-parchi-grid">
              {data.buyers.map((slip, index) => (
                <PartySlip key={index} slip={slip} parchi={false} firm={frame.header.firm.name} />
              ))}
            </div>
          </section>
        )}
        <PrintScreenFooter footer={frame.footer} />
      </section>
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
