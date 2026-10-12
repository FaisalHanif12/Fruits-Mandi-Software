// The opening balances grid as plain logic (plan 07, PRD FR-10.9): the rows of the grid from the
// accounts, which rows are lines to send, and where each problem of the backend belongs. Pure, so
// it is tested without a page. Nothing here works out money: the cells hold the text the user
// typed, and a line goes to the backend as that text (CLAUDE.md rule 1).

import type { UiError } from '../../bindings/UiError';
import type { UiErrorItem } from '../../bindings/UiErrorItem';
import { formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import type { EntryRow } from '../../shared/grid';
import type { AccountRow, OpeningLineInput, OpeningsPreview } from '../../shared/accounts';

/** The cells the user types in. They are also the field names of the grid's columns. */
export const DATE_FIELD = 'opening_date';
export const DR_FIELD = 'opening_dr';
export const CR_FIELD = 'opening_cr';
const ACCOUNT_FIELD = 'account_id';

/** What an account looked like when the grid was drawn, to tell what the user changed. */
export interface OpeningBaseline {
  /** The typed cells as they were first shown. */
  typed: Readonly<Record<string, string>>;
  /** The account had an opening already. One without gets the common date until it has an amount. */
  hasOpening: boolean;
  version: number;
  /** The opening debit of this seller counts as an advance (shown, and kept when he has a debit). */
  isAdvance: boolean;
}

/** An amount from the backend as the text the cell shows: nothing for no amount. */
function amountText(amount: AccountRow['opening_dr_paisa'] | null, grouping: Grouping): string {
  if (amount === null) return '';
  const text = formatPaisa(amount, { grouping });
  return text === '0' ? '' : text;
}

/** The typed cells of one account's row. `commonDate` is the date of an account with no opening. */
export function openingTyped(
  account: AccountRow,
  grouping: Grouping,
  commonDate: string,
): Record<string, string> {
  return {
    [ACCOUNT_FIELD]: String(account.id),
    [DATE_FIELD]:
      account.opening_date === null ? commonDate : isoToDisplayDate(account.opening_date),
    [DR_FIELD]: amountText(account.opening_dr_paisa, grouping),
    [CR_FIELD]: amountText(account.opening_cr_paisa, grouping),
  };
}

export function baselineOf(
  account: AccountRow,
  typed: Readonly<Record<string, string>>,
): OpeningBaseline {
  return {
    typed,
    hasOpening: account.opening_date !== null,
    version: account.version,
    isAdvance: account.account_type === 'seller' && account.opening_is_advance,
  };
}

/** A row the user changed, with the line that goes to the backend. */
export interface ChangedLine {
  rowId: string;
  line: OpeningLineInput;
}

/**
 * The rows the user changed, in the order of the grid. An account with no opening is a line only
 * when an amount was typed (its date follows the common date and says nothing by itself); an
 * account that had an opening is a line when its date or an amount changed.
 */
export function changedLines(
  rows: readonly EntryRow[],
  baselines: ReadonlyMap<string, OpeningBaseline>,
): ChangedLine[] {
  const changed: ChangedLine[] = [];
  for (const row of rows) {
    const id = row.typed[ACCOUNT_FIELD];
    const base = id === undefined ? undefined : baselines.get(id);
    if (id === undefined || base === undefined) continue;
    const dr = row.typed[DR_FIELD] ?? '';
    const cr = row.typed[CR_FIELD] ?? '';
    const date = row.typed[DATE_FIELD] ?? '';
    const amountChanged = dr !== base.typed[DR_FIELD] || cr !== base.typed[CR_FIELD];
    const dateChanged = base.hasOpening && date !== base.typed[DATE_FIELD];
    if (!amountChanged && !dateChanged) continue;
    changed.push({
      rowId: row.rowId,
      line: {
        account_id: Number(id),
        version: base.version,
        opening_date: date,
        opening_dr: dr,
        opening_cr: cr,
        // The advance mark stays with the debit: without a debit there is nothing to mark.
        opening_is_advance: base.isAdvance && dr !== '',
      },
    });
  }
  return changed;
}

/** One problem on one cell of the grid. */
export interface LineProblem {
  rowId: string;
  /** The column of the grid it belongs to. */
  field: string;
  item: UiErrorItem;
}

/** A problem about the advance mark belongs to the debit cell, which is where it is typed. */
function gridField(field: string): string {
  return field === 'opening_is_advance' ? DR_FIELD : field;
}

/** The problems the preview found, each on the row and cell it belongs to. */
export function problemsOfPreview(
  changed: readonly ChangedLine[],
  preview: OpeningsPreview,
): LineProblem[] {
  const problems: LineProblem[] = [];
  preview.rows.forEach((row, index) => {
    const line = changed[index];
    if (line === undefined) return;
    for (const error of row.errors) {
      problems.push({
        rowId: line.rowId,
        field: gridField(error.field),
        item: { code: error.code, field: error.field, values: {} },
      });
    }
  });
  return problems;
}

const LINE_PATH = /^lines\[(\d+)\]\.(.+)$/;

export interface SplitProblems {
  /** The problems that point at a line of the grid. */
  problems: LineProblem[];
  /** Everything else (a refused date, a busy database): shown as a notice. */
  rest: UiError | null;
}

/** The problems of a refused save: `lines[i].field` goes to the i-th changed row's cell. */
export function problemsOfError(error: UiError, changed: readonly ChangedLine[]): SplitProblems {
  const problems: LineProblem[] = [];
  const rest: UiErrorItem[] = [];
  for (const item of error.errors) {
    const match = item.field === null ? null : LINE_PATH.exec(item.field);
    const line = match === null ? undefined : changed[Number(match[1])];
    if (match === null || line === undefined) {
      rest.push(item);
      continue;
    }
    problems.push({ rowId: line.rowId, field: gridField(match[2] ?? ''), item });
  }
  return { problems, rest: rest.length === 0 ? null : { ...error, errors: rest } };
}
