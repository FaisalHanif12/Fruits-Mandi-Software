// Mounted once by the signed-in shell: records the page the user leaves when he opens another one
// (plan 78 F1). See page-trail.ts for what a "link", a "back" and a "fresh" change of page are.

import { useRouterState } from '@tanstack/react-router';
import { useEffect, useRef } from 'react';
import { clearTrail, pushStop, takeNavigationMode } from './page-trail';

export function usePageTrail(): void {
  // The trail is the signed-in shell's: when the shell goes (a log out, a lock that shows the sign-in, a test
  // that ends), the trail goes with it, so the next sign-in never starts with pages of the last one.
  useEffect(() => clearTrail, []);
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const href = useRouterState({ select: (state) => state.location.href });
  const before = useRef<{ pathname: string; href: string } | null>(null);
  useEffect(() => {
    const left = before.current;
    before.current = { pathname, href };
    // The first page, and a change of the page's own state (its month, its filters), is not a new page.
    if (left === null || left.pathname === pathname) return;
    const mode = takeNavigationMode(pathname);
    if (mode === 'back') return;
    if (mode === 'fresh') clearTrail();
    else pushStop(left.href);
  }, [pathname, href]);
}
