// What a sale in another shop does, live under the form (PRD FR-12.32, FR-12.27, C21, C25): the
// money received, the cost of the goods sold (C21), the result of the sale, how many نگ and how
// much cost are left in the purchase, and each trip partner's part of the money. A loss is red
// and also written (NFR-USE-12). Every number is the backend's, with its working behind the ⓘ.
// Nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { isNegativePaisa } from '../../../shared/format';
import { CalcInfo, Card, Num, ShareAmounts } from '../../../shared/ui';
import type { OtherSaleNumbers } from '../types';

interface OtherSaleNumbersCardProps {
  /** The backend's numbers for the form as it is, or null while it cannot work them out. */
  numbers: OtherSaleNumbers | null;
  empty: boolean;
}

export function OtherSaleNumbersCard({ numbers, empty }: OtherSaleNumbersCardProps) {
  const { l10n } = useLocalization();
  const loss = numbers !== null && isNegativePaisa(numbers.result_paisa);
  return (
    <Card compact title={<Localized id="trips-sale-numbers-title" />}>
      {numbers === null ? (
        <Localized id={empty ? 'trips-sale-numbers-empty' : 'trips-sale-numbers-incomplete'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <>
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-amount">
                <dt />
              </Localized>
              <dd>
                <Num paisa={numbers.amount} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="purchases-detail-cost-sold">
                <dt />
              </Localized>
              <dd className="flex items-center gap-1">
                <Num paisa={numbers.cost_paisa} />
                <CalcInfo working={numbers.cost_working} inCell />
              </dd>
            </div>
            <div
              className={`flex items-center justify-between gap-3 border-b border-border py-1 type-label ${loss ? 'text-due' : 'text-ink'}`}
            >
              <Localized id={loss ? 'glossary-tijarati-nuqsan' : 'glossary-tijarati-munafa'}>
                <dt />
              </Localized>
              <dd className="flex items-center gap-1">
                <Num paisa={numbers.result_paisa} />
                <CalcInfo working={numbers.result_working} inCell />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-left-after">
                <dt />
              </Localized>
              <dd>
                <Num>{String(numbers.qty_left_after)}</Num>
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-cost-left-after">
                <dt />
              </Localized>
              <dd>
                <Num paisa={numbers.cost_left_after_paisa} />
              </dd>
            </div>
          </dl>
          {loss ? (
            <p className="flex items-center gap-2 type-body text-due">
              <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              <Localized id="purchases-loss-note">
                <span />
              </Localized>
            </p>
          ) : null}
          <Localized id="trips-sale-credits-title">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <ShareAmounts
            credits={numbers.credits}
            label={l10n.getString('trips-sale-credits-title')}
          />
        </>
      )}
    </Card>
  );
}
