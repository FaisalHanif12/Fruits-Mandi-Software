// Test helper: types the item lines of a purchase entry or of a goods arrived entry with the keys
// only, the way the munshi does (plan 35, PRD FR-12.41, FR-7.14). The cursor goes into the grid
// with a click on the item cell of a row; from there Enter walks every cell that can be typed in
// and skips the ones the backend works out. The cells are looked for inside the lines grid only,
// because a screen has other grids (the lists of purchases and entries).

import { screen, waitFor } from '@testing-library/react';
import type userEvent from '@testing-library/user-event';

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

/** Types a picker cell and takes the first match: waits for the list, so Enter finds it. */
async function pick(user: User, text: string): Promise<void> {
  await user.keyboard(text);
  await screen.findAllByRole('option');
  await user.keyboard('{Enter}');
}

/** Types one cell and goes on to the next typed cell with Enter. */
async function typeCell(user: User, text: string | undefined): Promise<void> {
  if (text !== undefined) await user.keyboard(text);
  await user.keyboard('{Enter}');
}

/**
 * Types line `row` of a purchase entry from its item cell to its shop name, leaving the cursor on
 * the item of the next line (Enter on the last cell goes to the next row).
 */
export async function typePurchaseLine(user: User, row: number, line: LineText): Promise<void> {
  await enterRow(user, row, 'purchase');
  await pick(user, line.item);
  await pick(user, line.packing);
  await typeCell(user, line.qty);
  await typeCell(user, line.weight);
  await typeCell(user, line.rate);
  await typeCell(user, line.freight);
  await typeCell(user, line.labour);
  await typeCell(user, line.otherCost);
  await typeCell(user, line.otherQty);
  await typeCell(user, line.shop);
}

/** Types line `row` of a goods arrived entry: the goods, the quantity, the weight and the split. */
export async function typeArrivalLine(user: User, row: number, line: LineText): Promise<void> {
  await enterRow(user, row, 'arrival');
  await pick(user, line.item);
  await pick(user, line.packing);
  await typeCell(user, line.qty);
  await typeCell(user, line.weight);
  await typeCell(user, line.otherQty);
  await typeCell(user, line.shop);
}
