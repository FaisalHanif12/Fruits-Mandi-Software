// Where a row of کل مالیت leads: that party's history (PRD FR-9.10). روکڑ opens the cash book; a buyer or a
// seller opens his history on پارٹی کھاتے (one row for each بکری, as the dues pages do, PRD FR-19.8); a
// person who borrowed and a الگ کھاتہ open their own screens with him chosen; every other account (a
// payable, a garden, an account of the dead amount, any other balance) opens its full ledger, R13. The
// addresses are plain data, so a test can say where a click goes. Nothing is saved by opening them.

import type { TotalWorthRow } from '../../bindings/TotalWorthRow';
import { lentSearchOf, pickerOfParty } from './dues-links';

export type WorthLink =
  | { to: '/rokar' }
  | { to: '/bikri/party-ledgers'; search: { side: 'buyer' | 'seller'; party: number } }
  | { to: '/rokar/lent'; search: ReturnType<typeof lentSearchOf> }
  | { to: '/rokar/khata'; search: { party: ReturnType<typeof pickerOfParty> } }
  | { to: '/reports/$id'; params: { id: 'R13' }; search: { party: number } };

export function worthLinkOf(row: TotalWorthRow): WorthLink {
  const party = row.party.account_id;
  switch (row.source) {
    case 'cash':
      return { to: '/rokar' };
    case 'buyers_dues':
    case 'buyers_credit':
      return { to: '/bikri/party-ledgers', search: { side: 'buyer', party } };
    case 'sellers_dues':
    case 'sellers_payable':
    case 'open_advances':
      return { to: '/bikri/party-ledgers', search: { side: 'seller', party } };
    case 'money_lent':
      return { to: '/rokar/lent', search: lentSearchOf(row.party) };
    case 'khata_minus':
    case 'khata_plus':
      return { to: '/rokar/khata', search: { party: pickerOfParty(row.party) } };
    case 'dead_amount':
    case 'garden_minus':
    case 'garden_plus':
    case 'payables':
    case 'other_have':
    case 'other_give':
      return { to: '/reports/$id', params: { id: 'R13' }, search: { party } };
  }
}
