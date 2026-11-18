// Where the user came from (plan 78 F1, PRD FR-7.18): a short trail of the pages he has opened one from
// another, kept in memory. The Back button and Esc go to the last stop of the trail, in the state the page
// was left in (its address, with its month, filters and choices). A way in from a menu starts the trail
// again, because a menu is not "another page". Nothing is saved: after a restart the trail is empty and
// each page falls back to its parent (back-parents.ts).
//
// The trail holds addresses only (path and search). The data of a screen is never kept here.

import { create } from 'zustand';

/** How many pages back the trail remembers. The oldest is dropped first. */
export const MAX_TRAIL = 20;

interface TrailState {
  /** The addresses of the pages left, oldest first, each as `/path?search`. */
  stops: readonly string[];
}

export const useTrailStore = create<TrailState>(() => ({ stops: [] }));

/** The path of an address, without its search or hash: `/partners/capital?x=1` is `/partners/capital`. */
export function pathOf(href: string): string {
  const cut = href.search(/[?#]/);
  return cut < 0 ? href : href.slice(0, cut);
}

/** Adds the page that was just left, unless it is the page the trail already ends with. */
export function pushStop(href: string): void {
  const { stops } = useTrailStore.getState();
  const last = stops.at(-1);
  if (last !== undefined && pathOf(last) === pathOf(href)) {
    // The same page again with another state: keep the latest state of it.
    useTrailStore.setState({ stops: [...stops.slice(0, -1), href] });
    return;
  }
  useTrailStore.setState({ stops: [...stops, href].slice(-MAX_TRAIL) });
}

/** Cuts the trail down to the stops before `index` (the stop at `index` is where the user goes). */
export function cutTrailTo(index: number): void {
  const { stops } = useTrailStore.getState();
  useTrailStore.setState({ stops: stops.slice(0, Math.max(0, index)) });
}

/** Empties the trail: the user came in from a menu. */
export function clearTrail(): void {
  useTrailStore.setState({ stops: [] });
}

/**
 * What the next change of page is: a link from a page (the default: the page left joins the trail), the
 * Back button (nothing joins; the trail was cut), or a fresh start from a menu (the trail is emptied).
 * A mode is promised for one target path. If the page that opens is another one (the way was blocked by a
 * "save changes?" box and the user went elsewhere), the promise is dropped.
 */
export type NavigationMode = 'link' | 'back' | 'fresh';

let promised: { mode: Exclude<NavigationMode, 'link'>; path: string } | null = null;

/** Say what the coming change of page to `path` is, right before it is asked for. */
export function promiseNavigation(mode: Exclude<NavigationMode, 'link'>, path: string): void {
  promised = { mode, path };
}

/** Used once by the recorder when the page has changed to `path`. */
export function takeNavigationMode(path: string): NavigationMode {
  const mine = promised;
  promised = null;
  return mine !== null && mine.path === path ? mine.mode : 'link';
}

/** For tests: forget the trail and any promise. */
export function resetTrail(): void {
  promised = null;
  useTrailStore.setState({ stops: [] });
}
