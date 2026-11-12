// Test helper: the بکری بک screen opened in a language, and the keys of Event 1 (PRD 4.4) typed
// the way the munshi types them. The fake backend answers with the numbers of Event 1, so every
// number a test reads on the screen is a number the backend sent (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { Language } from '../../src/i18n';
import { fakeBikri } from './bikri';
import { renderApp } from './app';
import { message, visible } from './render';

export type User = ReturnType<typeof userEvent.setup>;

export const named = (text: string) => (name: string) => visible(name) === text;

/** The field called `id` in the app language (a picker, the truck number). */
export const combo = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });

/** The grid called `labelId` (the lots grid, the sale lines grid). */
export function gridOf(language: Language, labelId: string): HTMLElement {
  return screen.getByRole('group', { name: named(message(language, labelId)) });
}

export const lotsGrid = (language: Language) => gridOf(language, 'bikri-lots-label');
export const salesGrid = (language: Language) => gridOf(language, 'bikri-sales-label');

/** The cell of a grid, found by row and column. */
export function cellOf(grid: HTMLElement, rowIndex: number, field: string): HTMLElement {
  const found = grid.querySelector<HTMLElement>(
    `.ag-row[row-index="${String(rowIndex)}"] .ag-cell[col-id="${field}"]`,
  );
  if (found === null) throw new Error(`no cell ${String(rowIndex)}:${field}`);
  return found;
}

export const textOf = (grid: HTMLElement, rowIndex: number, field: string) =>
  visible(cellOf(grid, rowIndex, field).textContent).trim();

/**
 * Where focus is in a grid now, as "row:column". A picker cell opens its picker as soon as the
 * cell has the focus (plan 46, PRD FR-7.16): the field with the cursor is then in a popup outside
 * the grid, and its cell is the one the grid shows as focused.
 */
export function focusedIn(grid: HTMLElement): string | null {
  const active = document.activeElement;
  const inEditor = active?.closest('.ag-custom-component-popup') != null;
  const cell = inEditor
    ? grid.querySelector<HTMLElement>('.ag-cell-focus')
    : active?.closest<HTMLElement>('.ag-cell');
  if (cell == null || !grid.contains(cell)) return null;
  const row = cell.closest<HTMLElement>('.ag-row');
  return `${row?.getAttribute('row-index') ?? '?'}:${cell.getAttribute('col-id') ?? '?'}`;
}

/** The names the fake backend gives Event 1, in the language of the screen. */
export const names = (language: Language) =>
  language === 'ur'
    ? {
        rahim: 'رحیم',
        akram: 'اکرم ٹول ٹیکس',
        arshad: 'ارشد ملتانی',
        banana: 'کیلا',
        box: 'پیٹی',
        swat: 'سوات اڈا',
      }
    : {
        rahim: 'Rahim',
        akram: 'Akram Toll Tax',
        arshad: 'Arshad Multani',
        banana: 'Banana',
        box: 'Box',
        swat: 'Swat Adda',
      };

/** Opens the screen. `answer` decides what the preview says (default: Event 1 fitted to the draft). */
export async function openBook(
  language: Language,
  answer?: (draft: BikriDraft) => BikriPreview | Promise<BikriPreview>,
) {
  const backend = fakeBikri(answer);
  const app = await renderApp('/bikri/book');
  await screen.findByRole('heading', { level: 1, name: message(language, 'glossary-bikri-book') });
  return { ...app, backend, user: userEvent.setup() };
}

/**
 * Types the lot of Event 1 with the keys only. The header is only the date (plan 71 H1): it has the
 * cursor when the screen opens, and Enter goes on to the first lot's seller. The seller is typed as
 * his short code (`0` is Rahim, 0010); his item and packing come from his last lot. Then the lot's
 * cells. Ends with focus on the buyer of the first sale line.
 */
export async function typeHeaderAndLot(user: User, language: Language) {
  await user.keyboard('{Enter}');
  const lots = lotsGrid(language);
  await waitFor(() => {
    expect(focusedIn(lots)).toBe('0:seller');
  });
  await typeCodeAndEnter(user, '0');
  await waitFor(() => {
    expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
  });
  await waitFor(() => {
    expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
  });
  // Picking the seller moved on to the item: the item and the packing, then the quantity.
  await user.keyboard('{Enter}{Enter}50{Enter}3000{Enter}5{Enter}50{Enter}5000{Enter}');
  await waitFor(() => {
    expect(focusedIn(salesGrid(language))).toBe('0:buyer');
  });
}

