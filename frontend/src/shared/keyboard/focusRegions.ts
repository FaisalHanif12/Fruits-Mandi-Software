// The big parts of the shell the keyboard moves between: the menus and the screen (PRD FR-7.17).
// The shell registers them; an arrow at the edge of a form, or in a menu, calls focusRegion.
// Several elements can share a region (the main menu and the side menu are both "menu"): the one
// registered last is the nearest to the screen, so it answers. Nothing here knows about a
// particular screen.

import { useCallback, type RefCallback } from 'react';

export type FocusRegion = 'menu' | 'screen';

const regions = new Map<FocusRegion, HTMLElement[]>();

const FOCUSABLE =
  'input:not([type="hidden"]):not([disabled]), select:not([disabled]), textarea:not([disabled]), button:not([disabled]), a[href], [role="combobox"], [tabindex]:not([tabindex="-1"])';

/** Registers the element of a region. Returns the function that removes it. */
export function registerFocusRegion(region: FocusRegion, element: HTMLElement): () => void {
  regions.set(region, [...(regions.get(region) ?? []), element]);
  return () => {
    regions.set(
      region,
      (regions.get(region) ?? []).filter((candidate) => candidate !== element),
    );
  };
}

/** The row of tabs of a section (PRD FR-5.3, v1.56) marks itself with this; it is not a field of the screen. */
const TABS_MARK = '[data-section-tabs]';

/** The first element of `selector` in `element` that is not in the tab row of a section. */
function firstOutsideTabs(element: HTMLElement, selector: string): HTMLElement | null {
  return (
    Array.from(element.querySelectorAll<HTMLElement>(selector)).find(
      (candidate) => candidate.closest(TABS_MARK) === null,
    ) ?? null
  );
}

/**
 * Moves focus into a region: its current item, else its first field. True when it moved. The tab row of a
 * section sits in the title row of the screen, but it is the way in to other screens, not a field of this one:
 * the arrow from a menu, and the arrow down from the tabs, land on the screen's own first field.
 */
export function focusRegion(region: FocusRegion): boolean {
  const element = regions.get(region)?.at(-1);
  if (element === undefined) return false;
  const target =
    firstOutsideTabs(element, '[aria-current="page"]') ?? firstOutsideTabs(element, FOCUSABLE);
  if (target === null) return false;
  target.focus();
  return document.activeElement === target;
}

/** A ref that registers its element as a focus region while it is on the page. */
export function useFocusRegion<T extends HTMLElement>(region: FocusRegion): RefCallback<T> {
  return useCallback(
    (element: T | null) => (element === null ? undefined : registerFocusRegion(region, element)),
    [region],
  );
}
