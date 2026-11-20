// Where an open dropdown list goes (plan 86, Umar 20-11-2026: "when the dropdown is open, if there is
// less space under, it must open upward so all the options are visible"; NFR-USE-16: nothing overlaps or
// gets cut off). One rule for every list that drops down from a field: the pickers, the suggestion
// lists, the grid cell lists, the select box, the menus (Code Standards F7).
//
// - Below the field when the whole list fits there. When it does not, it flips above, but only if
//   there is more room above than below. Either way its height is cut to the room on that side and
//   the list scrolls inside itself, so every option can be reached.
// - Room is the window less the parts of the shell that stay on screen (the status bar, the top bar,
//   the assistant button: elements marked `data-dropdown-avoid`) and a small margin at every edge.
// - Sideways the list starts at the start edge of its field (the right edge in Urdu) and is pushed back
//   inside the window when it would leave it, in both directions.
//
// Pure functions only: the sizes come in, a placement goes out (tests give sizes, no browser is
// needed). `DropdownPopup` reads the real sizes and applies the result.

import { tokenPx } from '../hooks/tokenPx';

/** The gap between a field and its open list, in CSS pixels (the old `mt-1`). */
export const DROPDOWN_GAP_PX = 4;
/** Room kept between a list and the edge of the window or a bar of the shell (design token `--space-2`). */
export const DROPDOWN_EDGE_PX = 8;
/** Marks a part of the shell that stays on screen: a list must not go under it. */
export const DROPDOWN_AVOID_ATTRIBUTE = 'data-dropdown-avoid';

/**
 * A box in window coordinates. Across the window it is measured from the left edge (`minX` is the
 * smaller number, `maxX` the larger), whatever the direction of writing: the browser measures that way,
 * and the direction is applied once, when the placement is written (`inlineStart`).
 */
export interface Box {
  top: number;
  bottom: number;
  minX: number;
  maxX: number;
}

export type DropdownSide = 'bottom' | 'top';

export interface PlaceArgs {
  /** The field the list hangs from. */
  anchor: Box;
  /** The size the list wants when nothing limits it. */
  wanted: { width: number; height: number };
  /** The size of the window (the part a page is drawn in). */
  viewport: { width: number; height: number };
  /** Parts of the shell that stay on screen (bars, the assistant button). */
  obstacles?: readonly Box[];
  /** Right to left (Urdu). */
  rtl: boolean;
  /** The side to use when both fit, `bottom` unless a field sits at the bottom of a window (the chat box). */
  prefer?: DropdownSide;
  gap?: number;
  edge?: number;
}

export interface Placement {
  side: DropdownSide;
  /** The room on the chosen side: the list is never taller than this. */
  maxHeight: number;
  /** The room across the window: the list is never wider than this. */
  maxWidth: number;
  /**
   * Where the list starts, in the direction of writing: the distance from the right edge of the
   * window in Urdu and from the left edge in English. CSS `inset-inline-start`.
   */
  inlineStart: number;
  /**
   * Below the field: the distance from the top of the window to the top of the list (`inset-block-start`).
   * Above it: the distance from the bottom of the window to the bottom of the list (`inset-block-end`),
   * so the list grows upward from the field whatever its height is.
   */
  block: number;
}

/** Whether two ranges share more than a point. */
const overlaps = (aStart: number, aEnd: number, bStart: number, bEnd: number) =>
  aStart < bEnd && aEnd > bStart;

/**
 * The part of the window a list may use: the window, less the bars that stay on screen above or
 * below the field, less a margin at every edge. A bar counts only when it shares columns with the
 * list (the assistant button at the corner does not matter to a list at the other end).
 */
export function freeArea(
  viewport: PlaceArgs['viewport'],
  anchor: Box,
  span: { minX: number; maxX: number },
  obstacles: readonly Box[],
  edge: number,
): Box {
  let top = 0;
  let bottom = viewport.height;
  for (const obstacle of obstacles) {
    if (!overlaps(obstacle.minX, obstacle.maxX, span.minX, span.maxX)) continue;
    // Below the field: the list stops above it. Above the field: the list starts under it. A bar
    // that covers the field itself (the search box of the top bar) takes nothing from either side.
    if (obstacle.top >= anchor.bottom) bottom = Math.min(bottom, obstacle.top);
    else if (obstacle.bottom <= anchor.top) top = Math.max(top, obstacle.bottom);
  }
  return {
    top: top + edge,
    bottom: bottom - edge,
    minX: edge,
    maxX: viewport.width - edge,
  };
}

export function placeDropdown({
  anchor,
  wanted,
  viewport,
  obstacles = [],
  rtl,
  prefer = 'bottom',
  gap = DROPDOWN_GAP_PX,
  edge = DROPDOWN_EDGE_PX,
}: PlaceArgs): Placement {
  // Sideways first: the list is as wide as it wants, at least as wide as its field, at most as
  // wide as the window, and starts where the field starts.
  const roomAcross = Math.max(viewport.width - 2 * edge, 0);
  const width = Math.min(Math.max(wanted.width, anchor.maxX - anchor.minX), roomAcross);
  const startAt = rtl ? anchor.maxX - width : anchor.minX;
  const minX = Math.min(Math.max(startAt, edge), Math.max(viewport.width - edge - width, edge));
  const area = freeArea(viewport, anchor, { minX, maxX: minX + width }, obstacles, edge);

  const below = Math.max(area.bottom - (anchor.bottom + gap), 0);
  const above = Math.max(anchor.top - gap - area.top, 0);
  const fitsBelow = wanted.height <= below;
  const fitsAbove = wanted.height <= above;
  // The preferred side when the list fits there, or when the other side has no more room; else the other.
  const side: DropdownSide =
    prefer === 'top'
      ? fitsAbove || above >= below
        ? 'top'
        : 'bottom'
      : fitsBelow || below >= above
        ? 'bottom'
        : 'top';

  return {
    side,
    maxHeight: side === 'bottom' ? below : above,
    maxWidth: roomAcross,
    inlineStart: rtl ? viewport.width - (minX + width) : minX,
    block: side === 'bottom' ? anchor.bottom + gap : viewport.height - (anchor.top - gap),
  };
}

/** The four sides Radix takes margins for. They are the names of its API, not a layout of ours. */
const RADIX_SIDES = ['top', 'right', 'bottom', 'left'] as const;

/**
 * The margins Radix menus, hovers and the select list keep from the window edge, so they flip and
 * shift by the same rule as the lists above: clear of the top bar and the status bar (design
 * tokens `--topbar-h` and `--statusbar-h`). Radix wants its four sides here, as numbers.
 */
export function radixCollisionPadding(): Partial<Record<(typeof RADIX_SIDES)[number], number>> {
  const top = tokenPx('--topbar-h', 60) + DROPDOWN_EDGE_PX;
  const bottom = tokenPx('--statusbar-h', 32) + DROPDOWN_EDGE_PX;
  return Object.fromEntries(
    RADIX_SIDES.map((side) => [
      side,
      side === 'top' ? top : side === 'bottom' ? bottom : DROPDOWN_EDGE_PX,
    ]),
  );
}
