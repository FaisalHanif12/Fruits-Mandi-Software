// بیک اپ (route /backup; PRD Section 20, FR-20.8, NFR-BAK-1 to NFR-BAK-15): the state of the
// backups, "back up now", the list of copies, restore with its confirm box, test restore, the
// backup key and the settings. Backups happen by themselves (FR-20.1); this screen is for looking,
// for a copy before something risky, and for getting the data back.
//
// Four states: loading, an error with a retry, and the screen (the list has its own empty state).
// Nothing here is worked out on the screen: every time, size and total is the backend's.

import { Localized, useLocalization } from '@fluent/react';
import { DatabaseBackup, FlaskConical } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { Button, ErrorNotice, ErrorState, LoadingState, showToast } from '../../../shared/ui';
import { BackupListSection } from '../components/BackupListSection';
import { KeyPanel } from '../components/KeyPanel';
import { RestoreDialog } from '../components/RestoreDialog';
import { RunReportNotices } from '../components/RunReportNotices';
import { SettingsCard } from '../components/SettingsCard';
import { StatusCard } from '../components/StatusCard';
import { TestRestoreDialog } from '../components/TestRestoreDialog';
import { useBackupNow, useBackupStatus } from '../hooks';
import type { BackupRef, BackupRow, RunReport } from '../types';

export function BackupPage() {
  const { l10n } = useLocalization();
  const status = useBackupStatus();
  const now = useBackupNow();
  const flight = useSingleFlight();
  const [report, setReport] = useState<RunReport | null>(null);
  const [restoreRow, setRestoreRow] = useState<BackupRow | null>(null);
  // `undefined`: no test restore is open. `null`: test the newest copy.
  const [testTarget, setTestTarget] = useState<BackupRef | null | undefined>(undefined);

  if (status.isPending) return <LoadingState />;
  if (status.isError) {
    return (
      <ErrorState
        title={<Localized id="backup-load-failed" />}
        error={uiErrorOf(status.error)}
        onRetry={() => {
          void status.refetch();
        }}
      />
    );
  }
  const state = status.data;
  const busy = flight.busy || now.isPending || state.running;
  const nowError = now.error === null ? null : uiErrorOf(now.error);

  const backUpNow = () => {
    void flight.run(
      () =>
        now
          .mutateAsync({
            local: true,
            usb: state.usb_enabled,
            cloud: state.cloud_enabled,
          })
          .then((done) => {
            setReport(done);
            if (done.results.every((result) => result.ok)) {
              showToast('success', l10n.getString('backup-now-done'));
            }
          })
          // The refusal (a copy is already running) is shown on the screen.
          .catch(() => undefined),
      undefined,
    );
  };

  return (
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="backup-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id="backup-help">
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            onClick={() => {
              setTestTarget(null);
            }}
          >
            <FlaskConical aria-hidden="true" className="size-icon-md" />
            <Localized id="backup-test-now" />
          </Button>
          <Button variant="primary" busy={busy} onClick={backUpNow}>
            <DatabaseBackup aria-hidden="true" className="size-icon-md" />
            <Localized id="backup-now" />
          </Button>
        </div>
      </div>
      {nowError === null ? null : <ErrorNotice error={nowError} />}
      {report === null ? null : <RunReportNotices report={report} />}
      <KeyPanel status={state} />
      <StatusCard
        status={state}
        onTest={() => {
          setTestTarget(null);
        }}
      />
      <BackupListSection
        onRestore={setRestoreRow}
        onTest={(row) => {
          setTestTarget({ kind: row.kind, name: row.name });
        }}
        onBackupNow={backUpNow}
      />
      <SettingsCard />
      {restoreRow === null ? null : (
        <RestoreDialog
          key={`${restoreRow.kind}:${restoreRow.name}`}
          row={restoreRow}
          keySet={state.key_set}
          onClose={() => {
            setRestoreRow(null);
          }}
        />
      )}
      {testTarget === undefined ? null : (
        <TestRestoreDialog
          key={testTarget === null ? 'newest' : `${testTarget.kind}:${testTarget.name}`}
          backup={testTarget}
          onClose={() => {
            setTestTarget(undefined);
          }}
        />
      )}
    </div>
  );
}
