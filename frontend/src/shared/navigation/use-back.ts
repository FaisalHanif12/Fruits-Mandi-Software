// The Back action and its button's state (plan 78 F1, PRD FR-7.18). One hook for the Back button of the
// top bar and for Esc, so the two always do the same: go to the page the user came from, in the state he
// left it, else to the page's parent, else nothing (Esc then goes to the dashboard, as before).

import { useNavigate, useRouter, useRouterState } from '@tanstack/react-router';
import { useCallback } from 'react';
import { parentOf } from './back-parents';
import { cutTrailTo, pathOf, promiseNavigation, useTrailStore } from './page-trail';

/** Where Back goes now: the page the user came from, else the parent of this page, else nowhere. */
export function backTarget(
  path: string,
  stops: readonly string[],
): { kind: 'trail'; index: number; href: string } | { kind: 'parent'; to: string } | null {
  const index = stops.findLastIndex((stop) => pathOf(stop) !== path);
  const stop = index < 0 ? undefined : stops[index];
  if (stop !== undefined) return { kind: 'trail', index, href: stop };
  const parent = parentOf(path);
  return parent === null ? null : { kind: 'parent', to: parent };
}

export interface BackState {
  /** There is a page to go back to: the button shows. */
  available: boolean;
  /** Goes back. True when it went somewhere, false when there is nowhere to go. */
  go: () => boolean;
}

export function useBack(): BackState {
  const router = useRouter();
  const navigate = useNavigate();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const stops = useTrailStore((state) => state.stops);
  const target = backTarget(pathname, stops);
  const go = useCallback(() => {
    const now = backTarget(router.state.location.pathname, useTrailStore.getState().stops);
    if (now === null) return false;
    if (now.kind === 'trail') {
      cutTrailTo(now.index);
      promiseNavigation('back', pathOf(now.href));
      router.history.push(now.href);
    } else {
      promiseNavigation('back', now.to);
      void navigate({ to: now.to });
    }
    return true;
  }, [router, navigate]);
  return { available: target !== null, go };
}
