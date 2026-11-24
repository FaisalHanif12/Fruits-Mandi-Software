// The width of the window as the three layout steps see it (Code Standards F22): media queries that
// match up to a step. A test with no layout has the middle step; this sets another one. Call
// `vi.unstubAllGlobals()` after the test.

import { vi } from 'vitest';

export type Step = 'compact' | 'normal' | 'wide';

export function stubLayoutStep(step: Step): void {
  const matchesFor = (query: string) => {
    if (query.includes('1920')) return step === 'wide';
    if (query.includes('1366')) return step !== 'compact';
    return false;
  };
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: matchesFor(query),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));
  document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
  document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
}
