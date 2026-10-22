// What the purchase costs (PRD 12.10, FR-12.24), live under the form. Every line is the backend's
// number, shown with formatPaisa, and the calculated ones have their ⓘ with the backend's working
// (FR-7.43, F42): the value (نگ × rate, or weight × rate per kg), what is paid now, the total
// cost and the average cost for each نگ (PRD A45). Nothing is added up here (CLAUDE.md rule 1).
// Four states: nothing typed yet, the numbers, and a purchase that cannot be worked out yet,
// which says so in words.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { Working } from '../../../bindings/Working';
import type { Paisa } from '../../../shared/format';
import { CalcInfo, Card, Num } from '../../../shared/ui';
import type { PurchaseNumbers } from '../types';

interface Line {
  id: string;
  label: ReactNode;
  paisa: Paisa;
  working?: Working;
  emphasis?: boolean;
}

function linesOf(numbers: PurchaseNumbers): Line[] {
  const { workings } = numbers;
  return [
    {
      id: 'value',
      label: <Localized id="purchases-line-value" />,
      paisa: numbers.value,
      working: workings.value,
    },
    { id: 'freight', label: <Localized id="glossary-kiraya" />, paisa: numbers.freight },
    { id: 'labour', label: <Localized id="glossary-mazdoori" />, paisa: numbers.labour },
    {
      id: 'other',
      label: <Localized id="purchases-field-other-cost" />,
      paisa: numbers.other_cost,
    },
    {
      id: 'paid',
      label: <Localized id="purchases-line-paid-now" />,
      paisa: numbers.paid_now,
      working: workings.paid_now,
      emphasis: true,
    },
    {
      id: 'total',
      label: <Localized id="purchases-line-total-cost" />,
      paisa: numbers.total_cost,
      working: workings.total_cost,
    },
    {
      id: 'avg',
      label: <Localized id="purchases-line-avg-cost" />,
      paisa: numbers.avg_cost,
      working: workings.avg_cost,
    },
  ];
}

interface PurchaseNumbersCardProps {
  /** The backend's numbers for the form as it is, or null while it cannot work them out. */
  numbers: PurchaseNumbers | null;
  /** Nothing has been asked yet: the form is still empty. */
  empty: boolean;
}

export function PurchaseNumbersCard({ numbers, empty }: PurchaseNumbersCardProps) {
  return (
    <Card compact title={<Localized id="purchases-numbers-title" />}>
      {numbers === null ? (
        <Localized id={empty ? 'purchases-numbers-empty' : 'purchases-numbers-incomplete'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <dl className="flex flex-col">
          {linesOf(numbers).map((line) => (
            <div
              key={line.id}
              className={`flex items-center justify-between gap-3 border-b border-border py-1 text-ink ${line.emphasis === true ? 'type-label' : 'type-body'}`}
            >
              <dt>{line.label}</dt>
              <dd className="flex items-center gap-1">
                <Num paisa={line.paisa} />
                {line.working === undefined ? null : <CalcInfo working={line.working} inCell />}
              </dd>
            </div>
          ))}
        </dl>
      )}
    </Card>
  );
}
