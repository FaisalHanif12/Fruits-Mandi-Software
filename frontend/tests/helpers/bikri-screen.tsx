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

/** Where focus is in a grid now, as "row:column". */
export function focusedIn(grid: HTMLElement): string | null {
  const active = document.activeElement;
  const cell = active?.closest<HTMLElement>('.ag-cell');
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
 * Types the header and the lot of Event 1 with the keys only: the truck (which fills the اڈا, the
 * زمیندار and the driver), then Enter through the filled fields to the lots, then the lot's cells.
 * Ends with focus on the buyer of the first sale line.
 */
export async function typeHeaderAndLot(user: User, language: Language) {
  await user.keyboard('LES-1234{Enter}');
  await waitFor(() => {
    expect(
      screen.getByRole('combobox', { name: named(message(language, 'glossary-zamindar')) }),
    ).toHaveValue(names(language).rahim);
  });
  await user.keyboard('{Enter}{Enter}{Enter}');
  const lots = lotsGrid(language);
  await waitFor(() => {
    expect(focusedIn(lots)).toBe('0:seller');
  });
  await waitFor(() => {
    expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
  });
  await user.keyboard('{Enter}{Enter}{Enter}50{Enter}3000{Enter}5{Enter}50{Enter}5000{Enter}');
  await waitFor(() => {
    expect(focusedIn(salesGrid(language))).toBe('0:buyer');
  });
}

/** Types the two sale lines of Event 1: Akram 20 at 600 and Arshad 30 at 580. */
export async function typeTwoLines(user: User, language: Language) {
  const sales = salesGrid(language);
  await user.keyboard('1{Enter}');
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('0:qty');
  });
  await user.keyboard('20{Enter}600{Enter}');
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('1:buyer');
  });
  await user.keyboard('2{Enter}');
  await waitFor(() => {
    expect(focusedIn(sales)).toBe('1:qty');
  });
  await user.keyboard('{Enter}580{Enter}');
}

/** Types a picker field and takes the first match: waits for the list, so Enter finds it. */
export async function typeAndPick(user: User, text: string) {
  await user.keyboard(text);
  await screen.findAllByRole('option');
  await user.keyboard('{Enter}');
}

/**
 * Types Event 1 the way the backend's request files have it, with no truck memory: the truck, the
 * زمیندار picked by name, the lot (50 نگ, freight 3,000, labour 5 for each نگ, munshiana 50,
 * phone charge 20, نقد 5,000). The method of the labour and the phone charge are in the "more"
 * panel of the lot. Ends with the cursor on the buyer of the first sale line.
 */
export async function typeEvent1Lot(user: User, language: Language) {
  await user.keyboard('LES-1234{Enter}');
  await typeAndPick(user, 'rahim');
  // The adda and the total freight stay empty; the freight is typed on the lot.
  await user.keyboard('{Enter}{Enter}');
  const lots = lotsGrid(language);
  await waitFor(() => {
    expect(focusedIn(lots)).toBe('0:seller');
  });
  // The seller is the زمیندار, and his item and packing come from his last lot.
  await waitFor(() => {
    expect(textOf(lots, 0, 'packing')).toBe(names(language).box);
  });
  await user.keyboard('{Enter}{Enter}{Enter}50{Enter}3000{Enter}5{Enter}50{Enter}5000{Enter}');
  await waitFor(() => {
    expect(focusedIn(salesGrid(language))).toBe('0:buyer');
  });
  await user.click(
    screen.getByRole('button', { name: named(message(language, 'bikri-more-toggle', { lot: 1 })) }),
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
