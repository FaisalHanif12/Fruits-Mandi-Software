// Arrow keys in forms (PRD FR-7.13, FR-7.15): the arrow points at the field the user sees in that
// direction, whatever the layout. The pure parts live here so they can be tested without a page.

import type { Direction } from '../i18n';
import type { Side } from './direction';

/** The box of a field on screen. A DOMRect has these. */
export interface Box {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

export type Move = 'left' | 'right' | 'up' | 'down';

function centre(box: Box): { x: number; y: number } {
  return { x: (box.left + box.right) / 2, y: (box.top + box.bottom) / 2 };
}

/** Two boxes sit in the same row when their vertical ranges overlap by more than half. */
function sameRow(a: Box, b: Box): boolean {
  const overlap = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
  return overlap > Math.min(a.bottom - a.top, b.bottom - b.top) / 2;
}

/**
 * The index of the field the user sees in `move` direction from field `from`, or null when there
 * is none. Left and right stay in the row. Up and down go to the nearest row above or below, and
 * in it to the field whose centre is closest across.
 */
export function pickNeighbour(boxes: readonly Box[], from: number, move: Move): number | null {
  const current = boxes[from];
  if (current === undefined) return null;
  const here = centre(current);
  let best: number | null = null;
  let bestScore = Number.POSITIVE_INFINITY;
  boxes.forEach((box, index) => {
    if (index === from) return;
    const there = centre(box);
    let score: number;
    if (move === 'left' || move === 'right') {
      if (!sameRow(current, box)) return;
      const along = move === 'left' ? here.x - there.x : there.x - here.x;
      if (along <= 0) return;
      score = along;
    } else {
      const along = move === 'up' ? here.y - there.y : there.y - here.y;
      if (along <= 0 || sameRow(current, box)) return;
      // The nearest row first, then the closest across.
      score = along * 10_000 + Math.abs(there.x - here.x);
    }
    if (score < bestScore) {
      best = index;
      bestScore = score;
    }
  });
  return best;
}

interface CaretText {
  value: string;
  selectionStart: number | null;
  selectionEnd: number | null;
}

/**
 * Whether ← or → should leave a text field (PRD FR-7.15). While the whole value is selected,
 * which it is when the user enters the field, the arrow leaves. Once the user types or moves the
 * cursor, the arrow moves the cursor, and leaves only when the cursor is already at the visual
 * edge in the arrow's direction. In right-to-left text the visual left edge is the end of the
 * text, so ← is at its edge when the cursor is after the last character.
 */
export function shouldLeaveField(input: CaretText, side: Side, direction: Direction): boolean {
  const length = input.value.length;
  const start = input.selectionStart ?? length;
  const end = input.selectionEnd ?? length;
  if (length === 0) return true;
  if (start === 0 && end === length) return true;
  if (start !== end) return false;
  const edgeIsStartOfText = (side === 'left') === (direction === 'ltr');
  return edgeIsStartOfText ? start === 0 : start === length;
}
