// What a trip cost does, live under the form (PRD FR-12.31, C24): the amount, and how it is
// spread over the purchases that still have goods left, by نگ bought. Each purchase's part is the
// backend's number with the working behind its ⓘ; its cost left goes up by that part. Nothing is
// added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, Num } from '../../../shared/ui';
import { purchaseNumberText } from '../../purchases';
import type { TripCostNumbers } from '../types';

interface CostNumbersCardProps {
  /** The backend's numbers for the form as it is, or null while it cannot work them out. */
  numbers: TripCostNumbers | null;
  empty: boolean;
}

export function CostNumbersCard({ numbers, empty }: CostNumbersCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  return (
    <Card compact title={<Localized id="trips-cost-numbers-title" />}>
      {numbers === null ? (
        <Localized id={empty ? 'trips-cost-numbers-empty' : 'trips-cost-numbers-incomplete'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <>
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
              <Localized id="trips-cost-amount">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={numbers.amount} />
              </dd>
            </div>
          </dl>
          <Localized id="trips-cost-spread-title">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <ul aria-label={l10n.getString('trips-cost-spread-title')} className="flex flex-col">
            {numbers.spread.map((part) => (
              <li
                key={part.purchase_id}
                className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 border-b border-border py-1 type-body text-ink"
              >
                <span className="flex flex-wrap items-baseline gap-x-2">
                  <span>{purchaseNumberText(part.purchase_number, say)}</span>
                  <span className="text-ink-muted" lang={language}>
                    {displayName(part.item, language)}
                  </span>
                  <span className="text-ink-muted">
                    <Localized id="trips-cost-spread-qty" vars={{ qty: String(part.qty) }}>
                      <span />
                    </Localized>
                  </span>
                </span>
                <span className="flex items-center gap-1">
                  <Num paisa={part.amount_paisa} />
                  <CalcInfo working={part.working} inCell />
                </span>
              </li>
            ))}
          </ul>
        </>
      )}
    </Card>
  );
}
