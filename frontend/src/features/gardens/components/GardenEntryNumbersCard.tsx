// What the entry does, live under the form (PRD FR-12.35, FR-12.36, FR-12.37): the amount, the
// running balance before and after it with the working behind the ⓘ, and the rent still due
// after it. The parts of each partner are in a card of their own (EntryPartsCard). Every number
// is the backend's. Nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { ArrowLeftRight } from 'lucide-react';
import { CalcInfo, Card, Num } from '../../../shared/ui';
import type { GardenEntryNumbers } from '../types';

interface GardenEntryNumbersCardProps {
  /** The backend's numbers for the form as it is, or null while it cannot work them out. */
  numbers: GardenEntryNumbers | null;
  /** The form has nothing typed yet. */
  empty: boolean;
}

export function GardenEntryNumbersCard({ numbers, empty }: GardenEntryNumbersCardProps) {
  const { l10n } = useLocalization();
  return (
    <Card compact title={<Localized id="gardens-numbers-title" />}>
      {numbers === null ? (
        <Localized id={empty ? 'gardens-numbers-empty' : 'gardens-numbers-incomplete'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <>
          {numbers.qty_from_our_shop !== null && numbers.qty_from_our_shop > 0 ? (
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
          <dl className="flex flex-col" aria-label={l10n.getString('gardens-numbers-title')}>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
              <Localized id="gardens-numbers-amount">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={numbers.amount} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="gardens-numbers-balance-before">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={numbers.balance_before_paisa} />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="gardens-numbers-balance-after">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={numbers.balance_after_paisa} />
                <CalcInfo working={numbers.balance_working} inCell />
              </dd>
            </div>
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="gardens-numbers-rent-due-after">
                <dt />
              </Localized>
              <dd className="value-col">
                <Num paisa={numbers.rent_due_after_paisa} />
              </dd>
            </div>
            {numbers.qty_left_after === null || numbers.qty_other_left_after === null ? null : (
              <>
                <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
                  <Localized id="gardens-numbers-left-after">
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
              </>
            )}
          </dl>
        </>
      )}
    </Card>
  );
}
