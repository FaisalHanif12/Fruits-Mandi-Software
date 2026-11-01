// A table of an answer (PRD FR-7.74, FR-19.11): up to 7 rows of the report or view the model read,
// with that source's own columns and its totals row over the whole filter. The cells are written by
// the report screens' own cell renderer (`cellText`), never by the model. A row that opens a record
// has a button on its first cell, the same record the report screen's row opens (FR-19.4). A column
// with nothing in it in any shown row is left out, so a table of 7 columns fits the panel.

import { useLocalization } from '@fluent/react';
import type { AnswerTable } from '../../../bindings/AnswerTable';
import type { ReportCell } from '../../../bindings/ReportCell';
import type { ReportColumnKind } from '../../../bindings/ReportColumnKind';
import type { ListContext } from '../../../shared/grid';
import { Num, Table, type TableColumn } from '../../../shared/ui';
import { useListContext } from '../../analytics';
import { cellText, sideText, useOpenRowSource } from '../../reports';
import type { AnswerRow } from '../../../bindings/AnswerRow';

/** A figure, a date, a count: centred and left to right. Everything else starts at the start edge. */
function isFigureKind(kind: ReportColumnKind): boolean {
  return (
    kind === 'money' ||
    kind === 'balance' ||
    kind === 'number' ||
    kind === 'percent' ||
    kind === 'date' ||
    kind === 'weight'
  );
}

/** The column width of a kind, the same as in every table. */
function sizeOf(kind: ReportColumnKind): TableColumn<unknown>['size'] {
  switch (kind) {
    case 'money':
    case 'balance':
      return 'money';
    case 'date':
      return 'date';
    case 'number':
    case 'percent':
    case 'weight':
      return 'qty';
    default:
      return undefined;
  }
}

function isEmpty(cell: ReportCell | undefined): boolean {
  return cell === undefined || cell.type === 'empty';
}

/** One cell written as the report writes it; a figure is in `<Num>` and a balance has its side word. */
function CellView({
  cell,
  kind,
  context,
}: {
  cell: ReportCell | undefined;
  kind: ReportColumnKind;
  context: ListContext;
}) {
  if (cell === undefined || cell.type === 'empty') return null;
  const text = cellText(cell, context);
  if (isFigureKind(kind)) {
    const side = sideText(cell, context);
    return (
      <>
        <Num>{text}</Num>
        {side === '' ? null : <span> {side}</span>}
      </>
    );
  }
  return <bdi>{text}</bdi>;
}

interface RowView {
  key: string;
  row: AnswerRow;
}

export function AnswerTableView({ table }: { table: AnswerTable }) {
  const { l10n } = useLocalization();
  const context = useListContext();
  const open = useOpenRowSource();
  const rows: RowView[] = table.rows.map((row, index) => ({ key: String(index), row }));
  const title = context.text(table.title_id);
  const totalsLabel = context.text(table.totals_label_id);
  // Only the columns that have something to show in the rows or the totals.
  const kept = table.columns
    .map((column, index) => ({ column, index }))
    .filter(
      ({ index }) =>
        table.rows.some((row) => !isEmpty(row.row.cells[index])) || !isEmpty(table.totals[index]),
    );
  const hasTotals = table.totals.some((cell) => !isEmpty(cell));
  const columns: TableColumn<RowView>[] = kept.map(({ column, index }, place) => {
    const size = sizeOf(column.kind);
    const total = table.totals[index];
    return {
      id: column.key,
      header: context.text(column.title_id),
      kind: isFigureKind(column.kind) ? 'number' : 'text',
      ...(size === undefined ? {} : { size }),
      cell: (view) => {
        const content = (
          <CellView cell={view.row.row.cells[index]} kind={column.kind} context={context} />
        );
        const source = view.row.row.source;
        // The first column opens the record behind the row.
        if (place !== 0 || source === null) return content;
        return (
          <button
            type="button"
            onClick={() => {
              open(source);
            }}
            className="rounded-1 text-start text-primary underline"
          >
            {content}
          </button>
        );
      },
      ...(hasTotals
        ? {
            total:
              // The first column that has no total of its own carries the label of the totals row.
              place === 0 && isEmpty(total) ? (
                <span>{totalsLabel}</span>
              ) : (
                <CellView cell={total} kind={column.kind} context={context} />
              ),
          }
        : {}),
    };
  });
  return (
    <div className="flex flex-col gap-1">
      <p className="type-label text-ink">{title}</p>
      <Table
        label={l10n.getString('assistant-table-label', { source: title })}
        columns={columns}
        rows={rows}
        rowKey={(view) => view.key}
      />
      {table.count > rows.length ? (
        <p className="type-body text-ink-muted">
          {l10n.getString('assistant-table-more', {
            shown: String(rows.length),
            count: String(table.count),
          })}
        </p>
      ) : null}
    </div>
  );
}
