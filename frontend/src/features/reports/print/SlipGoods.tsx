// The goods of a party's day slip on the 80 mm roll (PRD FR-19.12, plan 42). A roll has no room for
// five columns, so each line is two rows of text, like the goods of the bill on the roll: the item and
// packing with the amount, then "20 @ 600" under them. The row after an item with more than one line
// ("Banana total") has the item total and its amount, then its quantity. A line of an item sold by weight
// has its kilos and its rate per kilo under it ("12.345 kg @ 100"; PRD A73). Every figure is the backend's
// text; this file picks the cells by their column key and adds up nothing (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import { NumberRuns } from '../../../shared/grid/NumberRuns';
import { Code } from '../../../shared/ui/Code';
import { Num } from '../../../shared/ui/Num';
import type { ReportPrintCell } from '../../../bindings/ReportPrintCell';
import type { ReportRowKind } from '../../../bindings/ReportRowKind';
import { Cell } from './PrintCell';

interface SlipGoodsProps {
  columns: readonly { key: string }[];
  rows: readonly { cells: readonly ReportPrintCell[]; kind: ReportRowKind }[];
}

/** The text of a figure cell the backend wrote, or an empty text for a cell with none. */
function figure(cell: ReportPrintCell | undefined): string {
  return cell !== undefined && (cell.type === 'number' || cell.type === 'money') ? cell.text : '';
}

export function SlipGoods({ columns, rows }: SlipGoodsProps) {
  const { l10n } = useLocalization();
  const at = (key: string) => columns.findIndex((column) => column.key === key);
  return (
    <ul className="print-compact">
      {rows.map((row, index) => {
        const item = row.cells[at('item')];
        const packing = row.cells[at('packing')];
        // The quality of the line (PRD FR-12.42): a column that is there only when a line of the slip has
        // one, and a cell that is empty for a line with none.
        const grade = row.cells[at('grade')];
        const qty = row.cells[at('qty')];
        const rate = row.cells[at('rate')];
        const amount = row.cells[at('amount')];
        // The kilos of a weighed line: the column is there only when some line of the slip is weighed.
        const weight = row.cells[at('weight')];
        const kilos = weight?.type === 'weight' ? weight.text : null;
        const total = row.kind === 'item_total';
        return (
          <li
            key={index}
            className={total ? 'print-compact-line print-compact-total' : 'print-compact-line'}
            data-row={row.kind}
          >
            <div className="print-compact-row">
              <span className="print-compact-name">
                {item === undefined ? null : <Cell cell={item} />}
                {grade === undefined || grade.type === 'empty' ? null : (
                  <>
                    {' '}
                    <Cell cell={grade} />
                  </>
                )}
                {packing === undefined || packing.type === 'empty' ? null : (
                  <>
                    {' '}
                    <Cell cell={packing} />
                  </>
                )}
              </span>
              <Num>{figure(amount)}</Num>
            </div>
            <div className="print-compact-row print-compact-sub">
              {/* "(20 @ 600)" is always written left to right (NFR-URD-13); an item total has no rate. */}
              {total ? (
                <>
                  <Num>{figure(qty)}</Num>
                  {kilos === null ? null : (
                    <NumberRuns text={l10n.getString('print-slip-weight-kg', { weight: kilos })} />
                  )}
                </>
              ) : kilos === null ? (
                <Code>{`${figure(qty)} @ ${figure(rate)}`}</Code>
              ) : (
                <NumberRuns
                  text={l10n.getString('print-slip-weight-line', {
                    weight: kilos,
                    rate: figure(rate),
                  })}
                />
              )}
            </div>
          </li>
        );
      })}
    </ul>
  );
}
