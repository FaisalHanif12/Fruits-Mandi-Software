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

/** Moves focus into a region: its current item, else its first field. True when it moved. */
export function focusRegion(region: FocusRegion): boolean {
  const element = regions.get(region)?.at(-1);
  if (element === undefined) return false;
  const target =
    element.querySelector<HTMLElement>('[aria-current="page"]') ??
    element.querySelector<HTMLElement>(FOCUSABLE);
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
