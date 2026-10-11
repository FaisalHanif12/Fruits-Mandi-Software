// A balance with its side as a word, never by colour alone (NFR-USE-12, design system 2): "2,880
// بنام" or "2,880 Debit". The amount comes from the backend and goes through Num; the side is the
// backend's too. The screen never works out a side or a sum (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import type { Paisa } from '../format';
import { Num } from './Num';

/** How the backend sends a balance: how much (never below zero) and on which side. */
export interface BalanceValue {
  amount_paisa: Paisa;
  side: 'dr' | 'cr' | 'none';
}

/** The Fluent message of a side. `none` has no word: the balance is zero. */
export function sideMessageId(side: BalanceValue['side']): string | null {
  if (side === 'dr') return 'common-side-dr';
  if (side === 'cr') return 'common-side-cr';
  return null;
}

export function Balance({ balance }: { balance: BalanceValue }) {
  const id = sideMessageId(balance.side);
  return (
    <span className="inline-flex items-baseline gap-2 whitespace-nowrap">
      <Num paisa={balance.amount_paisa} />
      {id === null ? null : <Localized id={id} />}
    </span>
  );
}
