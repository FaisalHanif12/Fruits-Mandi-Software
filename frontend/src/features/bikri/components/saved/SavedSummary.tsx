// The summary of a saved بکری (PRD 12.5), for one lot or for the whole بکری, from the stored sums
// the backend sends (`SavedTotals`): the seller's side, the buyer's side and the shared income. It
// is the layout of the live summary of the sale book without the working (ⓘ), because nothing is
// worked out again: the numbers are the ones stored when the بکری was saved (Code Standards B20,
// CLAUDE.md rule 1). A net below zero is red with the words "owed by the seller" (PRD FR-12.7).

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { TriangleAlert } from 'lucide-react';
import type { BikriDetail } from '../../../../bindings/BikriDetail';
import type { SavedTotals } from '../../../../bindings/SavedTotals';
import { formatPaisa, useGrouping, type Paisa } from '../../../../shared/format';
import { Field, Num, SegmentedControl, Select } from '../../../../shared/ui';

interface Line {
  labelId: string;
  paisa: Paisa;
  emphasis?: 'total';
}

function sellerLines(totals: SavedTotals): Line[] {
  return [
    { labelId: 'glossary-kham-bikri', paisa: totals.gross },
    { labelId: 'glossary-kiraya', paisa: totals.freight },
    { labelId: 'glossary-commission', paisa: totals.seller_comm },
    { labelId: 'glossary-mazdoori', paisa: totals.labour },
    { labelId: 'glossary-munshiana', paisa: totals.munshiana },
    { labelId: 'glossary-naqd', paisa: totals.cash_paid },
    { labelId: 'glossary-kiraya-store', paisa: totals.store_rent },
    { labelId: 'glossary-phone', paisa: totals.phone },
    { labelId: 'bikri-summary-deductions', paisa: totals.deductions, emphasis: 'total' },
    { labelId: 'glossary-safi-bikri', paisa: totals.net, emphasis: 'total' },
  ];
}

function buyerLines(totals: SavedTotals, hasRateDiff: boolean): Line[] {
  return [
    { labelId: 'bikri-summary-buyer-comm', paisa: totals.buyer_comm },
    { labelId: 'glossary-laga', paisa: totals.laga },
    ...(hasRateDiff ? [{ labelId: 'glossary-rate-farq', paisa: totals.rate_diff }] : []),
    { labelId: 'glossary-udhari', paisa: totals.buyer_credit },
    { labelId: 'glossary-naqdi', paisa: totals.buyer_cash },
    { labelId: 'bikri-summary-buyer-total', paisa: totals.buyer_total, emphasis: 'total' },
  ];
}

function Rows({ lines, negativeNet }: { lines: readonly Line[]; negativeNet: boolean }) {
  return (
    <dl className="flex flex-col">
      {lines.map((line) => {
        const red = line.labelId === 'glossary-safi-bikri' && negativeNet;
        return (
          <div
            key={line.labelId}
            className={`flex items-center justify-between gap-3 border-b border-border py-1 ${line.emphasis === 'total' ? 'type-label' : 'type-body'} ${red ? 'text-due' : 'text-ink'}`}
          >
            <Localized id={line.labelId}>
              <dt />
            </Localized>
            <dd>
              <Num paisa={line.paisa} />
            </dd>
          </div>
        );
      })}
    </dl>
  );
}

export function SavedSummary({ detail }: { detail: BikriDetail }) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  // The whole بکری, or one lot (a بکری with several lots can pick which). It starts on the whole.
  const [whole, setWhole] = useState(true);
  const [lotIndex, setLotIndex] = useState(0);
  const lot = whole ? undefined : detail.lots[lotIndex];
  const totals = lot?.totals ?? detail.totals;
  // The test is on the text the backend's number is written as, so nothing is compared as a number.
  const hasRateDiff = formatPaisa(totals.rate_diff, { grouping }) !== '0';
  const netIsNegative = formatPaisa(totals.net, { grouping }).startsWith('-');
  return (
    <section
      aria-labelledby="bikri-saved-summary-heading"
      className="flex min-w-0 flex-col gap-3 rounded-3 border border-border bg-surface p-4"
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Localized id="bikri-summary-title">
          <h2 id="bikri-saved-summary-heading" className="type-section text-ink" />
        </Localized>
        <div className="flex flex-wrap items-center gap-2">
          {whole || detail.lots.length < 2 ? null : (
            <Field hideLabel label={<Localized id="bikri-saved-pick-lot" />}>
              {(control) => (
                <Select
                  {...control}
                  value={String(lotIndex)}
                  onChange={(next) => {
                    setLotIndex(Number(next));
                  }}
                  options={detail.lots.map((candidate, index) => ({
                    value: String(index),
                    label: (
                      <Localized id="bikri-summary-lot" vars={{ lot: String(candidate.lot_no) }} />
                    ),
                  }))}
                />
              )}
            </Field>
          )}
          <SegmentedControl
            groupLabel={l10n.getString('bikri-summary-title')}
            value={whole ? 'whole' : 'lot'}
            onChange={(next) => {
              setWhole(next === 'whole');
            }}
            options={[
              {
                value: 'lot',
                label: l10n.getString('bikri-summary-lot', {
                  lot: String(detail.lots[lotIndex]?.lot_no ?? 1),
                }),
              },
              { value: 'whole', label: l10n.getString('bikri-summary-whole') },
            ]}
          />
        </div>
      </div>
      <div className="grid grid-cols-1 gap-x-4 gap-y-3 normal:grid-cols-2">
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
          <dl>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
              <Localized id="bikri-split-shared">
                <dt />
              </Localized>
              <dd>
                <Num paisa={totals.shared_income} />
              </dd>
            </div>
          </dl>
        </div>
      </div>
    </section>
  );
}
