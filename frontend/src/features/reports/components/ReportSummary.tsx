// The lines above a report's table (PRD FR-19.5): the opening and closing balance of a ledger, the
// difference of a trial balance, the figures of a statement. The backend sent each value, with its
// side; nothing is worked out here (CLAUDE.md rule 1). A balance shows its side as a word, never by
// colour alone (NFR-USE-12).

import { Localized } from '@fluent/react';
import type { ReportCell } from '../../../bindings/ReportCell';
import type { ReportSummaryLine } from '../../../bindings/ReportSummaryLine';
import { Balance, Fact, Num } from '../../../shared/ui';

function SummaryValue({ cell }: { cell: ReportCell }) {
  switch (cell.type) {
    case 'balance':
      return <Balance balance={{ amount_paisa: cell.paisa, side: cell.side }} />;
    case 'money':
      return <Num paisa={cell.paisa} />;
    case 'number':
      return <Num>{String(cell.value)}</Num>;
    case 'text':
      return <>{cell.text}</>;
    default:
      return null;
  }
}

interface ReportSummaryProps {
  lines: readonly ReportSummaryLine[];
  /**
   * The lines as a card of their own (the footer of the party day sheet). Without it they are one
   * strip, a label and its figure side by side, under the days of the report (plan 38).
   */
  card?: boolean;
}

export function ReportSummary({ lines, card = false }: ReportSummaryProps) {
  if (lines.length === 0) return null;
  if (card) {
    return (
      <dl className="grid grid-cols-4 gap-4 rounded-3 border border-border bg-surface p-3">
        {lines.map((line) => (
          <Fact key={line.label_id} label={<Localized id={line.label_id} />}>
            <SummaryValue cell={line.cell} />
          </Fact>
        ))}
      </dl>
    );
  }
  return (
    <dl className="flex flex-wrap items-baseline gap-x-6 gap-y-1">
      {lines.map((line) => (
        <div key={line.label_id} className="flex items-baseline gap-2">
          <dt className="type-label text-ink-muted">
            <Localized id={line.label_id} />
          </dt>
          <dd className="type-field min-w-0 break-words text-ink">
            <SummaryValue cell={line.cell} />
          </dd>
        </div>
      ))}
    </dl>
  );
}
