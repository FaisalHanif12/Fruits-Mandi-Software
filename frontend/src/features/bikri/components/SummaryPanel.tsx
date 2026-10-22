// The live summary (PRD 12.5): the seller's side, the buyer's side and the partners' share, for
// the selected lot or for the whole بکری ("پوری بکری", FR-12.1). Every line is the backend's
// number, shown with formatPaisa, and has its ⓘ with the backend's working (FR-7.43, F42). A
// net below zero is red with "بیوپاری کے ذمے" and is allowed (FR-12.7). The rate difference shows
// only when it is not 0. The cost, the trip partners' parts and the result of a lot of trip goods
// are in the panel of that lot (OwnLotPanel), not here (PRD FR-12.26). Nothing is added up here
// (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import type { Working } from '../../../bindings/Working';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { CalcInfo, Num, SegmentedControl } from '../../../shared/ui';
import { useLocalization } from '@fluent/react';
import type { SummaryTotals } from '../types';
import type { Paisa } from '../../../shared/format';

interface Line {
  labelId: string;
  paisa: Paisa;
  working: Working;
  /** Shown in red: a net below zero. */
  emphasis?: 'total' | 'negative';
}

interface SummaryPanelProps {
  totals: SummaryTotals | null;
  /** Which part is shown: the selected lot or the whole بکری. */
  whole: boolean;
  lotNumber: number | null;
  /** Without it there is one lot only (a later sale) and the lot / whole switch is not drawn. */
  onWholeChange?: ((whole: boolean) => void) | undefined;
  /** The net is below zero: the seller owes the firm (PRD FR-12.7). */
  netIsNegative: boolean;
}

function sellerLines(totals: SummaryTotals): Line[] {
  const { workings } = totals;
  return [
    { labelId: 'glossary-kham-bikri', paisa: totals.gross, working: workings.gross },
    { labelId: 'glossary-kiraya', paisa: totals.freight, working: workings.freight },
    { labelId: 'glossary-commission', paisa: totals.seller_comm, working: workings.seller_comm },
    { labelId: 'glossary-mazdoori', paisa: totals.labour, working: workings.labour },
    { labelId: 'glossary-munshiana', paisa: totals.munshiana, working: workings.munshiana },
    { labelId: 'glossary-naqd', paisa: totals.cash_paid, working: workings.cash_paid },
    { labelId: 'glossary-kiraya-store', paisa: totals.store_rent, working: workings.store_rent },
    { labelId: 'glossary-phone', paisa: totals.phone, working: workings.phone },
    {
      labelId: 'bikri-summary-deductions',
      paisa: totals.deductions,
      working: workings.deductions,
      emphasis: 'total',
    },
    {
      labelId: 'glossary-safi-bikri',
      paisa: totals.net,
      working: workings.net,
      emphasis: 'total',
    },
  ];
}

function buyerLines(totals: SummaryTotals, hasRateDiff: boolean): Line[] {
  const { workings } = totals;
  return [
    { labelId: 'bikri-summary-buyer-comm', paisa: totals.buyer_comm, working: workings.buyer_comm },
    { labelId: 'glossary-laga', paisa: totals.laga, working: workings.laga },
    ...(hasRateDiff
      ? [{ labelId: 'glossary-rate-farq', paisa: totals.rate_diff, working: workings.rate_diff }]
      : []),
    { labelId: 'glossary-udhari', paisa: totals.buyer_credit, working: workings.buyer_credit },
    { labelId: 'glossary-naqdi', paisa: totals.buyer_cash, working: workings.buyer_cash },
    {
      labelId: 'bikri-summary-buyer-total',
      paisa: totals.buyer_total,
      working: workings.buyer_total,
      emphasis: 'total',
    },
  ];
}

function Rows({ lines, negativeNet }: { lines: readonly Line[]; negativeNet: boolean }) {
  return (
    <dl className="flex flex-col">
      {lines.map((line) => {
        const isNet = line.labelId === 'glossary-safi-bikri';
        const red = isNet && negativeNet;
        return (
          <div
            key={line.labelId}
            className={`flex items-center justify-between gap-3 border-b border-border py-1 ${line.emphasis === 'total' ? 'type-label' : 'type-body'} ${red ? 'text-due' : 'text-ink'}`}
          >
            <Localized id={line.labelId}>
              <dt />
            </Localized>
            <dd className="flex items-center gap-1">
              <Num paisa={line.paisa} />
              <CalcInfo working={line.working} inCell />
            </dd>
          </div>
        );
      })}
    </dl>
  );
}

export function SummaryPanel({
  totals,
  whole,
  lotNumber,
  onWholeChange,
  netIsNegative,
}: SummaryPanelProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  // The rate difference shows only when it is not 0 (PRD 12.5). The test is on the text the
  // backend's number is written as, so nothing is compared as a number here.
  const hasRateDiff = totals !== null && formatPaisa(totals.rate_diff, { grouping }) !== '0';
  return (
    <section
      aria-labelledby="bikri-summary-heading"
      className="flex min-w-0 flex-col gap-3 rounded-3 border border-border bg-surface p-4"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="bikri-summary-title">
          <h2 id="bikri-summary-heading" className="type-section text-ink" />
        </Localized>
        {onWholeChange === undefined ? null : (
          <SegmentedControl
            groupLabel={l10n.getString('bikri-summary-title')}
            value={whole ? 'whole' : 'lot'}
            onChange={(next) => {
              onWholeChange(next === 'whole');
            }}
            options={[
              {
                value: 'lot',
                label: l10n.getString('bikri-summary-lot', { lot: String(lotNumber ?? 1) }),
              },
              { value: 'whole', label: l10n.getString('bikri-summary-whole') },
            ]}
          />
        )}
      </div>
      {totals === null ? (
        <Localized id="bikri-summary-incomplete">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <div className="grid grid-cols-2 gap-x-4 gap-y-3">
          <div className="flex flex-col gap-1">
            <Localized id="bikri-summary-seller-side">
              <h3 className="type-label text-ink-muted" />
            </Localized>
            <Rows lines={sellerLines(totals)} negativeNet={netIsNegative} />
            {netIsNegative ? (
              <p className="flex items-center gap-2 type-body text-due">
                <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
                <Localized id="bikri-summary-net-negative">
                  <span />
                </Localized>
              </p>
            ) : null}
          </div>
          <div className="flex flex-col gap-1">
            <Localized id="bikri-summary-buyer-side">
              <h3 className="type-label text-ink-muted" />
            </Localized>
            <Rows lines={buyerLines(totals, hasRateDiff)} negativeNet={false} />
          </div>
        </div>
      )}
    </section>
  );
}
