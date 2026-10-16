// A small read-only table for lists inside a screen (a card of vouchers, the learned words, the
// result of a rule test). Grids where the user types are `shared/grid`; this is for showing.
//
// It keeps the rules of the design system in one place (Code Standards F17, F19, NFR-USE-19):
//   - a `number` column is written left to right and set against the physical right edge, in
//     Urdu and in English, so units line up under units;
//   - the header of a number column and its total sit on the same edge, under and over the figures;
//   - a total row, when a column has one, is the backend's figure under its column.
// Every cell is a `td` or `th` with the same padding, so the same column type has the same look
// on every screen (F12).

import type { ReactNode } from 'react';

export type TableColumnKind = 'text' | 'number' | 'action';

export interface TableColumn<Row> {
  id: string;
  /** The column heading. For an `action` column it is read aloud and not shown. */
  header: ReactNode;
  kind?: TableColumnKind;
  cell: (row: Row) => ReactNode;
  /** The backend's total for this column. A column with a total puts it in the last row. */
  total?: ReactNode;
}

interface TableProps<Row> {
  /** The name of the table for a screen reader. */
  label: string;
  columns: readonly TableColumn<Row>[];
  rows: readonly Row[];
  rowKey: (row: Row) => string;
}

const CELL = 'px-3 py-2';
const KIND_CLASS: Record<TableColumnKind, string> = {
  text: 'text-start',
  // direction: ltr + text-align: end is the physical right edge in both languages (F17).
  number: 'text-end',
  action: 'text-end',
};

/** Direction is set once per column kind, never by the language of the screen (F37). */
function directionOf(kind: TableColumnKind): 'ltr' | undefined {
  return kind === 'text' ? undefined : 'ltr';
}

export function Table<Row>({ label, columns, rows, rowKey }: TableProps<Row>) {
  const hasTotal = columns.some((column) => column.total !== undefined);
  return (
    <div className="min-w-0 overflow-x-auto">
      <table aria-label={label} className="w-full border-collapse type-body">
        <thead className="bg-surface-sunken text-ink-muted">
          <tr>
            {columns.map((column) => {
              const kind = column.kind ?? 'text';
              return (
                <th
                  key={column.id}
                  scope="col"
                  dir={directionOf(kind)}
                  className={`${CELL} ${KIND_CLASS[kind]} font-normal`}
                >
                  {kind === 'action' ? (
                    <span className="sr-only">{column.header}</span>
                  ) : (
                    column.header
                  )}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={rowKey(row)} className="border-t border-border align-top">
              {columns.map((column) => {
                const kind = column.kind ?? 'text';
                return (
                  <td
                    key={column.id}
                    dir={directionOf(kind)}
                    className={`${CELL} ${KIND_CLASS[kind]}`}
                  >
                    {column.cell(row)}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
        {hasTotal ? (
          <tfoot className="bg-surface-sunken type-label">
            <tr className="border-t border-border">
              {columns.map((column) => {
                const kind = column.kind ?? 'text';
                return (
                  <td
                    key={column.id}
                    dir={directionOf(kind)}
                    className={`${CELL} ${KIND_CLASS[kind]}`}
                  >
                    {column.total}
                  </td>
                );
              })}
            </tr>
          </tfoot>
        ) : null}
      </table>
    </div>
  );
}
