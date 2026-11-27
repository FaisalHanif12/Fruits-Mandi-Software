// The two cards at the top of the روکڑ بک (PRD FR-16.14 v1.61, plan 100 G): کل سرمایہ (capital put in less taken
// back, opening capital included, up to the window's end) and آخر میں روکڑ (the cash at the window's end; for today it
// is the tile موجود روکڑ). The card شروع میں روکڑ was removed in v1.61 (Umar, 28-11-2026). Both are the backend's
// figures, signed (cash can be below zero, FR-6.7). Nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { Fact } from '../../../shared/ui';
import type { Cashbook } from '../types';
import { SignedNum } from './SignedNum';

export function CashCards({ book }: { book: Cashbook }) {
  return (
    <dl className="grid shrink-0 grid-cols-2 gap-3">
      <Fact stat slimWhenShort label={<Localized id="cashbook-card-capital" />}>
        <SignedNum inherit paisa={book.total_capital_paisa} />
      </Fact>
      <Fact stat slimWhenShort label={<Localized id="cashbook-closing" />}>
        <SignedNum inherit paisa={book.closing_paisa} />
      </Fact>
    </dl>
  );
}
