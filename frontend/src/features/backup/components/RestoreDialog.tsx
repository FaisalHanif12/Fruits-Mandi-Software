// Restore a backup (PRD FR-20.9, NFR-REL-4): pick a copy, see its date and what is in it next to the
// books now, see what the restore would lose, then confirm. The confirm box says exactly what will
// happen (FR-7.8) and starts on "Back", so a stray Enter never replaces the books. The numbers are
// read from the copy by the backend; the screen works out nothing. A copy made on the USB drive or
// in the cloud needs the printed key when this computer has none (a new computer): the key is asked
// first, and a wrong key is shown at its field with nothing changed.
//
// After the restore the Owner is signed out by the backend, and every cached answer is read again,
// so the sign-in screen opens (plan 18 contract).

import { Localized, useLocalization } from '@fluent/react';
import { HardDriveDownload } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { utcToLocalDisplay } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  Code,
  ConfirmDialog,
  ErrorNotice,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  Notice,
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import { useRestore, useRestorePreview } from '../hooks';
import type { BackupRef, BackupRow, RestorePreview } from '../types';
import { TotalsTable } from './TotalsTable';

const KEY_FIELDS = ['key'];

interface RestoreDialogProps {
  /** The copy to restore. The screen draws this dialog only while a copy is chosen. */
  row: BackupRow;
  /** True when this computer has the backup key (a fresh computer has none). */
  keySet: boolean;
  onClose: () => void;
}

export function RestoreDialog({ row, keySet, onClose }: RestoreDialogProps) {
  const { l10n } = useLocalization();
  const ref: BackupRef = { kind: row.kind, name: row.name };
  // A copy that is locked with the key, on a computer that has no key, asks for the key first.
  const asksForKey = row.encrypted && !keySet;
  const [typedKey, setTypedKey] = useState('');
  const [sentKey, setSentKey] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const preview = useRestorePreview(!asksForKey || sentKey !== null ? ref : null, sentKey);
  const restore = useRestore();
  const flight = useSingleFlight();
  const previewError = preview.isError ? uiErrorOf(preview.error) : null;
  const restoreError = restore.error === null ? null : uiErrorOf(restore.error);
  // The backend can also ask for the key (a copy it finds locked on a computer it thought had one).
  const keyCodes = [...(previewError?.errors ?? []), ...(restoreError?.errors ?? [])];
  const keyProblem = keyCodes.some((item) => item.field === 'key');
  const showKeyField = asksForKey || keyProblem;
  const data: RestorePreview | undefined = preview.data;
  const waitingForKey = asksForKey && sentKey === null;

  const sendKey = () => {
    if (typedKey.trim() !== '') setSentKey(typedKey.trim());
  };
  const keyNavigation = useFormNavigation(sendKey);

  const confirm = () => {
    setConfirming(false);
    void flight.run(
      () =>
        restore
          .mutateAsync({ backup: ref, key: sentKey, confirmed: true })
          .then((done) => {
            showToast(
              'success',
              l10n.getString('backup-restored', { date: utcToLocalDisplay(done.restored_from) }),
            );
            onClose();
          })
          // The refusal is shown in the dialog. Nothing was changed.
          .catch(() => undefined),
      undefined,
    );
  };

  let body;
  if (waitingForKey || (keyProblem && previewError !== null)) {
    body = (
      <div {...keyNavigation} className="flex flex-col gap-3">
        <Localized id="backup-restore-key-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        {previewError === null ? null : (
          <ErrorNotice error={previewError} fieldsShown={KEY_FIELDS} />
        )}
        <Field
          label={<Localized id="backup-restore-key-label" />}
          hint={<Localized id="backup-restore-key-hint" />}
          error={fieldError(previewError, 'key')}
        >
          {(control) => (
            <TextInput {...control} lang="en" value={typedKey} onChange={setTypedKey} />
          )}
        </Field>
        <div>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.confirm}
            disabled={typedKey.trim() === ''}
            onClick={sendKey}
          >
            <Localized id="backup-restore-key-continue" />
          </Button>
        </div>
      </div>
    );
  } else if (preview.isPending) {
    body = <LoadingState rows={3} />;
  } else if (previewError !== null) {
    body = (
      <ErrorState
        title={<Localized id="backup-restore-preview-failed" />}
        error={previewError}
        onRetry={() => {
          void preview.refetch();
        }}
      />
    );
  } else if (data !== undefined) {
    const lost = data.lost;
    const nothingLost =
      lost.journal_entries === 0 && lost.bikri_entries === 0 && lost.vouchers === 0;
    body = (
      <div className="flex flex-col gap-3">
        <Localized
          id="backup-restore-made-on"
          vars={{ date: utcToLocalDisplay(data.created_at) }}
          elems={{ num: <Code /> }}
        >
          <p className="type-body" />
        </Localized>
        <TotalsTable variant="restore" copy={data.totals} now={data.current} />
        {nothingLost ? (
          <Notice tone="info">
            <Localized id="backup-restore-lost-none" />
          </Notice>
        ) : (
          <Notice tone="warn">
            <Localized
              id="backup-restore-lost"
              vars={{
                journal: String(lost.journal_entries),
                bikri: String(lost.bikri_entries),
                vouchers: String(lost.vouchers),
              }}
              elems={{ num: <Code /> }}
            >
              <span />
            </Localized>
          </Notice>
        )}
        {data.needs_upgrade ? (
          <Notice tone="info">
            <Localized id="backup-restore-upgrade" />
          </Notice>
        ) : null}
        <Localized id="backup-restore-safety">
          <p className="type-body text-ink-muted" />
        </Localized>
        {restoreError === null ? null : <ErrorNotice error={restoreError} fieldsShown={[]} />}
      </div>
    );
  }

  // The box says exactly what will happen: the date, and what the restore loses (FR-7.8, FR-20.9).
  const lostNow = data?.lost;
  const restoreWords = (
    <Localized
      id="backup-restore-confirm-description"
      vars={{
        date: data === undefined ? '' : utcToLocalDisplay(data.created_at),
        // Only for the choice in the message: nothing lost, or something lost.
        lost:
          lostNow === undefined
            ? 0
            : lostNow.journal_entries + lostNow.bikri_entries + lostNow.vouchers,
        journal: String(lostNow?.journal_entries ?? 0),
        bikri: String(lostNow?.bikri_entries ?? 0),
        vouchers: String(lostNow?.vouchers ?? 0),
      }}
      elems={{ num: <Code /> }}
    >
      <span />
    </Localized>
  );

  return (
    <FormDialog
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      title={<Localized id="backup-restore-title" />}
      description={<Localized id="backup-restore-help" />}
    >
      {body}
      <div className="sticky -bottom-6 -mb-6 mt-2 flex flex-wrap justify-end gap-2 bg-surface pb-6 pt-2">
        <Button shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="common-back" />
        </Button>
        {showKeyField && data === undefined ? null : (
          <Button
            variant="danger"
            disabled={data === undefined}
            busy={flight.busy || restore.isPending}
            onClick={() => {
              setConfirming(true);
            }}
          >
            <HardDriveDownload aria-hidden="true" className="size-icon-md" />
            <Localized id="backup-restore-button" />
          </Button>
        )}
      </div>
      <ConfirmDialog
        open={confirming}
        onOpenChange={setConfirming}
        title={<Localized id="backup-restore-confirm-title" />}
        description={restoreWords}
        confirmLabel={<Localized id="backup-restore-button" />}
        tone="danger"
        onConfirm={confirm}
      />
    </FormDialog>
  );
}
