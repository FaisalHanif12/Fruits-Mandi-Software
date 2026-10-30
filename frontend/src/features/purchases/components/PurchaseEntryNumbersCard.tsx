// What the purchase entry costs (PRD 12.10, FR-12.41), live beside the form: one row for each
// item line with what is paid now and the total cost, and the money of the whole entry under them.
// Every number is the backend's, shown with formatPaisa, and what is paid now has its ⓘ with the
// backend's working (FR-7.43, F42). Nothing is added up here (CLAUDE.md rule 1): the total is the
// preview's `paid_now_total`.
// Four states: nothing typed yet, the rows (a line that cannot be worked out yet says so in
// words), and the total.

import { Localized, useLocalization } from '@fluent/react';
import type { Paisa } from '../../../shared/format';
import { CalcInfo, Card, Num, Table, type TableColumn } from '../../../shared/ui';
import type { PurchaseNumbers } from '../types';

/** One filled line of the entry with the backend's numbers for it (null while it has a mistake). */
export interface EntryLineNumbers {
  /** The row id of the line in the grid. */
  key: string;
  /** What names the line: the item and the packing, in the app language. */
  name: string;
  numbers: PurchaseNumbers | null;
}

interface PurchaseEntryNumbersCardProps {
  lines: readonly EntryLineNumbers[];
  /** The money of the whole entry, from the backend. Null while nothing can be worked out. */
  total: Paisa | null;
  /** Nothing has been asked yet: the form is still empty. */
  empty: boolean;
}

export function PurchaseEntryNumbersCard({ lines, total, empty }: PurchaseEntryNumbersCardProps) {
  const { l10n } = useLocalization();
  const columns: readonly TableColumn<EntryLineNumbers>[] = [
    {
      id: 'line',
      header: <Localized id="purchases-entry-col-line" />,
      cell: (line) => line.name,
      ...(total === null ? {} : { total: <Localized id="purchases-entry-total" /> }),
    },
    {
      id: 'paid',
      header: <Localized id="purchases-line-paid-now" />,
      kind: 'number',
      size: 'money',
      ...(total === null ? {} : { total: <Num paisa={total} /> }),
      cell: (line) =>
        line.numbers === null ? (
          <Localized id="purchases-entry-line-incomplete">
            <span className="text-ink-muted" />
          </Localized>
        ) : (
          <span className="inline-flex items-center gap-1">
            <Num paisa={line.numbers.paid_now} />
            <CalcInfo working={line.numbers.workings.paid_now} inCell />
          </span>
        ),
    },
    {
      id: 'total',
      header: <Localized id="purchases-line-total-cost" />,
      kind: 'number',
      size: 'money',
      cell: (line) => (line.numbers === null ? '' : <Num paisa={line.numbers.total_cost} />),
    },
  ];
  return (
    <Card compact title={<Localized id="purchases-entry-title" />}>
      {lines.length === 0 ? (
        <Localized id={empty ? 'purchases-numbers-empty' : 'purchases-entry-no-lines'}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <Table
          label={l10n.getString('purchases-entry-title')}
          columns={columns}
          rows={lines}
          rowKey={(line) => line.key}
        />
      )}
    </Card>
  );
}
