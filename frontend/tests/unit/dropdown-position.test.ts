// Where an open dropdown list goes (plan 86, NFR-USE-16): below its field when it fits, above it when the
// room below is too small and there is more above, its height cut to the room on that side, inside the
// window sideways in both directions, clear of the bars of the shell. Pure sizes in, a placement out.

import { describe, expect, it } from 'vitest';
import { freeArea, placeDropdown, type Box } from '../../src/shared/ui/dropdown-position';

const VIEWPORT = { width: 1366, height: 768 };
/** The status bar (32 px at the bottom) and the top bar (60 px at the top) of the shell. */
const STATUS_BAR: Box = { top: 736, bottom: 768, minX: 0, maxX: 1366 };
const TOP_BAR: Box = { top: 0, bottom: 60, minX: 0, maxX: 1366 };
/** The round assistant button at the bottom end corner, above the status bar. */
const LAUNCHER_AT_END: Box = { top: 650, bottom: 720, minX: 1270, maxX: 1340 };

/** A field 44 px high whose top edge is at `top`, 280 px wide, starting `minX` from the left edge. */
const field = (top: number, minX = 300): Box => ({
  top,
  bottom: top + 44,
  minX,
  maxX: minX + 280,
});

const LIST = { width: 280, height: 430 };

describe.each([false, true])('placeDropdown (rtl: %s)', (rtl) => {
  it('opens below a field near the top when the whole list fits', () => {
    const placement = placeDropdown({
      anchor: field(100),
      wanted: LIST,
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, TOP_BAR],
      rtl,
    });
    expect(placement.side).toBe('bottom');
    // 100 + 44 + the 4 px gap
    expect(placement.block).toBe(148);
  });

  it('flips above a field near the bottom when the room below is too small and there is more above', () => {
    const placement = placeDropdown({
      anchor: field(560),
      wanted: LIST,
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, TOP_BAR],
      rtl,
    });
    expect(placement.side).toBe('top');
    // tied to the bottom edge of the window: 768 - (560 - 4)
    expect(placement.block).toBe(212);
  });

  it('keeps the list below when neither side has room for it and below has more', () => {
    const placement = placeDropdown({
      anchor: field(200),
      wanted: { width: 280, height: 900 },
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, TOP_BAR],
      rtl,
    });
    expect(placement.side).toBe('bottom');
    expect(placement.maxHeight).toBeLessThan(900);
  });

  it('uses the side with more room when neither side fits the list', () => {
    const placement = placeDropdown({
      anchor: field(520),
      wanted: { width: 280, height: 900 },
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, TOP_BAR],
      rtl,
    });
    expect(placement.side).toBe('top');
  });

  it('cuts the height to the room on the chosen side, clear of the bars and the edges', () => {
    const below = placeDropdown({
      anchor: field(300),
      wanted: { width: 280, height: 900 },
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, TOP_BAR],
      rtl,
    });
    // the status bar starts at 736; 8 px edge; field bottom 344 and a 4 px gap
    expect(below.side).toBe('bottom');
    expect(below.maxHeight).toBe(736 - 8 - (344 + 4));

    const above = placeDropdown({
      anchor: field(560),
      wanted: { width: 280, height: 900 },
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, TOP_BAR],
      rtl,
    });
    // the top bar ends at 60; 8 px edge; field top 560 and a 4 px gap
    expect(above.side).toBe('top');
    expect(above.maxHeight).toBe(560 - 4 - (60 + 8));
  });

  it('prefers the side it is told to when both fit, and the other when only the other fits', () => {
    const args = { wanted: { width: 280, height: 200 }, viewport: VIEWPORT, rtl };
    expect(placeDropdown({ ...args, anchor: field(300), prefer: 'top' }).side).toBe('top');
    expect(placeDropdown({ ...args, anchor: field(300) }).side).toBe('bottom');
    // no room above the field: a list that prefers to open upward goes below
    expect(placeDropdown({ ...args, anchor: field(20), prefer: 'top' }).side).toBe('bottom');
  });

  it('stays clear of the assistant button only where the list is over it', () => {
    // a list too tall for either side, so it goes below (more room) and is cut to the room there
    const wanted = { width: 280, height: 900 };
    const overButton = placeDropdown({
      anchor: field(300, 1060),
      wanted,
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, LAUNCHER_AT_END],
      rtl: false,
    });
    // spans 1060 to 1340, over the button (top 650): 650 - 8 - (344 + 4)
    expect(overButton.side).toBe('bottom');
    expect(overButton.maxHeight).toBe(650 - 8 - (344 + 4));
    const awayFromButton = placeDropdown({
      anchor: field(300, 300),
      wanted,
      viewport: VIEWPORT,
      obstacles: [STATUS_BAR, LAUNCHER_AT_END],
      rtl: false,
    });
    // far from the button only the status bar counts: 736 - 8 - (344 + 4)
    expect(awayFromButton.side).toBe('bottom');
    expect(awayFromButton.maxHeight).toBe(736 - 8 - (344 + 4));
  });
});

describe('placeDropdown sideways', () => {
  it('starts at the start edge of the field and moves back into the window (English)', () => {
    const inside = placeDropdown({
      anchor: field(100, 300),
      wanted: LIST,
      viewport: VIEWPORT,
      rtl: false,
    });
    expect(inside.inlineStart).toBe(300);
    // a field at the end of the window with a list wider than the field
    const atEnd = placeDropdown({
      anchor: field(100, 1200),
      wanted: { width: 420, height: 200 },
      viewport: VIEWPORT,
      rtl: false,
    });
    // 1366 - 8 - 420 = 938 is the last start that keeps the list inside
    expect(atEnd.inlineStart).toBe(938);
  });

  it('starts at the right edge of the field, measured from the right edge of the window (Urdu)', () => {
    const inside = placeDropdown({
      anchor: field(100, 300),
      wanted: LIST,
      viewport: VIEWPORT,
      rtl: true,
    });
    // the field ends at 580 from the left, so 1366 - 580 = 786 from the right
    expect(inside.inlineStart).toBe(786);
    // a field at the start (right) edge of the window with a wider list
    const atRight = placeDropdown({
      anchor: { top: 100, bottom: 144, minX: 1000, maxX: 1360 },
      wanted: { width: 420, height: 200 },
      viewport: VIEWPORT,
      rtl: true,
    });
    // the list would leave the window on the right: it is moved back by the edge margin
    expect(atRight.inlineStart).toBe(8);
    // a field at the left edge of the window: the list is pushed right, off the left edge
    const atLeft = placeDropdown({
      anchor: { top: 100, bottom: 144, minX: 0, maxX: 200 },
      wanted: { width: 420, height: 200 },
      viewport: VIEWPORT,
      rtl: true,
    });
    // left edge at 8: 1366 - (8 + 420) from the right
    expect(atLeft.inlineStart).toBe(VIEWPORT.width - (8 + 420));
  });

  it('is never wider than the window', () => {
    const placement = placeDropdown({
      anchor: field(100, 300),
      wanted: { width: 5000, height: 200 },
      viewport: { width: 1024, height: 600 },
      rtl: false,
    });
    expect(placement.maxWidth).toBe(1024 - 16);
    expect(placement.inlineStart).toBe(8);
  });
});

describe('freeArea', () => {
  it('leaves out a bar that covers the field itself (the search box in the top bar)', () => {
    const area = freeArea(
      VIEWPORT,
      { top: 8, bottom: 52, minX: 300, maxX: 900 },
      { minX: 300, maxX: 900 },
      [TOP_BAR],
      8,
    );
    expect(area.top).toBe(8);
  });
});
