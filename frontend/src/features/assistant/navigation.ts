// Where the buttons of an answer lead (PRD FR-7.74, FR-7.79): a report or a view with the filters the
// model read it with, the dashboard, and the entry screen of a refused change request. The panel
// stays open when one is used, so the Owner can come back to the answer. A screen with unsaved work
// asks first, like any other way out of it (FR-7.9). Where no router exists (a component test, a
// gallery page) the functions are null and no button shows.

import { useRouter } from '@tanstack/react-router';
import { useMemo } from 'react';
import type { AnswerSource } from '../../bindings/AnswerSource';
import type { EntryRoute } from '../../bindings/EntryRoute';
import { useOpenReport } from '../reports';
import { analyticsSearchFromFilters, reportSearchFromFilters } from './addresses';

export interface AssistantNavigation {
  /** Opens the report or view a source was read from, with its filters. */
  openSource: (source: AnswerSource) => void;
  /** Opens the entry screen of a refused change request: the party chosen, every amount empty. */
  openEntry: (route: EntryRoute) => void;
  /** Opens Settings on the questions tab. */
  openSettings: () => void;
}

export function useAssistantNavigation(): AssistantNavigation | null {
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  const openReport = useOpenReport();
  return useMemo<AssistantNavigation | null>(() => {
    if (router === undefined) return null;
    return {
      openSource: (source) => {
        switch (source.kind) {
          case 'report':
            openReport?.(source.id, reportSearchFromFilters(source.filters));
            return;
          case 'analytics':
            void router.navigate({
              to: '/analytics/$view',
              params: { view: source.view },
              search: analyticsSearchFromFilters(source.filters),
            });
            return;
          case 'tiles':
            void router.navigate({ to: '/' });
            return;
          case 'party_card':
            // A party card has no screen of its own; his ledger is the next source of the answer.
            return;
        }
      },
      openEntry: (route) => {
        switch (route.screen) {
          case 'bikri':
            void router.navigate({ to: '/bikri/book' });
            return;
          case 'wasooli':
            void router.navigate({ to: '/bikri/voucher', search: { type: 'receipt' } });
            return;
          case 'adaigi':
            void router.navigate({ to: '/bikri/voucher', search: { type: 'payment' } });
            return;
          case 'peshgi':
            void router.navigate({ to: '/bikri/voucher', search: { type: 'advance' } });
            return;
          case 'kharcha':
            void router.navigate({ to: '/rokar/expense' });
            return;
          case 'udhar_raqam':
            void router.navigate({ to: '/rokar/lent' });
            return;
          case 'alag_khata':
            void router.navigate({ to: '/rokar/khata' });
            return;
          case 'sarmaya':
            void router.navigate({ to: '/partners/capital' });
            return;
          case 'phal':
            void router.navigate({ to: '/rokar/fruit' });
            return;
          case 'general':
            void router.navigate({ to: '/rokar/general' });
            return;
          case 'account':
            if (route.party === null) void router.navigate({ to: '/index/accounts' });
            else
              void router.navigate({
                to: '/index/accounts/$id',
                params: { id: String(route.party.id) },
              });
            return;
        }
      },
      openSettings: () => {
        void router.navigate({ to: '/settings', search: { tab: 'assistant' } });
      },
    };
  }, [router, openReport]);
}
