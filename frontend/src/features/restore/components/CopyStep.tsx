// Step 2 of the restore: choose the copy and type the backup key (PRD FR-20.11). The copies found
// are listed newest first, with the newest already chosen. The key is typed from the printed sheet
// and is masked as it is typed (NFR-SEC-7). A wrong key is shown at its field and he can try again;
// the copy is opened and checked only when he presses the button, so a mistyped key costs nothing.
// Four states: nothing found (a button goes back to change the place), the copies, and, while the
// copy is being checked, a busy button. A refusal is shown at the field it concerns (PRD FR-7.6).

import { Localized } from '@fluent/react';
import { DatabaseBackup, ShieldCheck } from 'lucide-react';
import { useMemo } from 'react';
import {
  ListGrid,
  ListRegion,
  isLongList,
  listHeightMode,
  type ListColumn,
} from '../../../shared/grid';
import { utcToLocalDisplay } from '../../../shared/format';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  Card,
  Code,
  EmptyState,
  ErrorNotice,
  Field,
  PasswordInput,
  fieldError,
} from '../../../shared/ui';
import { sizeText } from '../../backup';
import { KEY_PATH, KEY_PATHS } from '../field-paths';
import { checkCopy } from '../flow';
import { STEP_WHERE, useRestoreStore } from '../store';
import type { BackupRow } from '../types';
import { ProgressBar } from './ProgressBar';

export function CopyStep() {
  const copies = useRestoreStore((state) => state.copies);
  const copyName = useRestoreStore((state) => state.copyName);
  const backupKey = useRestoreStore((state) => state.backupKey);
  const error = useRestoreStore((state) => state.error);
  const busy = useRestoreStore((state) => state.busy);
  const progress = useRestoreStore((state) => state.progress);
  const chooseCopy = useRestoreStore((state) => state.chooseCopy);
  const setBackupKey = useRestoreStore((state) => state.setBackupKey);
  const goTo = useRestoreStore((state) => state.goTo);
  // The key field has focus as soon as the copies are listed: the newest is already chosen.
  const keyBox = useFocusFirstField<HTMLDivElement>(copies?.length);

  const newest = copies?.[0]?.name;
  const columns = useMemo<readonly ListColumn<BackupRow>[]>(
    () => [
      {
        field: 'created',
        type: 'datetime',
        headerId: 'backup-col-created',
        value: (row) => utcToLocalDisplay(row.created_at),
      },
      {
        field: 'size',
        type: 'number',
        headerId: 'backup-col-size',
        value: (row, { text }) => sizeText(row.size_bytes, text),
      },
      {
        field: 'newest',
        type: 'text',
        grow: true,
        headerId: 'restore-col-newest',
        value: (row, { text }) => (row.name === newest ? text('restore-newest') : ''),
      },
    ],
    [newest],
  );

  const check = () => {
    if (busy || backupKey.trim() === '') return;
    void checkCopy();
  };
  const navigation = useFormNavigation(check);
  const focusKey = () => {
    keyBox.current?.querySelector<HTMLElement>('input')?.focus();
  };

  if (copies === null || copies.length === 0) {
    return (
      <Card>
        <EmptyState
          icon={DatabaseBackup}
          title={<Localized id="restore-copy-empty" />}
          help={<Localized id="restore-copy-empty-help" />}
          action={
            <Button
              variant="primary"
              shortcut={SHORTCUTS.confirm}
              onClick={() => {
                goTo(STEP_WHERE);
              }}
            >
              <Localized id="restore-copy-change-place" />
            </Button>
          }
        />
      </Card>
    );
  }

  const chosen = copies.find((row) => row.name === copyName);
  return (
    <Card>
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <Localized id="restore-copy-title">
            <h2 className="type-section" />
          </Localized>
          <Localized
            id="restore-copy-help"
            vars={{ count: copies.length }}
            elems={{ num: <Code /> }}
          >
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        {/* A long list scrolls inside a box of its own, so the key field stays close (plan 38 G2). */}
        <div className={isLongList(copies.length) ? 'flex h-list-region flex-col' : undefined}>
          <ListRegion rowCount={copies.length}>
            <ListGrid<BackupRow>
              columns={columns}
              rows={copies}
              getRowId={(row) => row.name}
              labelId="restore-copy-list-label"
              onOpen={(row) => {
                chooseCopy(row.name);
                focusKey();
              }}
              onFocusRow={chooseCopy}
              heightMode={listHeightMode(copies.length)}
            />
          </ListRegion>
        </div>
        {chosen === undefined ? null : (
          <Localized
            id="restore-copy-chosen"
            vars={{ date: utcToLocalDisplay(chosen.created_at) }}
            elems={{ num: <Code /> }}
          >
            <p role="status" className="type-body" />
          </Localized>
        )}
        <div ref={keyBox} {...navigation} className="flex flex-col gap-4">
          {error === null ? null : <ErrorNotice error={error} fieldsShown={KEY_PATHS} />}
          <Field
            label={<Localized id="backup-restore-key-label" />}
            hint={<Localized id="restore-key-hint" />}
            error={fieldError(error, KEY_PATH)}
          >
            {(control) => (
              <PasswordInput
                {...control}
                value={backupKey}
                onChange={setBackupKey}
                autoComplete="new-password"
              />
            )}
          </Field>
          {busy ? (
            <div role="status" className="flex flex-col gap-2">
              <Localized id="restore-copy-checking">
                <p className="type-body text-ink-muted" />
              </Localized>
              {progress === null ? null : (
                <>
                  <Localized id={`restore-stage-${progress.stage.replaceAll('_', '-')}`}>
                    <p className="type-body" />
                  </Localized>
                  {progress.percent === null ? null : <ProgressBar percent={progress.percent} />}
                </>
              )}
            </div>
          ) : null}
          <div className="flex flex-wrap items-center justify-between gap-2">
            <Button
              shortcut={SHORTCUTS.back}
              onClick={() => {
                goTo(STEP_WHERE);
              }}
            >
              <Localized id="common-back" />
            </Button>
            <Button
              variant="primary"
              shortcut={SHORTCUTS.confirm}
              busy={busy}
              disabled={backupKey.trim() === ''}
              onClick={check}
            >
              <ShieldCheck aria-hidden="true" className="size-icon-md" />
              <Localized id="restore-copy-check" />
            </Button>
          </div>
        </div>
      </div>
    </Card>
  );
}
