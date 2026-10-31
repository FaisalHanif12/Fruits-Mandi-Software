// The labelled lines under the goods of a party's day slip (PRD FR-19.12, v1.12): for a buyer, the
// goods value, commission, لاگا, today's total, other entries, paid today, what is left today, the
// previous balance, the new balance and the total نگ; for a seller, the gross sale, each deduction,
// the net sale of the day and the same lines after it. They are a card of labels and values, the
// figures in one column on the physical right edge with the side word (بنام or جمع) after a balance in
// a slot of its own, so every figure stands under the one above (NFR-USE-19, plan 39 R8). Every line,
// every figure and every side comes from the backend; this file draws them and adds up nothing
// (CLAUDE.md rule 1). A line the backend sent EMPTY (nothing paid yet) is a blank to write in by hand,
// never a zero.

import { useLocalization } from '@fluent/react';
import type { ReportId } from '../../../bindings/ReportId';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { ReportPrintSummaryLine } from '../../../bindings/ReportPrintSummaryLine';
import { Num } from '../../../shared/ui/Num';
import { Cell, ZERO_SIDE_ID } from './PrintCell';

/** The lines that close a block of the card: a rule above them and a strong line. */
const SUBTOTAL_IDS: ReadonlySet<string> = new Set([
  'report-r35-today-total',
  'report-r35-net-sale',
]);

/** The line the slip ends on: the new balance, with a double rule under it. */
const TOTAL_IDS: ReadonlySet<string> = new Set(['report-r35-new-balance']);

/**
 * True when the print is one party's day slip: the party day sheet (R35) with the goods columns
 * (item, packing, quantity, rate, amount) and not the table of every party, which starts with a code.
 */
export function isSlip(data: {
  report?: ReportId | undefined;
  columns: readonly { key: string }[];
}): boolean {
  return data.report === 'R35' && data.columns[0]?.key === 'item';
}

function rowClass(id: string): string {
  if (TOTAL_IDS.has(id)) return 'print-amount-total';
  return SUBTOTAL_IDS.has(id) ? 'print-amount-subtotal' : '';
}

/** A figure and, in the slot after it, the side word of a balance (empty for plain money). */
function SlipValue({ cell }: { cell: ReportPrintCell }) {
  const { l10n } = useLocalization();
  switch (cell.type) {
    case 'empty':
      // Nothing paid yet: a line for the collector to write the amount on.
      return <span className="print-write-line" aria-hidden="true" />;
    case 'balance':
      return (
        <span className="print-slip-value">
          <Num>{cell.text}</Num>
          <span className="print-balance-side">
            {cell.side_id === ZERO_SIDE_ID ? null : l10n.getString(cell.side_id)}
          </span>
        </span>
      );
    case 'money':
    case 'number':
      return (
        <span className="print-slip-value">
          <Num>{cell.text}</Num>
          <span className="print-balance-side" />
        </span>
      );
    default:
      return <Cell cell={cell} />;
  }
}

export function SlipLines({ lines }: { lines: readonly ReportPrintSummaryLine[] }) {
  const { l10n } = useLocalization();
  if (lines.length === 0) return null;
  return (
    <div className="print-summary">
      <table className="print-amounts print-slip-lines">
        <tbody>
          {lines.map((line) => (
            <tr key={line.label_id} className={rowClass(line.label_id)}>
              <th scope="row">{l10n.getString(line.label_id)}</th>
              <td className="print-num">
                <SlipValue cell={line.cell} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
