// Where a tile, a line of the attention list or a name on the dashboard leads (PRD FR-9.1,
// FR-7.44). A destination is plain data, so a test can say where a click goes; `useOpenDestination`
// is the one place that turns it into a navigation (Code Standards F27: no screen writes its own).

import { useRouter } from '@tanstack/react-router';
import type { AttentionFirst } from '../../bindings/AttentionFirst';
import type { AttentionKind } from '../../bindings/AttentionKind';
import type { ReportId } from '../../bindings/ReportId';
import { useOpenArrival } from '../gardens';
import type { ReportPerson } from '../reports';

export type Destination =
  | { kind: 'day' }
  | { kind: 'bikri-book'; id?: number }
  | { kind: 'bikri-today' }
  | { kind: 'stock' }
  | { kind: 'cashbook' }
  | { kind: 'trips' }
  | { kind: 'gardens' }
  | { kind: 'garden'; id: number }
  /** The garden screen of a goods arrived entry, found by the entry's id (PRD FR-12.40). */
  | { kind: 'arrival'; id: number }
  | { kind: 'backup' }
  /** The dead amount (R38, PRD FR-10.10): everyone moved to ڈوبی رقم. */
  | { kind: 'dead-amount' }
  | { kind: 'index' }
  | { kind: 'analytics' }
  | { kind: 'personal-drawings' }
  | { kind: 'money-lent' }
  | { kind: 'shop-expenses' }
  | { kind: 'settings-day' }
  | { kind: 'settings-app' }
  | { kind: 'receipt' }
  | { kind: 'payment' }
  | { kind: 'advance' }
  | { kind: 'report'; id: ReportId; party?: number; partner?: number; person?: ReportPerson };

/** The screen behind each kind of the attention list (PRD FR-7.44). */
export function destinationOfKind(kind: AttentionKind): Destination {
  switch (kind) {
    case 'drafts':
      return { kind: 'bikri-book' };
    case 'open_days':
      return { kind: 'day' };
    case 'stock_waiting':
      return { kind: 'stock' };
    case 'over_credit_limit':
    case 'dues_old':
      return { kind: 'report', id: 'R20' };
    case 'payables_old':
      return { kind: 'report', id: 'R24' };
    case 'backup_old':
      return { kind: 'backup' };
    case 'loans_overdue':
      return { kind: 'report', id: 'R32' };
    case 'takes_overdue':
      return { kind: 'personal-drawings' };
    case 'names_missing':
      return { kind: 'index' };
  }
}

/**
 * The ledger of an account (R13). A one-time person inside a پختہ کھاتہ account has his own lines,
 * so his key goes with it; the names are only for the person box to show (PRD FR-13.5).
 */
export function ledgerOf(
  accountId: number,
  personKey: string | null,
  names: { name_ur: string | null; name_en: string | null },
): Destination {
  if (personKey === null) return { kind: 'report', id: 'R13', party: accountId };
  return {
    kind: 'report',
    id: 'R13',
    party: accountId,
    person: {
      accountId,
      personKey,
      name_ur: names.name_ur ?? '',
      name_en: names.name_en ?? '',
      phone: '',
      last_date: '',
    },
  };
}

/**
 * The record the most urgent line of an item leads to: an account opens its ledger, a partner his
 * statement, a بکری itself, a trip purchase the trips, garden goods the list of goods left. The backend names the record; the screen only
 * opens it.
 */
export function destinationOfFirst(first: AttentionFirst): Destination {
  switch (first.target) {
    case 'account':
      return ledgerOf(first.id, first.person_key, first);
    case 'partner':
      return { kind: 'report', id: 'R31', partner: first.id };
    case 'bikri':
      return { kind: 'bikri-book', id: first.id };
    case 'purchase':
      return { kind: 'trips' };
    // Garden goods waiting: the entry opens its garden (PRD FR-12.40).
    case 'arrival':
      return { kind: 'arrival', id: first.id };
  }
}

/**
 * Opens a destination. Where no router is around (a component test, a gallery page) nothing opens
 * and nothing fails, the way the party card's report buttons behave.
 */
export function useOpenDestination(): (destination: Destination) => void {
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  const navigate: ReturnType<typeof useRouter>['navigate'] = (options) =>
    router === undefined ? Promise.resolve() : router.navigate(options);
  const openArrival = useOpenArrival();
  return (destination) => {
    switch (destination.kind) {
      case 'day':
        void navigate({ to: '/day' });
        return;
      case 'bikri-book':
        if (destination.id === undefined) void navigate({ to: '/bikri/book' });
        else void navigate({ to: '/bikri/book', search: { id: destination.id } });
        return;
      case 'bikri-today':
        void navigate({ to: '/bikri/today' });
        return;
      case 'stock':
        void navigate({ to: '/bikri/stock' });
        return;
      case 'cashbook':
        void navigate({ to: '/rokar' });
        return;
      case 'trips':
        void navigate({ to: '/trips' });
        return;
      case 'gardens':
        void navigate({ to: '/gardens' });
        return;
      case 'garden':
        void navigate({ to: '/gardens/$id', params: { id: String(destination.id) } });
        return;
      case 'arrival':
        openArrival(destination.id);
        return;
      case 'backup':
        void navigate({ to: '/backup' });
        return;
      case 'dead-amount':
        void navigate({ to: '/reports/dead-amount' });
        return;
      case 'index':
        void navigate({ to: '/index' });
        return;
      case 'analytics':
        void navigate({ to: '/analytics' });
        return;
      case 'personal-drawings':
        void navigate({ to: '/rokar/expense', search: { type: 'personal_expense' } });
        return;
      case 'money-lent':
        void navigate({ to: '/rokar/lent' });
        return;
      case 'shop-expenses':
        void navigate({ to: '/rokar/expense', search: { type: 'shop_expense' } });
        return;
      case 'settings-day':
        void navigate({ to: '/settings', search: { tab: 'day' } });
        return;
      case 'settings-app':
        void navigate({ to: '/settings', search: { tab: 'app' } });
        return;
      case 'receipt':
        void navigate({ to: '/bikri/voucher', search: { type: 'receipt' } });
        return;
      case 'payment':
        void navigate({ to: '/bikri/voucher', search: { type: 'payment' } });
        return;
      case 'advance':
        void navigate({ to: '/bikri/voucher', search: { type: 'advance' } });
        return;
      case 'report':
        void navigate({
          to: '/reports/$id',
          params: { id: destination.id },
          search: {
            ...(destination.party === undefined ? {} : { party: destination.party }),
            ...(destination.partner === undefined ? {} : { partner: destination.partner }),
            ...(destination.person === undefined ? {} : { person: destination.person }),
          },
        });
        return;
    }
  };
}
