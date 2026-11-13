// The میزان of the window of earlier days of پچھلی بکری, after its last day (PRD FR-12.46): the نگ sold and
// received, the خام بکری, the صافی بکری and the buyers' bills of the whole filter, as the backend counted
// them (`totals`, FR-19.11). Nothing is added up from the days or the rows here (CLAUDE.md rule 1). It
// is one soft band with the figures as pills, like the pills of a day, so it reads as the end of the list.

import { useLocalization } from '@fluent/react';
import type { BikriListTotals } from '../../../bindings/BikriListTotals';
import { Num } from '../../../shared/ui';
import { DayPill } from '../day-headings';

export function PastWindowTotals({ totals }: { totals: BikriListTotals }) {
  const { l10n } = useLocalization();
  return (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-3 bg-surface-sunken px-3 py-2">
      <span className="type-label text-ink">{l10n.getString('glossary-meezan')}</span>
      <DayPill label={l10n.getString('bikri-list-col-qty')}>
        <Num inherit>
          {l10n.getString('bikri-list-qty', {
            sold: String(totals.qty_sold),
            received: String(totals.qty_received),
          })}
        </Num>
      </DayPill>
      <DayPill label={l10n.getString('glossary-kham-bikri')}>
        <Num inherit paisa={totals.gross} />
      </DayPill>
      <DayPill label={l10n.getString('glossary-safi-bikri')}>
        <Num inherit paisa={totals.net} />
      </DayPill>
      <DayPill label={l10n.getString('bikri-list-col-buyer-total')}>
        <Num inherit paisa={totals.buyer_total} />
      </DayPill>
    </div>
  );
}
