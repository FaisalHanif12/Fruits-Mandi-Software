// What the account list remembers for the Owner (PRD FR-7.45): the kind of account and whether
// inactive ones show, kept as text under the screen name `list.accounts`. The words typed in the
// search box are not kept: a forgotten search that hides rows would puzzle the next morning. A value
// that cannot be read any more is left out.

import type { AccountListKind } from '../../shared/accounts';
import { choiceOr, flagOf, isRecord } from '../../shared/screen-state';

export const ACCOUNTS_SCREEN = 'list.accounts';

// The list shows sellers, buyers and the پختہ کھاتہ accounts only (plan 32, R6). A kind kept from
// an earlier version that is not in this list ("money_lent", "other") cannot be read any more and
// the list starts at "all".
const TYPE_CHOICES: readonly AccountListKind[] = ['all', 'seller', 'buyer', 'pukhta'];

export interface AccountChoices {
  type: AccountListKind;
  showInactive: boolean;
}

export const NO_ACCOUNT_CHOICES: AccountChoices = { type: 'all', showInactive: false };

export function accountChoicesFrom(raw: unknown): AccountChoices {
  if (!isRecord(raw)) return NO_ACCOUNT_CHOICES;
  return {
    type: choiceOr(raw['type'], TYPE_CHOICES, 'all'),
    showInactive: flagOf(raw['showInactive']),
  };
}

export function accountChoicesText(choices: AccountChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.type !== 'all') kept['type'] = choices.type;
  if (choices.showInactive) kept['showInactive'] = true;
  return Object.keys(kept).length === 0 ? null : kept;
}
