// Where each line of the capital card opens (PRD FR-16.14, plan 87: "every line opens its page"). The backend names
// the page of each line (`CapitalLink`) and the record where the page needs one; this maps that to a route. A
// target is plain data, so a test can say where a click goes; `useOpenCapitalTarget` is the one place that turns
// it into a navigation (Code Standards F27).

import { useRouter } from '@tanstack/react-router';
import type { RecordRef } from '../../bindings/RecordRef';
import { useOpenDestination, type Destination } from '../dashboard';
import type { CapitalLink, CapitalSide } from './capital-types';

/** A page a line leads to: a dashboard destination, or one of the pages the dashboard has no name for. */
export type CapitalTarget =
  | Destination
  /** The cash rows below the card, on this very page. */
  | { kind: 'cash-here' }
  /** The الگ کھاتے, where a khata is chosen and its statement opens. */
  | { kind: 'khata' }
  /** The buyers' ledgers (پارٹی کھاتے). */
  | { kind: 'buyer-ledgers' }
  /** The partners' overview, where the linked money is shown in each total. */
  | { kind: 'partners' }
  /** One partner's statement (R31). */
  | { kind: 'partner-statement'; partner: number };

/** What a line says about the record its page needs. */
export interface CapitalLinkContext {
  /** The block of the line: the seller dues page opens on the side the line is about. */
  side?: CapitalSide;
  account?: RecordRef | null;
  partner?: RecordRef | null;
}

/** The page behind a line, or null when the line names a record it does not have (nothing to open). */
export function targetOfLink(
  link: CapitalLink,
  context: CapitalLinkContext = {},
): CapitalTarget | null {
  switch (link) {
    case 'cashbook':
      return { kind: 'cash-here' };
    case 'buyer_dues':
      return { kind: 'buyer-dues' };
    case 'buyer_ledgers':
      return { kind: 'buyer-ledgers' };
    // What a seller owes the shop is the `have` block; what the shop owes sellers is the `owe` block.
    case 'seller_dues':
      return { kind: 'seller-dues', side: context.side === 'owe' ? 'shop_owes' : 'seller_owes' };
    case 'dead_amount':
      return { kind: 'dead-amount' };
    case 'advances':
      return { kind: 'advances-open' };
    case 'money_lent':
      return { kind: 'lent-open' };
    case 'separate_khatas':
      return { kind: 'khata' };
    case 'gardens':
      return { kind: 'gardens' };
    case 'account_ledger':
      return context.account === null || context.account === undefined
        ? null
        : { kind: 'report', id: 'R13', party: context.account.id };
    case 'partner_statement':
      return context.partner === null || context.partner === undefined
        ? null
        : { kind: 'partner-statement', partner: context.partner.id };
    case 'partners':
      return { kind: 'partners' };
    case 'trial_balance':
      return { kind: 'report', id: 'R18' };
  }
}

/**
 * Opens a target. Where no router is around (a component test of a piece) nothing opens and nothing fails, the way
 * the dashboard's opener behaves. `showCash` is what the page does for the cash line: it is already on that page.
 */
export function useOpenCapitalTarget(showCash: () => void): (target: CapitalTarget) => void {
  const openDestination = useOpenDestination();
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  return (target) => {
    switch (target.kind) {
      case 'cash-here':
        showCash();
        return;
      case 'khata':
        void router?.navigate({ to: '/rokar/khata' });
        return;
      case 'buyer-ledgers':
        void router?.navigate({ to: '/bikri/party-ledgers', search: { side: 'buyer' } });
        return;
      case 'partners':
        void router?.navigate({ to: '/partners' });
        return;
      case 'partner-statement':
        void router?.navigate({ to: '/partners/statement', search: { partner: target.partner } });
        return;
      default:
        openDestination(target);
    }
  };
}
