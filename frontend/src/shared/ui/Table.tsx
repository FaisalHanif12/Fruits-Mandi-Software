// A small read-only table for lists inside a screen (a card of vouchers, the learned words, the
// result of a rule test). Grids where the user types are `shared/grid`; this is for showing.
//
// It keeps the rules of the design system in one place (Code Standards F17, F18, F19, NFR-USE-19,
// plan 39):
//   - a `number` column (a figure, a date, a code, a number of days) is written left to right and
//     CENTRED, in Urdu and in English; its heading and its total are centred the same way, so a
//     figure stands under the middle of its heading (R1);
//   - a `text` column's words and heading start at the start edge, right in Urdu and left in
//     English (R2);
//   - a `centre` column (a tick, a chip, an icon) and an `action` column (a button) are centred (R3);
//   - a heading is never cut and never broken inside a word, it may take two lines, and every
//     heading of the table has one height, in the middle of its row (R4);
//   - a column with a `size` has the width token of its kind, the same in every table (R5);
//   - a total row, when a column has one, is the backend's figure under its column (R6);
//   - one row height, the cells' content in the middle of the row (R7).
// Every cell is a `td` or `th` with the same padding, so the same column type has the same look
// on every screen (F12).

import type { ReactNode } from 'react';

export type TableColumnKind = 'text' | 'number' | 'centre' | 'action';

/** The kinds of column that have one width everywhere (plan 39 R5, `--col-*-w` in tokens.css). */
export type TableColumnSize = 'code' | 'money' | 'qty' | 'date' | 'datetime' | 'status';

export interface TableColumn<Row> {
  id: string;
  /** The column heading. For an `action` column it is read aloud and not shown. */
  header: ReactNode;
  kind?: TableColumnKind;
  /** The width of this kind of column, the same in every table. A text column takes the room left. */
  size?: TableColumnSize;
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

const CELL = 'px-3 py-2 align-middle';
const KIND_CLASS: Record<TableColumnKind, string> = {
  text: 'text-start',
  // direction: ltr + text-align: center is the middle of the column in both languages (F17).
  number: 'text-center',
  centre: 'text-center',
  action: 'text-center',
};

/**
 * The width classes of a size: the width of its kind, the same in every table (plan 39 R5). It is the
 * width the column has when there is room; a table with many figures in a narrow card shrinks its
 * columns together, like a grid that must fit (NFR-USE-15), so there is never a sideways scroll.
 */
const SIZE_CLASS: Record<TableColumnSize, string> = {
  code: 'w-col-code',
  money: 'w-col-money',
  qty: 'w-col-qty',
  date: 'w-col-date',
  datetime: 'w-col-datetime',
  status: 'w-col-status',
};

/** What the alignment check reads: which edge the column's words and figures stand on. */
const ALIGN_OF: Record<TableColumnKind, 'text' | 'num' | 'centre'> = {
  text: 'text',
  number: 'num',
  centre: 'centre',
  action: 'centre',
};

/** Direction is set once per column kind, never by the language of the screen (F37). */
function directionOf(kind: TableColumnKind): 'ltr' | undefined {
  return kind === 'number' ? 'ltr' : undefined;
}

function classOf(
  column: { kind?: TableColumnKind; size?: TableColumnSize },
  nameColumn: boolean,
): string {
  const kind = column.kind ?? 'text';
  return `${CELL} ${KIND_CLASS[kind]}${column.size === undefined ? '' : ` ${SIZE_CLASS[column.size]}`}${nameColumn ? ' min-w-col-name' : ''}`;
}

export function Table<Row>({ label, columns, rows, rowKey }: TableProps<Row>) {
  const hasTotal = columns.some((column) => column.total !== undefined);
  // The first column of words keeps room for a name; the figure columns give way (plan 39 R5).
  const nameAt = columns.findIndex((column) => (column.kind ?? 'text') === 'text');
  return (
    <div className="min-w-0 overflow-x-auto">
      <table aria-label={label} className="w-full border-collapse type-body">
        <thead className="bg-surface-sunken text-ink-muted">
          <tr>
            {columns.map((column, position) => {
              const kind = column.kind ?? 'text';
              return (
                <th
                  key={column.id}
                  scope="col"
                  dir={directionOf(kind)}
                  data-align={ALIGN_OF[kind]}
                  className={`${classOf(column, position === nameAt)} font-normal`}
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
            <tr key={rowKey(row)} className="border-t border-border">
              {columns.map((column, position) => {
                const kind = column.kind ?? 'text';
                return (
                  <td
                    key={column.id}
                    dir={directionOf(kind)}
                    data-align={ALIGN_OF[kind]}
                    className={classOf(column, position === nameAt)}
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
              {columns.map((column, position) => {
                const kind = column.kind ?? 'text';
                return (
                  <td
                    key={column.id}
                    dir={directionOf(kind)}
                    data-align={ALIGN_OF[kind]}
                    className={classOf(column, position === nameAt)}
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
