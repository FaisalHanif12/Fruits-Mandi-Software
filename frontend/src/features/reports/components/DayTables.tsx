// The two short tables under the lines of a day of the day total sheet R34 (PRD R34, v1.12): the
// buyers' (code, buyer, previous balance, today's total, paid today, new balance) and the sellers'
// (code, seller, previous balance, net sale today, paid today, new balance), each with a totals row.
// The backend sends the columns of each table (the "other entries" column only on a day that has any),
// every row and the totals row: the screen draws them with the same list grid as every report, and
// adds up nothing (CLAUDE.md rule 1, Code Standards F34). A table belongs to a day; when the page holds
// more than one day, the title says which.

import { Localized } from '@fluent/react';
import { useMemo } from 'react';
import type { ReportTable } from '../../../bindings/ReportTable';
import { ListGrid, type ListContext } from '../../../shared/grid';
import { isoToDisplayDate } from '../../../shared/format';
import { Num } from '../../../shared/ui';
import { listColumnOf, totalsFor, type ReportRowView } from '../columns';

/** The rows of these tables open nothing: they are a summary of the day. */
function noOpen(): void {
  return undefined;
}

/** The totals row of a short table: the word "Total" stands under the party, not the narrow code. */
function totalsOf(table: ReportTable, context: ListContext) {
  const made = totalsFor(
    table.columns,
    { labelId: table.totals_label_id, cells: table.totals },
    context,
  );
  const party = table.columns.find((column) => column.kind === 'party');
  const code = table.columns[0];
  if (party === undefined || code === undefined || party.key === code.key) return made;
  const label = made.values[code.key];
  if (label === undefined) return made;
  const values = Object.fromEntries(
    Object.entries(made.values).filter(([key]) => key !== code.key),
  );
  return { labelField: party.key, values: { ...values, [party.key]: label } };
}

function DayTable({ table, withDay }: { table: ReportTable; withDay: boolean }) {
  const columns = useMemo(
    () => table.columns.map((column, index) => listColumnOf<ReportRowView>(column, index)),
    [table],
  );
  const rows = useMemo<ReportRowView[]>(
    () => table.rows.map((row, index) => ({ key: `${table.key}-${String(index)}`, row })),
    [table],
  );
  return (
    <section className="flex flex-col gap-2">
      <h2 className="type-section">
        <Localized id={table.title_id} />
        {withDay ? (
          <>
            {' '}
            <Num>{isoToDisplayDate(table.day)}</Num>
          </>
        ) : null}
      </h2>
      <ListGrid<ReportRowView>
        columns={columns}
        rows={rows}
        getRowId={(view) => view.key}
        labelId={table.title_id}
        onOpen={noOpen}
        heightMode="auto"
        // The row with the people of the dead amount reads stronger and opens nothing.
        isGroup={(view) => view.row.kind !== 'normal'}
        totals={(context) => totalsOf(table, context)}
      />
    </section>
  );
}

export function DayTables({ tables }: { tables: readonly ReportTable[] }) {
  const withDay = new Set(tables.map((table) => table.day)).size > 1;
  return (
    <div className="flex shrink-0 flex-col gap-4">
      {tables.map((table) => (
        <DayTable key={`${table.day}-${table.key}`} table={table} withDay={withDay} />
      ))}
    </div>
  );
}
