// What the account list remembers for the Owner (PRD FR-7.45): the kind of account and whether
// inactive ones show, kept as text under the screen name `list.accounts`. The words typed in the
// search box are not kept: a forgotten search that hides rows would puzzle the next morning. A value
// that cannot be read any more is left out.

import type { AccountType } from '../../shared/accounts';
import { choiceOr, flagOf, isRecord } from '../../shared/screen-state';

export const ACCOUNTS_SCREEN = 'list.accounts';

const TYPE_CHOICES: readonly (AccountType | 'all')[] = [
  'all',
  'seller',
  'buyer',
  'money_lent',
  'other',
];

export interface AccountChoices {
  type: AccountType | 'all';
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
