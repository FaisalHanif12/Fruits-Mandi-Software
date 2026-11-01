// The marks of earlier edits on the cells of a جنرل entry open for edit (plan 44, PRD FR-7.71): the
// history names a line by its place in the entry (`lines[1].amount_paisa`), and the lines of the
// screen are in the same order as the lines that were sent. A line that changed its amount marks the
// cell of the side it is on (بنام or جمع). Display only: the tip is the words the history gave.

import type { EntryMarks } from '../../shared/edit';
import type { EntryRow } from '../../shared/grid';
import { GENERAL_ACCOUNT, GENERAL_CR, GENERAL_DR, GENERAL_NARRATION } from './general-draft';

type PerRow = Record<string, Record<string, string>>;

/** The marked cells of the lines grid, by row; undefined when no cell carries a mark. */
export function journalMarkCells(
  marks: EntryMarks | undefined,
  rowIds: readonly string[],
  rows: readonly EntryRow[],
): PerRow | undefined {
  if (marks?.any !== true) return undefined;
  const out: PerRow = {};
  rowIds.forEach((rowId, line) => {
    const cells: Record<string, string> = {};
    const account = marks.of(`lines[${String(line)}].account_id`);
    if (account !== undefined) cells[GENERAL_ACCOUNT] = account.tip;
    const amount = marks.of(`lines[${String(line)}].amount_paisa`);
    if (amount !== undefined) {
      // The cell of the side the line is on now; a line that moved to the other side has the mark
      // there too, because its amount is typed there.
      const row = rows.find((candidate) => candidate.rowId === rowId);
      const onDebit = (row?.typed[GENERAL_DR] ?? '').trim() !== '';
      cells[onDebit ? GENERAL_DR : GENERAL_CR] = amount.tip;
    }
    const note = marks.of(`lines[${String(line)}].narration`);
    if (note !== undefined) cells[GENERAL_NARRATION] = note.tip;
    if (Object.keys(cells).length > 0) out[rowId] = cells;
  });
  return Object.keys(out).length === 0 ? undefined : out;
}
