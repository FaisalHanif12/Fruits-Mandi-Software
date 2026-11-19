// The live summary (PRD 12.5): the seller's side, the buyer's side and the partners' share, for
// the selected lot or for the whole بکری ("پوری بکری", FR-12.1). A dropdown at the top lists every lot
// (its number, seller and name) and "پوری بکری", so the munshi reaches any lot's summary at once without
// scrolling to the lots and back (plan 78 C2). Every line is the backend's
// number, shown with formatPaisa, and has its ⓘ with the backend's working (FR-7.43, F42). A
// net below zero is red with "بیوپاری کے ذمے" and is allowed (FR-12.7). The rate difference shows
// only when it is not 0. The cost, the trip partners' parts and the result of a lot of trip goods
// are in the panel of that lot (OwnLotPanel), not here (PRD FR-12.26). Nothing is added up here
// (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import type { ReactNode } from 'react';
import type { Working } from '../../../bindings/Working';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { CalcInfo, Field, Num, Select, type SelectOption } from '../../../shared/ui';
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

/** The dropdown's value for the whole بکری. A lot's value is its number (1, 2 ...), so it is never this. */
const WHOLE_VALUE = '0';

interface SummaryPanelProps {
  totals: SummaryTotals | null;
  /** Which part is shown: the selected lot or the whole بکری. */
  whole: boolean;
  /** The label of each lot of the بکری, in order ("لاٹ 2: فیصل F، کیلا-1"). Without them there is one lot only (a later sale) and no dropdown. */
  lots?: readonly string[] | undefined;
  /** The number of the lot whose summary is shown, from 1 (used while `whole` is false). */
  selectedLot?: number | undefined;
  /** A lot (its number, from 1) was chosen in the dropdown, or the whole بکری (null). */
  onChoose?: ((lot: number | null) => void) | undefined;
  /** The net is below zero: the seller owes the firm (PRD FR-12.7). */
  netIsNegative: boolean;
  /** How many lines of this part the figures leave out, their quantity being wrong (PRD FR-12.6). */
  leftOut?: number | undefined;
  /** The tick of each seller's advance (plan 78 C3), under the seller's lines. */
  advance?: ReactNode;
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
            <dd className="value-col">
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
  lots,
  selectedLot,
  onChoose,
  netIsNegative,
  leftOut = 0,
  advance,
}: SummaryPanelProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  // The rate difference shows only when it is not 0 (PRD 12.5). The test is on the text the
  // backend's number is written as, so nothing is compared as a number here.
  const hasRateDiff = totals !== null && formatPaisa(totals.rate_diff, { grouping }) !== '0';
  const choices: readonly SelectOption<string>[] | null =
    lots === undefined || onChoose === undefined
      ? null
      : [
          ...lots.map((label, index) => ({ value: String(index + 1), label })),
          { value: WHOLE_VALUE, label: l10n.getString('bikri-summary-whole') },
        ];
  return (
    <section
      aria-labelledby="bikri-summary-heading"
      className="flex min-w-0 flex-col gap-3 rounded-3 border border-border bg-surface p-4"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="bikri-summary-title">
          <h2 id="bikri-summary-heading" className="type-section text-ink" />
        </Localized>
        {choices === null || onChoose === undefined ? null : (
          // The one dropdown of the summary: every lot and the whole بکری. The arrow keys, Enter and Esc come
          // from the shared Select, so the screen writes no key handler of its own (Code Standards F27).
          <div className="min-w-0 max-w-form flex-1">
            <Field label={<Localized id="bikri-summary-which" />} hideLabel>
              {(control) => (
                <Select
                  {...control}
                  value={whole ? WHOLE_VALUE : String(selectedLot ?? 1)}
                  onChange={(value) => {
                    onChoose(value === WHOLE_VALUE ? null : Number(value));
                  }}
                  options={choices}
                />
              )}
            </Field>
          </div>
        )}
      </div>
      {/* The seller's tick shows whenever there is a summary, also while no line is worked out yet. */}
      {advance}
      {totals === null ? (
        // No line is worked out: when the reason is a quantity, the words say the totals wait for it.
        <Localized id={leftOut > 0 ? 'bikri-summary-waits-qty' : 'bikri-summary-incomplete'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        // The two sides stand side by side only when the panel itself is wide enough for a label, a figure
        // and its ⓘ on one line in each (the panel is a narrow third column at the wide step); else one
        // under the other. It follows the panel's own width, not the window's (container query).
        <div className="@container min-w-0">
          <div className="grid grid-cols-1 gap-x-4 gap-y-3 @form3:grid-cols-2">
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
        </div>
      )}
      {totals !== null && leftOut > 0 ? (
        <Localized id="bikri-left-out" vars={{ count: leftOut }}>
          <p role="status" className="flex items-center gap-2 type-body text-warn" />
        </Localized>
      ) : null}
    </section>
  );
}