/**
 * A new, empty بکری is open (as after a save): the cursor is on the date, the first field, and the
 * first lot has no seller. Waits, because the screen is made again a moment after the save.
 */
export async function expectNewEmptyBook(language: Language) {
  await waitFor(() => {
    expect(
      screen.getByRole('textbox', { name: named(message(language, 'bikri-header-date')) }),
    ).toHaveFocus();
    expect(textOf(lotsGrid(language), 0, 'seller')).toBe('');
  });
}

/** Types the two sale lines of Event 1: Akram 20 at 600 and Arshad 30 at 580. */
export async function typeTwoLines(user: User, language: Language) {
  const sales = salesGrid(language);
  await user.keyboard('0{Enter}');
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('0:qty');
  });
  // Buyer, qty, rate and the paid cell: four Enters, then the next line (PRD FR-12.44, NFR-USE-6).
  await user.keyboard('20{Enter}600{Enter}{Enter}');
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('1:buyer');
  });
  await user.keyboard('1{Enter}');
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('1:qty');
  });
  await user.keyboard('{Enter}580{Enter}{Enter}');
}

/** Types a picker field and takes the first match: waits for the list, so Enter finds it. */
export async function typeAndPick(user: User, text: string) {
  await user.keyboard(text);
  await screen.findAllByRole('option');
  await user.keyboard('{Enter}');
}

/**
 * Types Event 1 the way the backend's request files have it: the seller by his short code, the lot
 * (50 نگ, freight 3,000, labour 5 for each نگ, munshiana 50, phone charge 20, نقد 5,000). The method
 * of the labour and the phone charge are in the "more" panel of the lot. Ends with the cursor on the
 * buyer of the first sale line.
 */
export async function typeEvent1Lot(user: User, language: Language) {
  await user.keyboard('{Enter}');
  const lots = lotsGrid(language);
  await waitFor(() => {
    expect(focusedIn(lots)).toBe('0:seller');
  });
  await typeCodeAndEnter(user, '0');
  // His item and packing come from his last lot.
  await waitFor(() => {
    expect(textOf(lots, 0, 'seller')).toBe(names(language).rahim);
  });
  await waitFor(() => {
    expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
  });
  await user.keyboard('{Enter}{Enter}50{Enter}3000{Enter}5{Enter}50{Enter}5000{Enter}');
  await waitFor(() => {
    expect(focusedIn(salesGrid(language))).toBe('0:buyer');
  });
  await user.click(
    // The button names the lot after its number once the seller or item is chosen (PRD FR-12.6),
    // so it is found by the words that come first.
    screen.getByRole('button', {
      name: (label) =>
        visible(label).startsWith(
          message(language, 'bikri-more-toggle', { lot: 1, who: '', named: 'no' }),
        ),
    }),
  );
  const labour = await screen.findByRole('group', {
    name: named(message(language, 'glossary-mazdoori')),
  });
  await user.click(
    within(labour).getByRole('button', { name: named(message(language, 'bikri-method-per-unit')) }),
  );
  await user.click(
    screen.getByRole('textbox', { name: named(message(language, 'glossary-phone')) }),
  );
  await user.keyboard('20');
  await user.click(cellOf(salesGrid(language), 0, 'buyer'));
}

/**
 * Types a code into the picker cell the cursor is on and presses Enter. The first key opens the
 * cell's editor; the rest is typed once the editor has the focus, and Enter is pressed once the
 * list has answered. An Enter that comes before the list is open is the grid's Enter, which closes
 * the cell without taking the code, so on a busy computer the helper waits for the list, as a
 * person looking at the screen does.
 */
export async function typeCodeAndEnter(user: User, code: string) {
  await user.keyboard(code.slice(0, 1));
  await waitFor(
    () => {
      expect(document.activeElement?.tagName).toBe('INPUT');
    },
    { timeout: 8_000 },
  );
  if (code.length > 1) await user.keyboard(code.slice(1));
  await screen.findAllByRole('option', undefined, { timeout: 8_000 });
  await user.keyboard('{Enter}');
}

/**
 * Waits until the first sale line has its quantity: the screen fills it from the lot's remaining
 * quantity a moment after the lot is typed (PRD FR-7.52). A test that checks the cells of the line
 * waits for it first, so it reads the filled value. (A fill that lands while a cell is being typed
 * in does not close the cell: bikri-fill-while-typing.test.tsx.)
 */
export async function waitForLineFill(language: Language) {
  await waitFor(
    () => {
      expect(textOf(salesGrid(language), 0, 'qty')).not.toBe('');
    },
    { timeout: 8_000 },
  );
}
