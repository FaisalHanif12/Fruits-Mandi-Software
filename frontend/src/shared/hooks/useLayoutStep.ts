import { useSyncExternalStore } from 'react';

/** The three layout steps by window width (Code Standards F22, design system 4). */
export type LayoutStep = 'compact' | 'normal' | 'wide';

/** The width where a step starts, read from the design tokens (--breakpoint-normal, -wide). */
function queryOf(token: string): string | null {
  const width = getComputedStyle(document.documentElement).getPropertyValue(token).trim();
  return width === '' ? null : `(min-width: ${width})`;
}

function lists(): MediaQueryList[] {
  if (typeof window.matchMedia !== 'function') return [];
  return ['--breakpoint-normal', '--breakpoint-wide']
    .map(queryOf)
    .flatMap((query) => (query === null ? [] : [window.matchMedia(query)]));
}

function subscribe(onChange: () => void): () => void {
  const watched = lists();
  for (const list of watched) list.addEventListener('change', onChange);
  return () => {
    for (const list of watched) list.removeEventListener('change', onChange);
  };
}

function snapshot(): LayoutStep {
  // Without media queries (a test with no layout) the middle step is the safe default.
  const [normal, wide] = lists();
  if (normal === undefined || wide === undefined) return 'normal';
  if (wide.matches) return 'wide';
  if (normal.matches) return 'normal';
  return 'compact';
}

/**
 * The layout step of the window now, kept up to date when it is resized. A list uses it to keep
 * the less used columns for the wider steps, so nothing ever scrolls sideways (NFR-USE-15, F22).
 */
export function useLayoutStep(): LayoutStep {
  return useSyncExternalStore(subscribe, snapshot, () => 'normal');
}

const ORDER: Record<LayoutStep, number> = { compact: 0, normal: 1, wide: 2 };

/** True when `step` is `from` or wider. */
export function isFromStep(step: LayoutStep, from: LayoutStep): boolean {
  return ORDER[step] >= ORDER[from];
}
