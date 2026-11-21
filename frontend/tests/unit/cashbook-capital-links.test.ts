// Plan 87, PRD FR-16.14: every line of the capital card opens its page. The backend names the page of each line
// (`CapitalLink`) and the record where the page needs one; `targetOfLink` maps that to a route. Every kind has a
// target, a line that names a record it does not have opens nothing, and the seller dues page opens on the side
// the line is about (what sellers owe the shop is the `have` block, what the shop owes them the `owe` block).

import { describe, expect, it } from 'vitest';
import { targetOfLink } from '../../src/features/cashbook/capital-links';
import type { CapitalLink } from '../../src/features/cashbook/capital-types';
import { ADNAN, OPENING_CAPITAL_ACCOUNT, PARTNER_B } from '../helpers/capital-card';

describe('the page of each line of the capital card', () => {
  it.each([
    ['cashbook', {}, { kind: 'cash-here' }],
    ['buyer_dues', {}, { kind: 'buyer-dues' }],
    ['buyer_ledgers', {}, { kind: 'buyer-ledgers' }],
    ['dead_amount', {}, { kind: 'dead-amount' }],
    ['advances', {}, { kind: 'advances-open' }],
    ['money_lent', {}, { kind: 'lent-open' }],
    ['separate_khatas', {}, { kind: 'khata' }],
    ['gardens', {}, { kind: 'gardens' }],
    ['partners', {}, { kind: 'partners' }],
    ['trial_balance', {}, { kind: 'report', id: 'R18' }],
    ['seller_dues', { side: 'owe' }, { kind: 'seller-dues', side: 'shop_owes' }],
    ['seller_dues', { side: 'have' }, { kind: 'seller-dues', side: 'seller_owes' }],
    [
      'account_ledger',
      { account: OPENING_CAPITAL_ACCOUNT },
      { kind: 'report', id: 'R13', party: OPENING_CAPITAL_ACCOUNT.id },
    ],
    [
      'partner_statement',
      { partner: PARTNER_B },
      { kind: 'partner-statement', partner: PARTNER_B.id },
    ],
  ] as const satisfies readonly (readonly [CapitalLink, object, object])[])(
    '%s opens %j',
    (link, context, target) => {
      expect(targetOfLink(link, context)).toEqual(target);
    },
  );

  it('a line that names a record it does not have opens nothing', () => {
    expect(targetOfLink('account_ledger', { account: null })).toBeNull();
    expect(targetOfLink('account_ledger')).toBeNull();
    expect(targetOfLink('partner_statement', { partner: null })).toBeNull();
    expect(targetOfLink('partner_statement', { account: ADNAN })).toBeNull();
  });
});
