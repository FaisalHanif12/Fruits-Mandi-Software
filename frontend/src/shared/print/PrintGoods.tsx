// The goods of one page: a table with a repeating header and rows that never split, or, on the
// 80 mm roll, two lines of text for each line of goods (a roll has no room for columns).
// It lays out what it is given. It adds up nothing and works out nothing (CLAUDE.md rule 1):
// every quantity, rate and amount is a string already made by the backend.

import { useLocalization } from '@fluent/react';
import { Code } from '../ui/Code';
import { Num } from '../ui/Num';
import { ScriptText } from '../ui/ScriptText';
import { PrintName } from './PrintName';
import { COLUMN_HEAD_ID, type Paper, type PrintColumn, type PrintLine } from './types';

interface PrintGoodsProps {
  lines: readonly PrintLine[];
  columns: readonly PrintColumn[];
  paper: Paper;
}

/** The column types that hold figures: they sit on the physical right edge. */
const FIGURE_COLUMNS: ReadonlySet<PrintColumn> = new Set(['qty', 'weight', 'rate', 'amount']);

function figureClass(column: PrintColumn): string {
  if (!FIGURE_COLUMNS.has(column)) return '';
  return column === 'qty' ? 'print-num print-col-qty' : 'print-num print-col-num';
}

function Cell({ column, line }: { column: PrintColumn; line: PrintLine }) {
  switch (column) {
    case 'date':
      return <Num>{line.date}</Num>;
    case 'buyer':
      return <PrintName name={line.name ?? ''} tag={line.name_tag} />;
    case 'item':
      return <ScriptText>{line.item}</ScriptText>;
    case 'packing':
      return <ScriptText>{line.packing}</ScriptText>;
    case 'qty':
      return <Num>{line.qty}</Num>;
    case 'weight':
      return <Num>{line.weight_kg ?? ''}</Num>;
    case 'rate':
      return <Num>{line.rate}</Num>;
    case 'amount':
      return <Num>{line.amount}</Num>;
  }
}

function GoodsTable({ lines, columns }: Omit<PrintGoodsProps, 'paper'>) {
  const { l10n } = useLocalization();
  return (
    <table className="print-table">
      <thead>
        <tr>
          {columns.map((column) => (
            <th key={column} scope="col" className={figureClass(column)}>
              {l10n.getString(COLUMN_HEAD_ID[column])}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {lines.map((line, index) => (
          // The same goods can be on two lines (two sales of one lot), so the position is the key.
          <LineRows key={index} line={line} columns={columns} />
        ))}
      </tbody>
    </table>
  );
}

function LineRows({ line, columns }: { line: PrintLine; columns: readonly PrintColumn[] }) {
  return (
    <>
      <tr>
        {columns.map((column) => (
          <td key={column} className={figureClass(column)}>
            <Cell column={column} line={line} />
          </td>
        ))}
      </tr>
      {line.note === null ? null : (
        <tr>
          <td colSpan={columns.length} className="print-line-note-cell">
            <bdi dir="auto">{line.note}</bdi>
          </td>
        </tr>
      )}
    </>
  );
}

function CompactLines({ lines, columns }: Omit<PrintGoodsProps, 'paper'>) {
  const showDate = columns.includes('date');
  const showBuyer = columns.includes('buyer');
  return (
    <ul className="print-compact">
      {lines.map((line, index) => (
        <li key={index} className="print-compact-line">
          <div className="print-compact-row">
            <span className="print-compact-name">
              {showBuyer && line.name !== null ? (
                <>
                  <PrintName name={line.name} tag={line.name_tag} />{' '}
                </>
              ) : null}
              <ScriptText>{line.item}</ScriptText> <ScriptText>{line.packing}</ScriptText>
            </span>
            {showDate ? <Num>{line.date}</Num> : null}
          </div>
          <div className="print-compact-row print-compact-sub">
            {/* "(16 @ 900)" is always written left to right (NFR-URD-13). */}
            <Code>{`${line.qty} @ ${line.rate}${line.weight_kg === null ? '' : ` · ${line.weight_kg}`}`}</Code>
            <Num>{line.amount}</Num>
          </div>
          {line.note === null ? null : (
            <div className="print-compact-sub">
              <bdi dir="auto">{line.note}</bdi>
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

export function PrintGoods({ lines, columns, paper }: PrintGoodsProps) {
  return paper === 'mm80' ? (
    <CompactLines lines={lines} columns={columns} />
  ) : (
    <GoodsTable lines={lines} columns={columns} />
  );
}
