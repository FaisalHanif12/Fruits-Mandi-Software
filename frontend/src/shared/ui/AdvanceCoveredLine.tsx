// The one line under a seller's open advance when his own credit already covers some of it (PRD FR-9.6
// v1.61, plan 100 A "Covered advance"): "پیشگی 100,000: بقایا میں سے کٹ چکی". The part covered is not shown
// as پیشگی باقی anywhere; this line says where it went. The amount is the backend's, written as it came;
// nothing is worked out here (CLAUDE.md rule 1). Used by the party card, the پیشگی card, the payables panel
// of ادائیگی and the advance tick of a بکری.

import { Localized } from '@fluent/react';
import { formatPaisa, useGrouping, type Paisa } from '../format';
import { RUNNING_NUMBER } from './Num';

export function AdvanceCoveredLine({ amount }: { amount: Paisa }) {
  const grouping = useGrouping();
  return (
    <Localized
      id="assist-advance-covered"
      vars={{ amount: formatPaisa(amount, { grouping }) }}
      elems={RUNNING_NUMBER}
    >
      <p className="type-body text-ink-muted" data-advance-covered />
    </Localized>
  );
}
