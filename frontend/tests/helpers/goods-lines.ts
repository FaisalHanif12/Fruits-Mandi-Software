// Test helper: types the item lines of a purchase entry or of a goods arrived entry with the keys
// only, the way the munshi does (plan 35, PRD FR-12.41, FR-7.14). The cursor goes into the grid
// with a click on the item cell of a row; from there Enter walks every cell that can be typed in
// and skips the ones the backend works out. The cells are looked for inside the lines grid only,
// because a screen has other grids (the lists of purchases and entries).

import { screen, waitFor } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';
import { expect } from 'vitest';

type User = ReturnType<typeof userEvent.setup>;

export type LinesKind = 'purchase' | 'arrival';

/** What to type on one line. A cell left out is passed over with Enter. */
export interface LineText {
  /** The text to find the item with, and the one to find the packing with. */
  item: string;
  packing: string;
  qty: string;
  weight?: string;
  rate?: string;
  freight?: string;
  labour?: string;
  otherCost?: string;
  /** The نگ for another shop, and the shop's name. */
  otherQty?: string;
  shop?: string;
}

/** The section of the lines grid, or null when the screen has none open. */
export function linesSection(kind: LinesKind = 'purchase'): HTMLElement | null {
  return document.querySelector<HTMLElement>(
    `section[aria-labelledby="goods-lines-heading-${kind}"]`,
  );
}

/** The cell of a row of the lines grid, by its column. */
export function lineCell(row: number, field: string, kind: LinesKind = 'purchase'): HTMLElement {
  const found = linesSection(kind)?.querySelector<HTMLElement>(
    `.ag-row[row-index="${String(row)}"] .ag-cell[col-id="${field}"]`,
  );
  if (found === null || found === undefined) throw new Error(`no cell ${String(row)}:${field}`);
  return found;
}

/** What the cell shows, as text, without the direction marks. */
export function lineCellText(row: number, field: string, kind: LinesKind = 'purchase'): string {
  return lineCell(row, field, kind)
    .textContent.replace(/[‎‏⁦⁩]/g, '')
    .trim();
}

/** Waits for the grid to be drawn, then puts the cursor on the item cell of a row. */
async function enterRow(user: User, row: number, kind: LinesKind): Promise<void> {
  await waitFor(() => {
    lineCell(row, 'item', kind);
  });
  await user.click(lineCell(row, 'item', kind));
}

/** How many times a pick is tried when a busy machine loses it (see `pick`). */
const PICK_TRIES = 3;
/** How long one step of a pick is waited for before it counts as lost. */
const PICK_STEP_MS = 4_000;

/** Text for comparing: no direction marks, lower case. */
const plain = (text: string | null): string =>
  (text ?? '').replace(/[\u200e\u200f\u2066-\u2069]/g, '').toLowerCase();

/** Where a pick is made: the cell of the line that holds the picker. */
interface PickCell {
  row: number;
  kind: LinesKind;
  field: 'item' | 'packing';
}

/** The option the open picker has highlighted, or null when none is. */
function highlightedOption(): HTMLElement | null {
  const id = document.activeElement?.getAttribute('aria-activedescendant') ?? '';
  return id === '' ? null : document.getElementById(id);
}

/**
 * Types a picker cell and takes the first match. A picker opens as soon as its cell has the focus
 * (plan 46), so a list is on screen before the first letter is typed, and a list that is drawn
 * again has no row highlighted for a moment. Enter must wait until the highlighted row is one that
 * holds the typed text, or it takes nothing, or the first row of the whole list. The pick has
 * landed only when the cell shows a name. On a busy machine a pick can be lost: the editor is left,
 * the cell entered again and the text typed again, up to three times.
 */
async function pick(user: User, text: string, cell: PickCell): Promise<void> {
  for (let attempt = 1; attempt <= PICK_TRIES; attempt += 1) {
    if (attempt > 1) {
      await user.keyboard('{Escape}');
      await user.click(lineCell(cell.row, cell.field, cell.kind));
    }
    await user.keyboard(text);
    try {
      await waitFor(
        () => {
          expect(plain(highlightedOption()?.textContent ?? null)).toContain(plain(text));
        },
        { timeout: PICK_STEP_MS },
      );
    } catch {
      // Nothing is highlighted, or a text the list does not show as typed (a code): take the first
      // row with the arrow key, once the list is there.
      await screen.findAllByRole('option');
      if (highlightedOption() === null) await user.keyboard('{ArrowDown}');
    }
    await user.keyboard('{Enter}');
    try {
      await waitFor(
        () => {
          expect(lineCellText(cell.row, cell.field, cell.kind)).not.toBe('');
        },
        { timeout: PICK_STEP_MS },
      );
      return;
    } catch (lost) {
      if (attempt === PICK_TRIES) throw lost;
    }
  }
}

/** Types one stop of the line and presses Enter, which goes on to the next stop. */
async function typeStop(user: User, text: string | undefined): Promise<void> {
  if (text !== undefined) await user.keyboard(text);
  await user.keyboard('{Enter}');
}

/**
 * Types the cells Enter does not stop at (freight, labour, other cost, the part for another shop
 * and the shop's name, PRD FR-12.41): a click on the cell, then the text. Clicking the item of the
 * next line afterwards finishes the last one, as the user's mouse would.
 */
async function typeOptional(
  user: User,
  row: number,
  kind: LinesKind,
  cells: readonly (readonly [string, string | undefined])[],
): Promise<void> {
  for (const [field, text] of cells) {
    if (text === undefined) continue;
    await user.click(lineCell(row, field, kind));
    await user.keyboard(text);
  }
  // Enter on the last stop made the next line: its item is where the cursor goes next.
  await waitFor(() => {
    lineCell(row + 1, 'item', kind);
  });
  await user.click(lineCell(row + 1, 'item', kind));
}

/**
 * Types line `row` of a purchase entry the way the munshi does: item, packing, quantity and rate
 * with Enter after each (four Enters), and the weight too when the item is sold by weight (give
 * `weight`). Enter on the rate goes to the item of the next line. The optional cells are clicked.
 */
export async function typePurchaseLine(user: User, row: number, line: LineText): Promise<void> {
  await enterRow(user, row, 'purchase');
  await pick(user, line.item, { row, kind: 'purchase', field: 'item' });
  await pick(user, line.packing, { row, kind: 'purchase', field: 'packing' });
  await typeStop(user, line.qty);
  if (line.weight !== undefined) await typeStop(user, line.weight);
  await typeStop(user, line.rate);
  await typeOptional(user, row, 'purchase', [
    ['freight', line.freight],
    ['labour', line.labour],
    ['otherCost', line.otherCost],
    ['otherQty', line.otherQty],
    ['shop', line.shop],
  ]);
}

/** Types line `row` of a goods arrived entry: item, packing and quantity (three Enters). */
export async function typeArrivalLine(user: User, row: number, line: LineText): Promise<void> {
  await enterRow(user, row, 'arrival');
  await pick(user, line.item, { row, kind: 'arrival', field: 'item' });
  await pick(user, line.packing, { row, kind: 'arrival', field: 'packing' });
  await typeStop(user, line.qty);
  if (line.weight !== undefined) await typeStop(user, line.weight);
  await typeOptional(user, row, 'arrival', [
    ['otherQty', line.otherQty],
    ['shop', line.shop],
  ]);
}
