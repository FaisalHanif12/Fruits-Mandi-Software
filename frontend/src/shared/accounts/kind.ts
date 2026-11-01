// The kinds an account is shown as (plan 47, PRD FR-13.11, FR-10.2). The backend knows two things:
// the side (`account_type`, seller or buyer) and the mark `is_pukhta_party`. The user sees four
// names on the account form: بیوپاری, گاہک, پختہ کھاتہ گاہک and پختہ کھاتہ بیوپاری. This file is the
// one place that turns one into the other, so no screen works out the mapping on its own.
// Nothing here touches money.

import type { AccountType } from './types';

/** What the account form, the list, the picker row and the party card call an account. */
export type AccountKind = AccountType | 'pukhta_buyer' | 'pukhta_seller';

/** The kinds a new account can be, in the order the form lists them (PRD 10.2). */
export const NEW_ACCOUNT_KINDS: readonly AccountKind[] = [
  'seller',
  'buyer',
  'pukhta_buyer',
  'pukhta_seller',
];

/**
 * The kind of an account from what the backend sends. The mark is missing when false, and it only
 * means something on a seller or a buyer.
 */
export function kindOfAccount(type: AccountType, isPukhtaParty: boolean | undefined): AccountKind {
  if (isPukhtaParty !== true) return type;
  if (type === 'buyer') return 'pukhta_buyer';
  if (type === 'seller') return 'pukhta_seller';
  return type;
}

/** The side the backend knows for a kind: a پختہ گاہک is a buyer, a پختہ بیوپاری a seller. */
export function sideOfKind(kind: AccountKind): AccountType {
  if (kind === 'pukhta_buyer') return 'buyer';
  if (kind === 'pukhta_seller') return 'seller';
  return kind;
}

/** True for the two kinds of a پختہ party's own account. */
export function isPukhtaKind(kind: AccountKind): boolean {
  return kind === 'pukhta_buyer' || kind === 'pukhta_seller';
}

/** The message that names a kind in the language of the app (accounts.ftl). */
export function kindMessageId(kind: AccountKind): string {
  return `accounts-type-${kind}`;
}

/** The message that names an account from the two things the backend sends about it. */
export function typeMessageId(type: AccountType, isPukhtaParty: boolean | undefined): string {
  return kindMessageId(kindOfAccount(type, isPukhtaParty));
}

/**
 * Does a new account of this kind start with credit allowed? A پختہ buyer starts without it, so his
 * lines start as نقدی (PRD FR-13.11, A79, cash first); every other kind starts with it ticked, as
 * before (PRD 10.2). The Owner can change the tick.
 */
export function creditStartsAllowed(kind: AccountKind): boolean {
  return kind !== 'pukhta_buyer';
}
