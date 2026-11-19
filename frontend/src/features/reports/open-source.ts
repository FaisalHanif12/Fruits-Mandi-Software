// Opening the record behind a row of a list (plan 44, PRD FR-19.4, FR-7.70): a row of a report or of
// the cash book carries what it opens (`source`), and one place turns it into a screen, so no list
// writes its own addresses. A بکری opens in the sale book, a voucher on the screen of its kind (the
// voucher screen sends it on), fruit taken on its screen, a trip or a garden on its own page, a trip
// purchase on its trip, and an account on its ledger. A record that has no screen of its own opens
// nothing.

import { useNavigate } from '@tanstack/react-router';
import { useCallback } from 'react';
import type { ReportRowSource } from '../../bindings/ReportRowSource';
import { purchasesGet } from '../purchases';

/** The function that opens a source, or does nothing for one that has no screen. */
export function useOpenRowSource(): (source: ReportRowSource | null) => void {
  const navigate = useNavigate();
  return useCallback(
    (source) => {
      if (source === null) return;
      switch (source.kind) {
        case 'bikri':
          void navigate({ to: '/bikri/book', search: { id: source.id } });
          return;
        case 'voucher':
          // The voucher screen moves a voucher of another kind to its own screen.
          void navigate({ to: '/bikri/voucher', search: { id: source.id } });
          return;
        case 'fruit':
          void navigate({ to: '/rokar/fruit', search: { entry: source.id } });
          return;
        case 'trip':
          void navigate({ to: '/trips/$id', params: { id: String(source.id) } });
          return;
        case 'garden':
          void navigate({ to: '/gardens/$id', params: { id: String(source.id) } });
          return;
        case 'purchase':
          // A purchase lives on its trip: the trip page opens with the purchase chosen.
          void purchasesGet(source.id).then((row) =>
            navigate({
              to: '/trips/$id',
              params: { id: String(row.trip.id) },
              search: { purchase: row.id },
            }),
          );
          return;
        case 'account':
          void navigate({
            to: '/reports/$id',
            params: { id: 'R13' },
            search: { party: source.id },
          });
          return;
        case 'arrival':
        case 'venture':
        case 'month_close':
        case 'entry':
          return;
      }
    },
    [navigate],
  );
}
