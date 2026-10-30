// What a sale in another shop does, live under the form (PRD FR-12.32, FR-12.27, C21, C25): the
// money received, the cost of the goods sold (C21), the result of the sale, how many نگ and how
// much cost are left in the purchase, and each trip partner's part of the money. A loss is red
// and also written (NFR-USE-12). Every number is the backend's, with its working behind the ⓘ.
// Nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { ArrowLeftRight, TriangleAlert } from 'lucide-react';
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
          {numbers.qty_from_our_shop > 0 ? (
            // The part set for another shop is not enough: the rest comes from our shop's part, in
            // this same save (PRD FR-12.41, D1). Told in words and an icon, before the save.
            <p role="status" className="flex items-start gap-2 type-label text-warn">
              <ArrowLeftRight aria-hidden="true" className="mt-1 size-icon-sm shrink-0" />
              <Localized
                id="split-moved-from-our"
                vars={{ count: String(numbers.qty_from_our_shop) }}
              >
                <span />
              </Localized>
            </p>
          ) : null}
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-amount">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={numbers.amount} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="purchases-detail-cost-sold">
                <dt />
              </Localized>
              <dd className="value-col">
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
              <dd className="value-col">
                <Num paisa={numbers.result_paisa} />
                <CalcInfo working={numbers.result_working} inCell />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-left-after">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num>{String(numbers.qty_left_after)}</Num>
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-other-left-after">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num>{String(numbers.qty_other_left_after)}</Num>
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="trips-sale-cost-left-after">
                <dt />
              </Localized>
              <dd className="value-col">
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
