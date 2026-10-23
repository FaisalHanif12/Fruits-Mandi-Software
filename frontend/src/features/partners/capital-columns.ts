// The columns of a partner's capital history (PRD 17.4, FR-19.11): date, what it was, its number,
// the note, the signed amount and the running capital. Amounts are the backend's and shown with
// `money`, never worked out. Less used columns come with the wider layout steps, so a list never
// scrolls sideways (NFR-USE-15, Code Standards F22).

import { isoToDisplayDate } from '../../shared/format';
import type { ListColumn } from '../../shared/grid';
import type { CapitalRow } from './types';

/** The Fluent message ID of the number series of a row, or null when the row has no number. */
function numberId(row: CapitalRow): string | null {
  if (row.number === null) return null;
  return row.kind === 'invested'
    ? 'vouchers-number-capital-in'
    : row.kind === 'taken_back'
      ? 'vouchers-number-capital-out'
      : null;
}

export const CAPITAL_COLUMNS: readonly ListColumn<CapitalRow>[] = [
  {
    field: 'date',
    type: 'date',
    headerId: 'cashbook-col-date',
    value: (row) => isoToDisplayDate(row.entry_date),
  },
  {
    field: 'kind',
    type: 'text',
    headerId: 'cashbook-col-kind',
    grow: true,
    value: (row, { text }) => text(`partners-capital-kind-${row.kind.replaceAll('_', '-')}`),
  },
  {
    field: 'number',
    type: 'code',
    headerId: 'cashbook-col-number',
    from: 'normal',
    value: (row, { text }) => {
      const id = numberId(row);
      return id === null || row.number === null ? '' : text(id, { number: String(row.number) });
    },
  },
  {
    field: 'narration',
    type: 'name',
    headerId: 'cashbook-col-narration',
    from: 'wide',
    grow: true,
    // A narration is free text, shown as typed (PRD FR-7.25).
    value: (row) => row.narration,
  },
  {
    field: 'amount',
    type: 'money',
    headerId: 'cashbook-col-amount',
    value: (row, { money }) => money(row.amount_paisa),
  },
  {
    field: 'running',
    type: 'money',
    headerId: 'partners-capital-col-balance',
    value: (row, { money }) => money(row.running_paisa),
  },
];
