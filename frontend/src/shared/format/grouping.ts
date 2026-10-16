// The number grouping chosen in Settings (PRD Section 21, Language): 1,316,040 or 13,16,040.
// One value for the whole app, read by every amount and quantity on screen. The Settings screen
// (and the saved user preference) sets it in a later plan; until then it is international
// (PRD A20).

import { create } from 'zustand';
import type { Grouping } from './money';

export const DEFAULT_GROUPING: Grouping = 'international';

interface GroupingState {
  grouping: Grouping;
  setGrouping: (grouping: Grouping) => void;
}

export const useGroupingStore = create<GroupingState>()((set) => ({
  grouping: DEFAULT_GROUPING,
  setGrouping: (grouping) => {
    set({ grouping });
  },
}));

/** The grouping in use now. */
export function useGrouping(): Grouping {
  return useGroupingStore((state) => state.grouping);
}
