// The list of backups (PRD FR-20.8): date, place, size, whether it needs the key, and the result of
// its check, newest first, one page at a time (Code Standards B31). Enter (or a double click) on a row
// starts a restore; the buttons below do the same for the row that has focus, and test it. Four
// states: loading, an error with a retry, nothing yet (with a button), and the rows. A cloud that
// could not be reached is said in words, not hidden (the cloud copies are missing from the list).

import { Localized, useLocalization } from '@fluent/react';
import { DatabaseBackup, HardDriveDownload } from 'lucide-react';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, Pager, useListPaging } from '../../../shared/grid';
import {
  Button,
  Card,
  EmptyState,
  ErrorState,
  LoadingState,
  Notice,
  SegmentedControl,
  type SegmentedOption,
} from '../../../shared/ui';
import { BACKUP_COLUMNS, backupRowId } from '../list-columns';
import { useBackupList } from '../hooks';
import type { BackupKind, BackupRow } from '../types';

/** A list with more rows than this scrolls inside its region and draws only the rows in view (F34). */
const SHORT_LIST_ROWS = 30;

type KindChoice = BackupKind | 'all';
const CHOICES: readonly KindChoice[] = ['all', 'local', 'usb', 'cloud'];

interface BackupListSectionProps {
  /** A restore of this copy was asked (Enter, a double click, or the button). */
  onRestore: (row: BackupRow) => void;
  /** A test restore of this copy was asked. */
  onTest: (row: BackupRow) => void;
  /** Make a copy now, for the empty state's button. */
  onBackupNow: () => void;
}

export function BackupListSection({ onRestore, onTest, onBackupNow }: BackupListSectionProps) {
  const { l10n } = useLocalization();
  const [choice, setChoice] = useState<KindChoice>('all');
  const [focusId, setFocusId] = useState<string | null>(null);
  const list = useBackupList(choice === 'all' ? null : choice);
  const rows = useMemo(() => list.data?.pages.flatMap((page) => page.rows), [list.data]);
  const paging = useListPaging(list.pager);
  const first = list.data?.pages[0];
  const focused = rows?.find((row) => backupRowId(row) === focusId);

  const options: readonly SegmentedOption<KindChoice>[] = CHOICES.map((value) => ({
    value,
    label: <Localized id={`backup-filter-${value}`} />,
  }));

  let body;
  if (rows === undefined && list.isPending) {
    body = <LoadingState />;
  } else if (list.isError && rows === undefined) {
    body = (
      <ErrorState
        title={<Localized id="backup-list-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows?.length === 0) {
    body = (
      <EmptyState
        icon={DatabaseBackup}
        title={<Localized id="backup-list-empty" />}
        help={<Localized id="backup-list-empty-help" />}
        action={
          <Button variant="primary" onClick={onBackupNow}>
            <Localized id="backup-now" />
          </Button>
        }
      />
    );
  } else if (rows !== undefined) {
    const long = rows.length > SHORT_LIST_ROWS;
    body = (
      <>
        <div className={long ? 'h-list-region' : undefined}>
          <ListGrid<BackupRow>
            columns={BACKUP_COLUMNS}
            rows={rows}
            getRowId={backupRowId}
            labelId="backup-list-label"
            onOpen={onRestore}
            onFocusRow={setFocusId}
            focusRowId={focusId}
            heightMode={long ? 'fill' : 'auto'}
            isInactive={(row) => row.status === 'bad'}
            paging={paging}
          />
        </div>
        <Pager pager={list.pager} />
        <Localized id="backup-list-how">
          <p className="type-body text-ink-muted" />
        </Localized>
        <div className="flex flex-wrap gap-2">
          <Button
            variant="danger"
            disabled={focused === undefined}
            onClick={() => {
              if (focused !== undefined) onRestore(focused);
            }}
          >
            <HardDriveDownload aria-hidden="true" className="size-icon-md" />
            <Localized id="backup-restore-selected" />
          </Button>
          <Button
            disabled={focused === undefined}
            onClick={() => {
              if (focused !== undefined) onTest(focused);
            }}
          >
            <Localized id="backup-test-selected" />
          </Button>
        </div>
      </>
    );
  }

  return (
    <Card title={<Localized id="backup-list-title" />}>
      <SegmentedControl<KindChoice>
        options={options}
        value={choice}
        onChange={setChoice}
        groupLabel={l10n.getString('backup-filter-label')}
      />
      {first?.cloud_reachable === false ? (
        <Notice tone="warn">
          <Localized id="backup-list-cloud-unreachable" />
        </Notice>
      ) : null}
      {body}
    </Card>
  );
}
