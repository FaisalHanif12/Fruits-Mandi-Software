// What the entry does, live under the form (PRD FR-12.35, FR-12.36, FR-12.37): the amount, the
// running balance before and after it with the working behind the ⓘ, and the rent still due
// after it. The parts of each partner are in a card of their own (EntryPartsCard). Every number
// is the backend's. Nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
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
        <dl className="flex flex-col" aria-label={l10n.getString('gardens-numbers-title')}>
          <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
            <Localized id="gardens-numbers-amount">
              <dt />
            </Localized>
            <dd>
              <Num paisa={numbers.amount} />
            </dd>
          </div>
          <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
            <Localized id="gardens-numbers-balance-before">
              <dt />
            </Localized>
            <dd>
              <Num paisa={numbers.balance_before_paisa} />
            </dd>
          </div>
          <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
            <Localized id="gardens-numbers-balance-after">
              <dt />
            </Localized>
            <dd className="flex items-center gap-1">
              <Num paisa={numbers.balance_after_paisa} />
              <CalcInfo working={numbers.balance_working} inCell />
            </dd>
          </div>
          <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
            <Localized id="gardens-numbers-rent-due-after">
              <dt />
            </Localized>
            <dd>
              <Num paisa={numbers.rent_due_after_paisa} />
            </dd>
          </div>
        </dl>
      )}
    </Card>
  );
}
