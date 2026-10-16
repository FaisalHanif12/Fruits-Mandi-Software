// The key numbers pinned to the bottom of the screen in compact and normal windows, so the munshi
// sees them while he types in the grids above (the summary is under the grids there). It shows
// the selected lot and the whole بکری: خام بکری, کل اخراجات, صافی بکری and the buyers' bills, and
// the cash note when cash would go below zero. Every number is the backend's, taken straight from
// the preview with formatPaisa; nothing is added up here (CLAUDE.md rule 1).
//
// It is pinned 24 px into the bottom padding of the scrolled area (the padding is 32 px), so it sits
// just above the status bar with a gap of 8 px. `main` keeps a clearance under a focused cell
// (`scroll-pb-strip`, token --strip-clearance), so the strip never covers the cell being typed in.
//
// It holds no button, link or field, so the Tab key and the arrow keys pass over it (it is not a
// tab stop). It is a region with a name, not a live region, so a screen reader does not read it
// out on every key press. In a wide window the summary has its own column and this is not drawn.

import { Localized, useLocalization } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import type { ReactNode } from 'react';
import type { CashCheck } from '../../../bindings/CashCheck';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { Num } from '../../../shared/ui';
import type { SummaryTotals } from '../types';

interface Group {
  /** The words in front of the numbers: the lot, or the whole sale. */
  label: ReactNode;
  /** Null: the backend could not work this out yet (nothing finished in it). */
  totals: SummaryTotals | null;
  /** The net is below zero: the seller owes it (PRD FR-12.7). */
  netIsNegative: boolean;
}

interface NumbersStripProps {
  lotNumber: number;
  lot: Omit<Group, 'label'>;
  whole: Omit<Group, 'label'>;
  cash: CashCheck;
}

/** The four numbers of the strip: the Fluent ID of their words, and the number to show. */
const NUMBERS = [
  { id: 'glossary-kham-bikri', pick: (totals: SummaryTotals) => totals.gross, isNet: false },
  {
    id: 'bikri-summary-deductions',
    pick: (totals: SummaryTotals) => totals.deductions,
    isNet: false,
  },
  { id: 'glossary-safi-bikri', pick: (totals: SummaryTotals) => totals.net, isNet: true },
  {
    id: 'bikri-summary-buyer-total',
    pick: (totals: SummaryTotals) => totals.buyer_total,
    isNet: false,
  },
] as const;

function StripRow({ label, totals, netIsNegative }: Group) {
  return (
    <div className="flex flex-wrap items-baseline gap-x-4">
      <span className="type-label text-ink">{label}</span>
      {totals === null ? (
        <Localized id="bikri-strip-incomplete">
          <span className="type-body text-ink-muted" />
        </Localized>
      ) : (
        NUMBERS.map((number) => (
          <span
            key={number.id}
            className={`type-body inline-flex items-baseline gap-1 ${number.isNet && netIsNegative ? 'text-due' : 'text-ink'}`}
          >
            <Localized id={number.id}>
              <span className="text-ink-muted" />
            </Localized>
            <Num paisa={number.pick(totals)} />
          </span>
        ))
      )}
    </div>
  );
}

export function NumbersStrip({ lotNumber, lot, whole, cash }: NumbersStripProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  return (
    <section
      aria-label={l10n.getString('bikri-strip-label')}
      className="sticky -bottom-5 z-10 flex flex-col gap-1 rounded-3 border border-border bg-surface p-2 shadow-overlay"
    >
      <StripRow
        label={<Localized id="bikri-summary-lot" vars={{ lot: String(lotNumber) }} />}
        {...lot}
      />
      <StripRow label={<Localized id="bikri-summary-whole" />} {...whole} />
      {cash.needs_confirm ? (
        <p className="flex items-center gap-2 type-body text-warn">
          <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
          <Localized
            id="bikri-cash-after"
            vars={{ amount: formatPaisa(cash.cash_after_paisa, { grouping }) }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        </p>
      ) : null}
    </section>
  );
}
