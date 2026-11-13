// One party's statement on paper, drawn on screen in the print language (plan 74 T3, PRD FR-12.47,
// FR-19.5, FR-19.7, FR-19.13): the ONE header of the app, who it is for, then ONE table. The table has a
// row group for the opening balance, one tight block for each date with an entry (the lines of goods, the
// commission and لاگا on one line, the day's bill, what was paid on the spot, each وصولی or ادائیگی, and the
// balance at the end of the day), and a row group for the totals of the period with the closing balance and
// its amount in words. The blocks are oldest first and never split across two pages where it can be helped;
// the heading row repeats on every page; "Page x of y" is the footer of the page (the `PrintRoot` frame).
// Every figure, date and side is already text from the backend: this file lays them out and adds up nothing
// (CLAUDE.md rule 1). The in-app preview and the hidden print window draw this same page.

import { useLocalization, Localized } from '@fluent/react';
import { Fragment, type ReactNode, type Ref } from 'react';
import type { SaleGrade } from '../../../bindings/SaleGrade';
import type { PrintFrame } from '../../../shared/print';
import { PrintHeader, PrintPartyBlock, PrintRoot, PrintScreenFooter } from '../../../shared/print';
import { Num } from '../../../shared/ui/Num';
import { ScriptText } from '../../../shared/ui/ScriptText';
import type { StatementBalance } from '../../../bindings/StatementBalance';
import type { StatementDay } from '../../../bindings/StatementDay';
import type { StatementEntry } from '../../../bindings/StatementEntry';
import type { StatementItem } from '../../../bindings/StatementItem';
import type { StatementKind } from '../../../bindings/StatementKind';
import type { StatementPrintData } from '../../../bindings/StatementPrintData';

const GRADE_WORD: Record<SaleGrade, string> = {
  first: 'glossary-darja-awwal',
  second: 'glossary-darja-doyam',
  third: 'glossary-darja-soyam',
};

const SIDE_WORD = { debit: 'glossary-banaam', credit: 'glossary-jama' } as const;

/**
 * A balance: its amount as a number, centred under the heading, and the word of its side hanging on the
 * physical right of it in both languages (the same slot as the balances of the reports, reports.css). At 0
 * there is no side.
 */
function Balance({ balance }: { balance: StatementBalance }) {
  const { l10n } = useLocalization();
  return (
    <span className="print-balance">
      <Num>{balance.amount}</Num>
      <span className="print-balance-side">
        {balance.side === 'none' ? null : (
          <ScriptText>{l10n.getString(SIDE_WORD[balance.side])}</ScriptText>
        )}
      </span>
    </span>
  );
}

/** One line of goods, "Mango 20 × 1,000 = 20,000": the words come from Fluent, the numbers are numbers. */
function ItemLine({ item }: { item: StatementItem }) {
  const { l10n } = useLocalization();
  const name =
    item.grade === null
      ? item.name
      : l10n.getString('common-item-grade', {
          item: item.name,
          grade: l10n.getString(GRADE_WORD[item.grade]),
        });
  return (
    <Localized
      id={item.weight_kg === null ? 'print-statement-item' : 'print-statement-item-weight'}
      vars={{
        item: name,
        qty: item.qty,
        weight: item.weight_kg ?? '',
        rate: item.rate,
        amount: item.amount,
      }}
      elems={{ item: <ScriptText />, num: <Num inherit /> }}
    >
      <span />
    </Localized>
  );
}

/** Commission and لاگا (a seller's deductions) on one line: each label with its amount. */
function ChargesLine({ day }: { day: StatementDay }) {
  const { l10n } = useLocalization();
  return (
    <span className="print-statement-charges">
      {day.charges.map((charge) => (
        <span key={charge.label_id}>
          {l10n.getString(charge.label_id)} <Num>{charge.amount}</Num>
        </span>
      ))}
    </span>
  );
}

function EntryLabel({ entry }: { entry: StatementEntry }) {
  const { l10n } = useLocalization();
  return (
    <>
      {l10n.getString(entry.label_id)}
      {entry.note === null || entry.note === '' ? null : (
        <>
          {' '}
          <span className="print-statement-note">
            <ScriptText>{entry.note}</ScriptText>
          </span>
        </>
      )}
    </>
  );
}

/** One row of a day's block, without its date: the date is drawn once, in the first row. */
interface DayRow {
  key: string;
  className?: string;
  cells: ReactNode;
}

