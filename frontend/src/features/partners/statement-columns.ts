// The columns of a partner statement (PRD FR-17.9, FR-17.10, FR-19.11): the date, the heading, one
// sentence that says what it was (with its source number, the item or the reason), the signed
// amount and the running کل رقم. A note line has no amount and leaves the running total the same;
// its amount cell is empty and its sentence says so. Amounts are the backend's and shown with
// `money`, never worked out. Less used columns come with the wider layout steps, so a list never
// scrolls sideways (NFR-USE-15, Code Standards F22).

import { isoToDisplayDate, type Grouping } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import { detailText, headingText, type StatementSay } from './statement-text';
import type { StatementLine } from './types';

/**
 * A line with the place it has in the list. Two lines can look alike (two write-offs of one trip on
 * one day), so the grid's row ID is the place, which does not change while pages are added.
 */
export interface StatementRow {
  id: string;
  line: StatementLine;
}

export function statementColumns(
  grouping: Grouping,
  exists: (id: string) => boolean,
): readonly ListColumn<StatementRow>[] {
  const say = (context: { language: StatementSay['language']; text: StatementSay['text'] }) => ({
    ...context,
    grouping,
    exists,
  });
  return [
    {
      field: 'date',
      type: 'date',
      headerId: 'cashbook-col-date',
      value: (row) => isoToDisplayDate(row.line.entry_date),
    },
    {
      field: 'heading',
      type: 'text',
      headerId: 'partners-statement-col-heading',
      from: 'normal',
      value: (row, context) => headingText(row.line, say(context)),
    },
    {
      field: 'detail',
      type: 'name',
      headerId: 'partners-statement-col-detail',
      grow: true,
      value: (row, context) => detailText(row.line, say(context)),
    },
    {
      field: 'amount',
      type: 'money',
      headerId: 'cashbook-col-amount',
      value: (row, { money }) =>
        row.line.amount_paisa === null ? '' : money(row.line.amount_paisa),
    },
    {
      field: 'running',
      type: 'money',
      headerId: 'partners-statement-col-running',
      value: (row, { money }) => money(row.line.running_paisa),
    },
  ];
}
