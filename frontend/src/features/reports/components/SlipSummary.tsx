// The labelled lines under the goods of a party's day slip on screen (PRD FR-19.12, v1.12): the
// buyer's goods value, commission, لاگا, today's total, paid today, what is left, the previous and the
// new balance; the seller's gross sale, deductions and net sale of the day. A card of labels and
// values: the figures end on one physical right edge and a balance has its side word (بنام or جمع) in
// a slot after it, so every figure stands under the one above (NFR-USE-19, plan 39 R8). Every line,
// figure and side is the backend's; nothing is added up here (CLAUDE.md rule 1). A line the backend
// sent EMPTY (nothing paid yet) is a blank for the collector to write on, never a zero.

import { Localized } from '@fluent/react';
import type { ReportCell } from '../../../bindings/ReportCell';
import type { ReportSummaryLine } from '../../../bindings/ReportSummaryLine';
import { Num } from '../../../shared/ui';
import { sideMessageId } from '../../../shared/ui/Balance';

/** The lines that close a block of the card: a rule above them and the label weight. */
const SUBTOTAL_IDS: ReadonlySet<string> = new Set([
  'report-r35-today-total',
  'report-r35-net-sale',
]);

/** The line the slip ends on, set apart from the rest. */
const TOTAL_IDS: ReadonlySet<string> = new Set(['report-r35-new-balance']);

function SlipValue({ cell }: { cell: ReportCell }) {
  switch (cell.type) {
    case 'empty':
      return <span className="slip-write-line" aria-hidden="true" />;
    case 'balance': {
      const side = sideMessageId(cell.side);
      return (
        <>
          <Num paisa={cell.paisa} />
          <span className="slip-side">{side === null ? null : <Localized id={side} />}</span>
        </>
      );
    }
    case 'money':
      return (
        <>
          <Num paisa={cell.paisa} />
          <span className="slip-side" />
        </>
      );
    case 'number':
      return (
        <>
          <Num>{String(cell.value)}</Num>
          <span className="slip-side" />
        </>
      );
    default:
      return null;
  }
}

function lineClass(id: string): string {
  if (TOTAL_IDS.has(id)) return 'type-section border-t-2 border-ink pt-2';
  if (SUBTOTAL_IDS.has(id)) return 'type-label border-t border-ink';
  return 'type-body';
}

export function SlipSummary({ lines }: { lines: readonly ReportSummaryLine[] }) {
  if (lines.length === 0) return null;
  return (
    <section className="flex w-slip-card max-w-full shrink-0 flex-col rounded-3 border border-border bg-surface p-4">
      <dl className="flex flex-col">
        {lines.map((line) => (
          <div
            key={line.label_id}
            className={`flex items-center justify-between gap-3 border-b border-border py-1 text-ink ${lineClass(line.label_id)}`}
          >
            <dt>
              <Localized id={line.label_id} />
            </dt>
            <dd className="value-col">
              <SlipValue cell={line.cell} />
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

/** True when the page is one party's day slip (the goods columns), not the table of every party. */
export function isSlipPage(page: { id: string; columns: readonly { key: string }[] }): boolean {
  return page.id === 'R35' && page.columns[0]?.key === 'item';
}
