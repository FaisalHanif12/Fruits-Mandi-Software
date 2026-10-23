// The text side of the جنرل screen (Code Standards F39, PRD 16.4, FR-13.5): the lines as the grid's
// store holds them, the one-time person chosen for a پختہ کھاتہ line, the entry the backend takes,
// and the way a problem at `lines[2].dr` finds its row and cell again. Nothing here works out money
// (CLAUDE.md rule 1): the totals, the balance check and the split are the backend's.

import type { UiErrorItem } from '../../bindings/UiErrorItem';
import type { PukhtaPerson } from '../../bindings/PukhtaPerson';
import type { EntryRow } from '../../shared/grid';
import { isPukhtaCode } from '../pukhta_khata';
import type { GeneralInput, GeneralLineInput, GeneralPreviewInput } from './types';

/** The cells of a line. `person` is read only: it shows the one-time person chosen below the grid. */
export const GENERAL_ACCOUNT = 'account';
export const GENERAL_PERSON = 'person';
export const GENERAL_DR = 'dr';
export const GENERAL_CR = 'cr';
export const GENERAL_NARRATION = 'narration';

/** The voucher type of a جنرل entry: picks its number series and its saved message. */
export const GENERAL_KIND = 'general';

/** The lines a new entry starts with: a جنرل entry needs two. */
export const GENERAL_START_LINES = 2;

/**
 * The one-time person of a line on a پختہ کھاتہ account, by row. `text` is what is typed in his
 * field; `person` is the one taken from the list (PRD FR-7.29). Kept above the grid, so a language
 * switch loses nothing (F39).
 */
export interface PersonChoice {
  text: string;
  person: PukhtaPerson | null;
}
export type PeopleMap = Readonly<Record<string, PersonChoice>>;

/** The paths the form itself shows next to its fields, so no problem is told twice. */
export const GENERAL_FIELD_PATHS: readonly string[] = ['date', 'narration'];

function typedOf(row: EntryRow, field: string): string {
  return (row.typed[field] ?? '').trim();
}

/**
 * A line with no account, no amount and no note is a blank row: the preview and the save ignore
 * it (the same rule as the blank row of the sale lines, PRD FR-7.52).
 */
export function isBlankLine(row: EntryRow): boolean {
  return (
    row.picked?.[GENERAL_ACCOUNT] === undefined &&
    typedOf(row, GENERAL_DR) === '' &&
    typedOf(row, GENERAL_CR) === '' &&
    typedOf(row, GENERAL_NARRATION) === ''
  );
}

/** The lines to send, and the row each one came from, in the same order. */
export interface SentLines {
  lines: GeneralLineInput[];
  rowIds: string[];
}

export function sentLines(rows: readonly EntryRow[], people: PeopleMap): SentLines {
  const lines: GeneralLineInput[] = [];
  const rowIds: string[] = [];
  for (const row of rows) {
    if (isBlankLine(row)) continue;
    const note = typedOf(row, GENERAL_NARRATION);
    // A person goes with a line only while the line is on a پختہ کھاتہ account (PRD FR-13.5).
    const account = row.picked?.[GENERAL_ACCOUNT];
    lines.push({
      account_id: account?.id ?? null,
      person_key: isPukhtaCode(account?.code)
        ? (people[row.rowId]?.person?.person_key ?? null)
        : null,
      dr: typedOf(row, GENERAL_DR),
      cr: typedOf(row, GENERAL_CR),
      narration: note === '' ? null : note,
    });
    rowIds.push(row.rowId);
  }
  return { lines, rowIds };
}

/** True once anything was typed, so leaving asks first (PRD FR-7.9). */
export function generalIsFilled(narration: string, rows: readonly EntryRow[]): boolean {
  return narration.trim() !== '' || rows.some((row) => !isBlankLine(row));
}

function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** The input of `general_preview`. With a date it also asks for the split (PRD FR-6.5). */
export function buildGeneralPreview(date: string, lines: readonly GeneralLineInput[]) {
  const input: GeneralPreviewInput = { date: textOrNull(date), lines: [...lines] };
  return input;
}

/** The input of `general_save`, with the answer to the cash question (PRD FR-6.7). */
export function buildGeneral(
  date: string,
  narration: string,
  lines: readonly GeneralLineInput[],
  confirm: boolean,
): GeneralInput {
  return {
    date: date.trim(),
    narration: textOrNull(narration),
    lines: [...lines],
    confirm_cash_below_zero: confirm,
  };
}

/** The cell a field of a line is shown in. */
const CELL_OF_FIELD: Readonly<Record<string, string>> = {
  account_id: GENERAL_ACCOUNT,
  person_key: GENERAL_PERSON,
  dr: GENERAL_DR,
  cr: GENERAL_CR,
  narration: GENERAL_NARRATION,
};

/** `lines[3].dr` as `{ index: 3, field: 'dr' }`; null for any other path. */
export function linePathOf(path: string): { index: number; field: string } | null {
  const match = /^lines\[(\d+)\]\.(\w+)$/.exec(path);
  if (match === null) return null;
  return { index: Number(match[1]), field: match[2] ?? '' };
}

export interface LineProblem {
  rowId: string;
  /** The cell it belongs to. */
  cell: string;
  item: UiErrorItem;
}

/**
 * The problems of the lines, each on its row and cell. A problem that only says "this is empty"
 * waits until the user tries to save, so a cell does not turn red before he has reached it
 * (PRD FR-7.6). A path that points at no known cell is left to the notice under the grid.
 */
export function lineProblems(
  errors: readonly UiErrorItem[],
  rowIds: readonly string[],
  tried: boolean,
): { onCells: LineProblem[]; rest: UiErrorItem[] } {
  const onCells: LineProblem[] = [];
  const rest: UiErrorItem[] = [];
  for (const item of errors) {
    if (item.code === 'ERR_REQUIRED' && !tried) continue;
    const parsed = item.field === null ? null : linePathOf(item.field);
    const rowId = parsed === null ? undefined : rowIds[parsed.index];
    const cell = parsed === null ? undefined : CELL_OF_FIELD[parsed.field];
    if (rowId === undefined || cell === undefined) rest.push(item);
    else onCells.push({ rowId, cell, item });
  }
  return { onCells, rest };
}
