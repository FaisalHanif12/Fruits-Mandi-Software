// "From روکڑ" under the opening balance of a party (PRD FR-10.9 v1.60, plan 99): the opening of a seller, a buyer,
// a person lent money or a الگ کھاتہ posts against the shop's cash, not the opening difference. بنام is money the
// shop gave him, so it goes out of روکڑ; جمع is money he gave, so it comes into روکڑ. The note only says which way;
// the cash after the save, and the question when it would go below zero (FR-6.7), come from the backend on save.

import { Localized } from '@fluent/react';
import { Info } from 'lucide-react';
import type { AccountType } from '../../../shared/accounts';

/** The control of the الگ کھاتے: their accounts are `other`, and their openings go through روکڑ too. */
const SEPARATE_KHATA_CONTROL = '0305';

/** Does this account's opening balance go through روکڑ (PRD FR-10.9)? */
export function opensThroughCash(accountType: AccountType, controlCode: string | null): boolean {
  return accountType !== 'other' || controlCode === SEPARATE_KHATA_CONTROL;
}

/** The message for what is typed: money out of cash, into it, or the rule when neither side is typed yet. */
function noteId(dr: string, cr: string): string {
  const debit = dr.trim() !== '';
  const credit = cr.trim() !== '';
  if (debit && !credit) return 'accounts-opening-cash-out';
  if (credit && !debit) return 'accounts-opening-cash-in';
  return 'accounts-opening-cash';
}

/** A plain hint under the fields: it explains, and is not a message that is read out as it changes. */
export function OpeningCashNote({ dr, cr }: { dr: string; cr: string }) {
  return (
    <p className="mt-3 flex items-start gap-2 rounded-2 bg-surface-sunken p-3 type-body text-ink">
      <Info aria-hidden="true" className="size-icon-md shrink-0 text-ink-muted" />
      <Localized id={noteId(dr, cr)}>
        <span className="min-w-0" />
      </Localized>
    </p>
  );
}
