// The cells of a printed report, in the print language (PRD FR-7.36, FR-19.13): one cell of the
// backend's print data as markup, and the class of its column. The printed report and the slips of all
// the credit buyers (R5) are laid out with these, so a figure, a date, a name and a sale line look the
// same on both. Every figure and date is already written by the backend in the print language; this
// file lays them out and adds up nothing (CLAUDE.md rule 1).

import { useLocalization, type ReactLocalization } from '@fluent/react';
import type { ReportColumnKind } from '../../../bindings/ReportColumnKind';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { ReportPrintDetailPart } from '../../../bindings/ReportPrintDetailPart';
import type { ReportPrintMessageArg } from '../../../bindings/ReportPrintMessageArg';
import { NumberRuns } from '../../../shared/grid/NumberRuns';
import { PrintName } from '../../../shared/print';
import { scriptOf } from '../../../shared/text/scriptHint';
import { Code } from '../../../shared/ui/Code';
import { Num } from '../../../shared/ui/Num';
import { monthMessageId } from '../cells';

/** The column kinds that hold figures: they sit on the physical right edge (NFR-USE-19). */
const FIGURE_KINDS: ReadonlySet<ReportColumnKind> = new Set([
  'number',
  'money',
  'balance',
  'percent',
]);

/** The message of a zero balance's side: a nil balance has no side word. */
const ZERO_SIDE_ID = 'report-side-zero';

function DetailLine({ part, shortSale }: { part: ReportPrintDetailPart; shortSale: boolean }) {
  const { l10n } = useLocalization();
  switch (part.kind) {
    case 'sale':
      // The buyers' list (R5) writes a line as the client's own print does: "(item qty . rate)".
      if (shortSale) {
        return (
          // Spaces round the dot stay, the line does not wrap, and each number is in Inter (F25).
          <p className="print-nowrap">
            <NumberRuns
              text={l10n.getString('report-detail-sale-short', {
                item: part.item,
                qty: part.qty,
                rate: part.rate,
              })}
            />
          </p>
        );
      }
      return (
        <p>
          <NumberRuns
            text={l10n.getString('report-detail-sale', {
              number: part.bikri_number,
              item: part.item,
              qty: part.qty,
              packing: part.packing,
              rate: part.rate,
              value: part.value,
              commission: part.commission,
              laga: part.laga,
            })}
          />
        </p>
      );
    case 'lot':
      return (
        <p>
          <NumberRuns
            text={l10n.getString('report-detail-lot', {
              number: part.bikri_number,
              item: part.item,
              qty: part.qty,
              packing: part.packing,
              gross: part.gross,
              deductions: part.deductions,
            })}
          />
        </p>
      );
    case 'cash_lent':
      return (
        <p>
          <NumberRuns text={l10n.getString('report-detail-cash-lent', { reason: part.reason })} />
        </p>
      );
    case 'text':
      return (
        <p>
          <bdi dir="auto">
            <NumberRuns text={part.text} />
          </bdi>
        </p>
      );
  }
}

/**
 * A Fluent message with the arguments of a print `message` cell. An argument is a text the backend
 * already wrote, or another message that is worked out first (a voucher number inside a sentence).
 */
function printMessageText(
  l10n: ReactLocalization,
  id: string,
  args: readonly ReportPrintMessageArg[],
): string {
  const vars: Record<string, string> = {};
  for (const arg of args) {
    vars[arg.name] =
      arg.value.type === 'text'
        ? arg.value.text
        : printMessageText(l10n, arg.value.id, arg.value.args);
  }
  return l10n.getString(id, vars);
}

export function Cell({ cell, shortSale = false }: { cell: ReportPrintCell; shortSale?: boolean }) {
  const { l10n } = useLocalization();
  switch (cell.type) {
    case 'empty':
      return null;
    case 'text':
      return (
        <bdi dir="auto">
          <NumberRuns text={cell.text} />
        </bdi>
      );
    case 'label':
      return <NumberRuns text={l10n.getString(cell.id)} />;
    case 'number':
    case 'money':
    case 'percent':
    case 'date':
      return <Num>{cell.text}</Num>;
    case 'month':
      return (
        <>
          {l10n.getString(monthMessageId(cell.month))} <Num>{cell.year}</Num>
        </>
      );
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
      // The name, then the tag in its own isolate (PRD 10.2, FR-19.13): the shared `PrintName`.
      return (
        <>
          <Code>{party.short_code ?? party.code}</Code>{' '}
          <PrintName name={party.name} tag={party.tag ?? undefined} />
        </>
      );
    }
    case 'names':
      // The script of the whole name; a date inside it keeps its digit order (F25).
      return (
        <bdi lang={scriptOf(cell.text)}>
          <NumberRuns text={cell.text} />
        </bdi>
      );
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
            <DetailLine key={index} part={part} shortSale={shortSale} />
          ))}
        </div>
      );
    case 'message':
      // A sentence of a partner statement line: the backend wrote every figure in the print
      // language, so the message only puts the words around them (PRD FR-17.10).
      return <NumberRuns text={printMessageText(l10n, cell.id, cell.args)} />;
  }
}

/** Figures sit on the right edge; a date and the kind of an entry stay on one line. */
export function cellClass(kind: ReportColumnKind): string {
  if (FIGURE_KINDS.has(kind)) return 'print-num';
  if (kind === 'date' || kind === 'month') return 'print-col-date print-nowrap';
  if (kind === 'doc') return 'print-nowrap';
  return '';
}
