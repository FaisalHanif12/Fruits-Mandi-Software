// The filters of the analytics area, kept above the screens (Code Standards F39): they stay while
// the user moves from one view to another, and when the language changes (which builds the grid
// and the chart again). A small store, like the other screens' stores (Tech Stack 3).

import { create } from 'zustand';
import { EMPTY_ANALYTICS_FILTERS, type AnalyticsFilterState } from './filters';

interface AnalyticsStore {
  filters: AnalyticsFilterState;
  /** True once the filters the Owner last used have been put back (PRD FR-7.45): done once per run. */
  hydrated: boolean;
  /** The first period column shown in views D and E, when they have more than fit (see columns.ts). */
  windowStart: number;
  /** The user asked for the columns that do not fit the window by default (see columns.ts). */
  otherColumns: boolean;
  /** The saved view that was opened or saved last, so it can be deleted or saved again. */
  savedName: string | null;
  change: (part: Partial<AnalyticsFilterState>) => void;
  replace: (filters: AnalyticsFilterState, savedName?: string | null) => void;
  /** Puts back the filters kept from last time (null: none kept), once. */
  hydrate: (filters: AnalyticsFilterState | null) => void;
  setWindowStart: (start: number) => void;
  setOtherColumns: (other: boolean) => void;
  setSavedName: (name: string | null) => void;
  reset: () => void;
}

export const useAnalyticsStore = create<AnalyticsStore>((set) => ({
  filters: EMPTY_ANALYTICS_FILTERS,
  hydrated: false,
  windowStart: 0,
  otherColumns: false,
  savedName: null,
  change: (part) => {
    // Another filter means another set of columns: start from the first one again.
    set((now) => ({ filters: { ...now.filters, ...part }, windowStart: 0 }));
  },
  replace: (filters, savedName = null) => {
    set({ filters, windowStart: 0, savedName });
  },
  hydrate: (filters) => {
    set((now) =>
      now.hydrated ? now : { hydrated: true, ...(filters === null ? {} : { filters }) },
    );
  },
  setWindowStart: (start) => {
    set({ windowStart: start });
  },
  setOtherColumns: (otherColumns) => {
    set({ otherColumns });
  },
  setSavedName: (savedName) => {
    set({ savedName });
  },
  reset: () => {
    set({
      filters: EMPTY_ANALYTICS_FILTERS,
      hydrated: false,
      windowStart: 0,
      otherColumns: false,
      savedName: null,
    });
  },
}));
