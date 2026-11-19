// The kinds an account is shown as (PRD 10.2, FR-10.2). The backend knows two sides, seller and
// buyer (`account_type`), and the account form lists the same two. A چکتا buyer (PRD Section 13) is
// not an account of his own, so he has no kind here. This is the one place that names them, so no
// screen works out the mapping on its own. Nothing here touches money.

import type { AccountType } from './types';

/** What the account form, the list, the picker row and the party card call an account. */
export type AccountKind = AccountType;

/** The kinds a new account can be, in the order the form lists them (PRD 10.2). */
export const NEW_ACCOUNT_KINDS: readonly AccountKind[] = ['seller', 'buyer'];

/** The message that names a kind in the language of the app (accounts.ftl). */
export function kindMessageId(kind: AccountKind): string {
  return `accounts-type-${kind}`;
}

/** The message that names an account from the side the backend sends about it. */
export function typeMessageId(type: AccountType): string {
  return kindMessageId(type);
}
