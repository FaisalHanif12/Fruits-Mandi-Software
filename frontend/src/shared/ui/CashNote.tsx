// The amber note under an entry that would take cash in hand below zero (PRD FR-6.7): the cash
// after the entry, from the backend (it uses the same function as the save). The note only tells;
// asking "is this entry correct?" is the confirm box on save (CashConfirmDialog). One piece for
// every entry screen: the sale book, the vouchers and the entries of plan 17.

import { Localized } from '@fluent/react';
import type { CashCheck } from '../../bindings/CashCheck';
import { formatPaisa, useGrouping } from '../format';
import { Notice } from './Notice';
import { Num } from './Num';

export function CashNote({ cash }: { cash: CashCheck }) {
  const grouping = useGrouping();
  if (!cash.needs_confirm) return null;
  return (
    <Notice tone="warn">
      <Localized id="common-cash-confirm-title">
        <span className="type-label" />
      </Localized>{' '}
      <Localized
        id="common-cash-after"
        vars={{ amount: formatPaisa(cash.cash_after_paisa, { grouping }) }}
        elems={{ num: <Num /> }}
      >
        <span />
      </Localized>
    </Notice>
  );
}
