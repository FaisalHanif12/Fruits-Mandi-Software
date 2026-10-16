// The amber note under the entry when it would take cash in hand below zero (PRD FR-6.7): the cash
// after the entry, from the backend (it uses the same function as the save). The note only
// tells; asking "is this entry correct?" is the confirm box on save (CashConfirmDialog, plan 13).

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { Num } from '../../../shared/ui';
import type { CashCheck } from '../../../bindings/CashCheck';

export function CashNote({ cash }: { cash: CashCheck }) {
  const grouping = useGrouping();
  if (!cash.needs_confirm) return null;
  return (
    <div
      role="status"
      className="flex flex-wrap items-center gap-2 rounded-2 border border-warn-border bg-warn-soft p-3 text-warn"
    >
      <TriangleAlert aria-hidden="true" className="size-icon-md shrink-0" />
      <Localized id="bikri-cash-note">
        <span className="type-label" />
      </Localized>
      <Localized
        id="bikri-cash-after"
        vars={{ amount: formatPaisa(cash.cash_after_paisa, { grouping }) }}
        elems={{ num: <Num /> }}
      >
        <span className="type-body" />
      </Localized>
    </div>
  );
}
