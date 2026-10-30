// The books of one backup next to the books now (PRD FR-20.9 before a restore, NFR-BAK-11 after a
// test restore). Every figure is the backend's, read from the copy itself; the screen only lines
// them up. Money is shown with `Num`, counts as plain numbers, the date as DD-MM-YYYY. Numbers are
// centred under their heading (NFR-USE-19, plan 39 R1).

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode } from 'react';
import { isoToDisplayDate } from '../../../shared/format';
import { Num, Table, type TableColumn } from '../../../shared/ui';
import type { BackupTotals } from '../types';

interface TotalsTableProps {
  /**
   * Before a restore the columns are "In this copy" and "Now"; after a test restore they are
   * "Restored test copy" and "Live data".
   */
  variant: 'restore' | 'test';
  copy: BackupTotals;
  now: BackupTotals;
}

/** The words of the table for each use, as Fluent message IDs. */
const WORDS = {
  restore: {
    label: 'backup-restore-totals-label',
    copy: 'backup-totals-in-backup',
    now: 'backup-totals-now',
  },
  test: {
    label: 'backup-test-totals-label',
    copy: 'backup-totals-restored',
    now: 'backup-totals-live',
  },
} as const;

interface Line {
  id: string;
  labelId: string;
  of: (totals: BackupTotals) => ReactNode;
}

const LINES: readonly Line[] = [
  { id: 'accounts', labelId: 'backup-totals-accounts', of: (t) => <Num>{String(t.accounts)}</Num> },
  {
    id: 'bikri',
    labelId: 'backup-totals-bikri',
    of: (t) => <Num>{String(t.bikri_entries)}</Num>,
  },
  { id: 'vouchers', labelId: 'backup-totals-vouchers', of: (t) => <Num>{String(t.vouchers)}</Num> },
  {
    id: 'journal',
    labelId: 'backup-totals-journal',
    of: (t) => <Num>{String(t.journal_entries)}</Num>,
  },
  {
    id: 'last',
    labelId: 'backup-totals-last-entry',
    of: (t) =>
      t.last_entry_date === null ? (
        <Localized id="backup-totals-none">
          <span />
        </Localized>
      ) : (
        <Num>{isoToDisplayDate(t.last_entry_date)}</Num>
      ),
  },
  { id: 'debit', labelId: 'backup-totals-debit', of: (t) => <Num paisa={t.debit_paisa} /> },
  { id: 'credit', labelId: 'backup-totals-credit', of: (t) => <Num paisa={t.credit_paisa} /> },
  { id: 'cash', labelId: 'backup-totals-cash', of: (t) => <Num paisa={t.cash_paisa} /> },
];

export function TotalsTable({ variant, copy, now }: TotalsTableProps) {
  const { l10n } = useLocalization();
  const words = WORDS[variant];
  const columns: readonly TableColumn<Line>[] = [
    {
      id: 'what',
      header: <Localized id="backup-totals-what" />,
      cell: (line) => <Localized id={line.labelId} />,
    },
    {
      id: 'copy',
      header: <Localized id={words.copy} />,
      kind: 'number',
      size: 'qty',
      cell: (line) => line.of(copy),
    },
    {
      id: 'now',
      header: <Localized id={words.now} />,
      kind: 'number',
      size: 'qty',
      cell: (line) => line.of(now),
    },
  ];
  return (
    <Table
      label={l10n.getString(words.label)}
      columns={columns}
      rows={LINES}
      rowKey={(l) => l.id}
    />
  );
}