/** The rows of a day, in the order they are printed: the lines, the charges, the bill, the paid, the entries, the end. */
function rowsOf(
  day: StatementDay,
  billLabel: string | null,
  paidLabel: string,
  endLabel: string,
): DayRow[] {
  const rows: DayRow[] = day.items.map((item, index) => ({
    key: `item-${String(index)}`,
    cells: (
      <td colSpan={3} className="print-statement-line">
        <ItemLine item={item} />
      </td>
    ),
  }));
  if (day.charges.length > 0) {
    rows.push({
      key: 'charges',
      cells: (
        <td colSpan={3} className="print-statement-line">
          <ChargesLine day={day} />
        </td>
      ),
    });
  }
  // A day with a receipt or a payment but no sale line has no bill (the backend sends none).
  if (day.bill !== null && billLabel !== null) {
    rows.push({
      key: 'bill',
      className: 'print-statement-bill',
      cells: (
        <>
          <td>{billLabel}</td>
          <td className="print-num">
            <Num>{day.bill.amount}</Num>
          </td>
          <td />
        </>
      ),
    });
  }
  if (day.paid_on_spot !== null) {
    rows.push({
      key: 'paid',
      cells: (
        <>
          <td>{paidLabel}</td>
          <td className="print-num">
            <Num>{day.paid_on_spot}</Num>
          </td>
          <td />
        </>
      ),
    });
  }
  day.entries.forEach((entry, index) => {
    rows.push({
      key: `entry-${String(index)}`,
      cells: (
        <>
          <td>
            <EntryLabel entry={entry} />
          </td>
          <td className="print-num">
            <Num>{entry.amount}</Num>
          </td>
          <td />
        </>
      ),
    });
  });
  rows.push({
    key: 'end',
    className: 'print-statement-end',
    cells: (
      <>
        <td colSpan={2}>{endLabel}</td>
        <td className="print-num">
          <Balance balance={day.closing} />
        </td>
      </>
    ),
  });
  return rows;
}

/** One date with an entry: a tight block of rows, the date once at its start, covering the whole block. */
function DayBlock({ day, kind }: { day: StatementDay; kind: StatementKind }) {
  const { l10n } = useLocalization();
  const rows = rowsOf(
    day,
    day.bill === null ? null : l10n.getString(day.bill.label_id),
    // What a buyer paid on the spot, and the نقد given to a seller with his lots: the same figure, two words.
    l10n.getString(kind === 'seller' ? 'print-cash-paid' : 'print-statement-paid-on-spot'),
    l10n.getString('print-statement-day-end'),
  );
  return (
    <tbody className="print-statement-day">
      {rows.map((row, index) => (
        <tr key={row.key} {...(row.className === undefined ? {} : { className: row.className })}>
          {index === 0 ? (
            <td rowSpan={rows.length} className="print-num print-statement-date">
              <Num>{day.date}</Num>
            </td>
          ) : null}
          {row.cells}
        </tr>
      ))}
    </tbody>
  );
}

function StatementBody({ data, frame }: { data: StatementPrintData; frame: PrintFrame }) {
  const { l10n } = useLocalization();
  return (
    <div className="print-report">
      <section className="print-sheet">
        <PrintHeader header={frame.header} />
        <PrintPartyBlock party={data.party} />
        <table className="print-table print-table-dense print-statement">
          <thead>
            <tr>
              <th scope="col" className="print-num print-statement-col-date">
                {l10n.getString('print-statement-col-date')}
              </th>
              <th scope="col">{l10n.getString('print-statement-col-details')}</th>
              <th scope="col" className="print-num print-statement-col-amount">
                {l10n.getString('print-statement-col-amount')}
              </th>
              <th scope="col" className="print-num print-statement-col-balance">
                {l10n.getString('print-statement-col-balance')}
              </th>
            </tr>
          </thead>
          <tbody className="print-statement-opening">
            <tr>
              <td colSpan={3}>{l10n.getString('print-statement-opening')}</td>
              <td className="print-num">
                <Balance balance={data.opening} />
              </td>
            </tr>
          </tbody>
          {data.days.map((day) => (
            <DayBlock key={day.date} day={day} kind={data.kind} />
          ))}
          <tbody className="print-statement-totals">
            {/* A period with no entry has no totals to show: only the balance carried through it. */}
            {(data.days.length === 0 ? [] : data.totals).map((total) => (
              <Fragment key={total.label_id}>
                <tr>
                  <td colSpan={2}>{l10n.getString(total.label_id)}</td>
                  <td className="print-num">
                    <Num>{total.amount}</Num>
                  </td>
                  <td />
                </tr>
              </Fragment>
            ))}
            <tr className="print-statement-closing">
              <td colSpan={3}>{l10n.getString('print-statement-total-closing')}</td>
              <td className="print-num">
                <Balance balance={data.closing} />
              </td>
            </tr>
          </tbody>
        </table>
        {data.words === null ? null : (
          <p className="print-words">
            <span className="print-words-label">{l10n.getString(data.words.label_id)}</span>
            <span className="print-words-text">{data.words.text}</span>
          </p>
        )}
        {/* On paper the footer is the page margin (print.css). This line is for the preview. */}
        <PrintScreenFooter footer={frame.footer} />
      </section>
    </div>
  );
}

interface StatementPrintDocumentProps {
  data: StatementPrintData;
  /** The header and footer the backend built for this print (`reports_print_header`). */
  frame: PrintFrame;
  /**
   * True in the hidden print window: the root element then takes the print language, direction and footer,
   * so the page margin boxes are right. False in the in-app preview, which must not change the app's own
   * language.
   */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

export function StatementPrintDocument({
  data,
  frame,
  applyRoot = false,
  ref,
}: StatementPrintDocumentProps) {
  return (
    <PrintRoot
      frame={frame}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <StatementBody data={data} frame={frame} />
    </PrintRoot>
  );
}
