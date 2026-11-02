// Where the buttons of an answer lead (PRD FR-7.74, FR-7.79): a report or a view with the filters the
// model read it with, the dashboard, and the entry screen of a refused change request. The panel
// stays open when one is used, so the Owner can come back to the answer. A screen with unsaved work
// asks first, like any other way out of it (FR-7.9). Where no router exists (a component test, a
// gallery page) the functions are null and no button shows.

import { useRouter } from '@tanstack/react-router';
import { useMemo } from 'react';
import type { AnswerSource } from '../../bindings/AnswerSource';
import type { EntryRoute } from '../../bindings/EntryRoute';
import type { EntryScreen } from '../../bindings/EntryScreen';
import { useOpenReport } from '../reports';
import { analyticsSearchFromFilters, entryPartyOf, reportSearchFromFilters } from './addresses';

/**
 * The main screens a help answer points to (PRD FR-7.89) and the route each one opens. The screen opens
 * as it is, with no search and no party: a main screen never carries a party (plan 52). The
 * routes are the ones the backend names in `EntryScreen`.
 */
export const MAIN_SCREEN_PATHS = {
  dashboard: '/',
  settings: '/settings',
  masters: '/index',
  accounts: '/index/accounts',
  commission_rules: '/index/rules',
  today_sales: '/bikri/today',
  unsold_stock: '/bikri/stock',
  pukhta_khata: '/bikri/pukhta',
  collection_list: '/bikri/collection',
  cash_book: '/rokar',
  gardens: '/gardens',
  market_trips: '/trips',
  partners: '/partners',
  month_close: '/month',
  day_close: '/day',
  reports: '/reports',
  dead_amount: '/reports/dead-amount',
  analytics: '/analytics',
  backup: '/backup',
} as const;

/** A main screen: a screen of the app that is not an entry screen (no amount to write). */
export type MainScreen = keyof typeof MAIN_SCREEN_PATHS;

export function isMainScreen(screen: EntryScreen): screen is MainScreen {
  return screen in MAIN_SCREEN_PATHS;
}

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
        // The party goes into the screen's own address as a whole record; with none the screen
        // opens as it always did. Every amount stays empty and nothing is saved (FR-7.79).
        const screen = route.screen;
        if (isMainScreen(screen)) {
          // A help answer's screen opens as it is (FR-7.89); it never carries a party.
          void router.navigate({ to: MAIN_SCREEN_PATHS[screen] });
          return;
        }
        const party = route.party === null ? {} : { party: entryPartyOf(route.party) };
        switch (screen) {
          case 'bikri':
            void router.navigate({ to: '/bikri/book' });
            return;
          case 'wasooli':
            void router.navigate({ to: '/bikri/voucher', search: { type: 'receipt', ...party } });
            return;
          case 'adaigi':
            void router.navigate({ to: '/bikri/voucher', search: { type: 'payment', ...party } });
            return;
          case 'peshgi':
            void router.navigate({ to: '/bikri/voucher', search: { type: 'advance', ...party } });
            return;
          case 'kharcha':
            void router.navigate({ to: '/rokar/expense', search: party });
            return;
          case 'udhar_raqam':
            void router.navigate({ to: '/rokar/lent', search: party });
            return;
          case 'alag_khata':
            void router.navigate({ to: '/rokar/khata', search: party });
            return;
          case 'sarmaya':
            // The capital screen knows a partner by his id; it reads a whole `party` the same way.
            void router.navigate({
              to: '/partners/capital',
              search: route.party === null ? {} : { partner: route.party.id },
            });
            return;
          case 'phal':
            void router.navigate({ to: '/rokar/fruit', search: party });
            return;
          case 'general':
            void router.navigate({ to: '/rokar/general', search: party });
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
