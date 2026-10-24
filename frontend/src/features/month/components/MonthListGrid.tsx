// The months from the first entry to this month, newest first (PRD FR-17.6): the month, where it
// stands, the day it was closed and how many times a close of it was reversed. Enter or a click on a
// row opens that month below. The list is read only; every word of the state is from Fluent.

import { useMemo } from 'react';
import type { MonthRow } from '../../../bindings/MonthRow';
import { isoToDisplayDate } from '../../../shared/format';
import { ListGrid, type ListColumn } from '../../../shared/grid';
import { monthName, stateMessageId } from '../text';

const COLUMNS: readonly ListColumn<MonthRow>[] = [
  {
    field: 'month',
    type: 'text',
    headerId: 'month-col-month',
    grow: true,
    value: (row, context) => monthName((id) => context.text(id), row.month),
  },
  {
    field: 'state',
    type: 'choice',
    headerId: 'month-col-state',
    value: (row, context) => context.text(stateMessageId(row.state)),
  },
  {
    field: 'closed_on',
    type: 'date',
    headerId: 'month-col-closed-on',
    value: (row) => (row.closed_on === null ? '' : isoToDisplayDate(row.closed_on)),
  },
  {
    field: 'reversed',
    type: 'count',
    headerId: 'month-col-reversed',
    from: 'normal',
    value: (row) => (row.reversed_count === 0 ? '' : String(row.reversed_count)),
  },
];

interface MonthListGridProps {
  rows: readonly MonthRow[];
  /** The month that is open below, `YYYY-MM`. */
  selected: string | null;
  onOpen: (month: string) => void;
}

export function MonthListGrid({ rows, selected, onOpen }: MonthListGridProps) {
  const columns = useMemo(() => COLUMNS, []);
  return (
    <ListGrid<MonthRow>
      columns={columns}
      rows={rows}
      getRowId={(row) => row.month}
      labelId="month-list-label"
      heightMode="auto"
      focusRowId={selected}
      onOpen={(row) => {
        onOpen(row.month);
      }}
    />
  );
}
