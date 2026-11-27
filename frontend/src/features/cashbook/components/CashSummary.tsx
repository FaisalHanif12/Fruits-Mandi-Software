// What came in and went out by type over the days on screen (PRD FR-16.14 v1.60, plan 99): one row for each type
// that has an entry, with its count and its cash in and out, from the backend (`Cashbook.summary`, the entries in
// force only). It does not narrow with the type filter, so every type stays in view; a click on a type filters the
// book by it. It is shown from a button in the filter row, shut at first so the book stays in view, and the screen
// remembers how the user left it. Nothing is added up here (CLAUDE.md rule 1): the book's own totals row carries
// the totals of what it shows.

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { isZeroPaisa } from '../../../shared/format';
import { Button, LinkButton, Num, Table, type TableColumn } from '../../../shared/ui';
import { groupMessageId } from '../row-text';
import type { CashGroup, CashGroupSummary } from '../types';

interface CashSummaryToggleProps {
  open: boolean;
  onToggle: () => void;
  /** The id of the summary the button opens and shuts. */
  summaryId: string;
}

/** The button of the filter row that opens and shuts the summary by type. */
export function CashSummaryToggle({ open, onToggle, summaryId }: CashSummaryToggleProps) {
  return (
    <Button expanded={open} controls={summaryId} onClick={onToggle}>
      {open ? (
        <ChevronUp aria-hidden="true" className="size-icon-md" />
      ) : (
        <ChevronDown aria-hidden="true" className="size-icon-md" />
      )}
      <Localized id="cashbook-summary-title" />
    </Button>
  );
}

interface CashSummaryProps {
  id: string;
  summary: readonly CashGroupSummary[];
  /** A type was chosen in the table: the book shows only it. */
  onChoose: (group: CashGroup) => void;
}

export function CashSummary({ id, summary, onChoose }: CashSummaryProps) {
  const { l10n } = useLocalization();
  const rows = summary.filter((line) => line.count > 0);
  const columns: readonly TableColumn<CashGroupSummary>[] = [
    {
      id: 'group',
      header: <Localized id="cashbook-summary-col-type" />,
      cell: (line) => (
        <LinkButton
          onClick={() => {
            onChoose(line.group);
          }}
        >
          <Localized id={groupMessageId(line.group)} />
        </LinkButton>
      ),
    },
    {
      id: 'count',
      header: <Localized id="cashbook-summary-col-count" />,
      kind: 'number',
      size: 'qty',
      cell: (line) => <Num>{String(line.count)}</Num>,
    },
    {
      id: 'in',
      header: <Localized id="cashbook-col-in" />,
      kind: 'number',
      size: 'money',
      cell: (line) => (isZeroPaisa(line.in_paisa) ? null : <Num paisa={line.in_paisa} />),
    },
    {
      id: 'out',
      header: <Localized id="cashbook-col-out" />,
      kind: 'number',
      size: 'money',
      cell: (line) => (isZeroPaisa(line.out_paisa) ? null : <Num paisa={line.out_paisa} />),
    },
  ];
  return (
    <div id={id} className="shrink-0 rounded-3 border border-border bg-surface p-2">
      {rows.length === 0 ? (
        <Localized id="cashbook-empty">
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : (
        <Table
          label={l10n.getString('cashbook-summary-title')}
          columns={columns}
          rows={rows}
          rowKey={(line) => line.group}
          dense
        />
      )}
    </div>
  );
}
