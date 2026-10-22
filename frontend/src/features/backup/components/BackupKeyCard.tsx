// The backup key (PRD FR-20.10, NFR-BAK-7, NFR-INS-2): made once, shown once, printed, and typed
// back from the paper to prove the Owner has it. Used by the first-run wizard (its key step) and by
// the backup screen (no key yet, or a new one). The key comes from the backend and is never worked
// out here. While the step is open it lives in the key store, so a language switch loses nothing
// and the backend is asked for one key only. F9 prints.

import { Localized, useLocalization } from '@fluent/react';
import { useEffect, useRef } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { utcToLocalDisplay } from '../../../shared/format';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  Code,
  ErrorNotice,
  ErrorState,
  Field,
  LoadingState,
  Notice,
  TextInput,
  fieldError,
  showToast,
} from '../../../shared/ui';
import '../backup.css';
import { hasWireCode } from '../errors';
import { useConfirmBackupKey, useMakeBackupKey } from '../hooks';
import { useBackupKeyStore } from '../key-store';

const KEY_FIELDS = ['key'];

interface KeyConfirmFormProps {
  /** The key was typed right: the step is over. */
  onConfirmed: () => void;
  /** The Owner has not printed the key yet, so confirming waits (the wizard asks first). */
  locked?: boolean;
}

/**
 * The field where the Owner types the key from his printed sheet, and the button that checks it
 * (NFR-BAK-7). The backend compares it with the stored key and is the final word; a wrong key is
 * shown at the field and nothing else changes.
 */
export function KeyConfirmForm({ onConfirmed, locked = false }: KeyConfirmFormProps) {
  const { l10n } = useLocalization();
  const typed = useBackupKeyStore((state) => state.typed);
  const setTyped = useBackupKeyStore((state) => state.setTyped);
  const confirm = useConfirmBackupKey();
  const flight = useSingleFlight();
  const error = confirm.error === null ? null : uiErrorOf(confirm.error);
  const ready = !locked && typed.trim() !== '';

  const check = () => {
    if (!ready) return;
    void flight.run(
      () =>
        confirm
          .mutateAsync(typed.trim())
          .then(() => {
            showToast('success', l10n.getString('backup-key-confirmed'));
            onConfirmed();
          })
          // The refusal is shown at the field.
          .catch(() => undefined),
      undefined,
    );
  };
  const navigation = useFormNavigation(check);

  return (
    <div {...navigation} className="flex flex-col gap-3">
      {error === null ? null : <ErrorNotice error={error} fieldsShown={KEY_FIELDS} />}
      <Field
        label={<Localized id="backup-key-type-label" />}
        hint={<Localized id="backup-key-type-hint" />}
        error={fieldError(error, 'key')}
      >
        {(control) => <TextInput {...control} lang="en" value={typed} onChange={setTyped} />}
      </Field>
      <div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.confirm}
          disabled={!ready}
          busy={flight.busy || confirm.isPending}
          onClick={check}
        >
          <Localized id="backup-key-confirm" />
        </Button>
      </div>
    </div>
  );
}

interface BackupKeyCardProps {
  /** The step is over: the key is confirmed, left for later, or there was one already. */
  onDone: () => void;
}

export function BackupKeyCard({ onDone }: BackupKeyCardProps) {
  const key = useBackupKeyStore((state) => state.key);
  const createdAt = useBackupKeyStore((state) => state.createdAt);
  const printed = useBackupKeyStore((state) => state.printed);
  const setPrinted = useBackupKeyStore((state) => state.setPrinted);
  const make = useMakeBackupKey();
  // The backend is asked once, even when the step is drawn again.
  const asked = useRef(false);
  const { mutate } = make;
  useEffect(() => {
    if (key !== null || asked.current) return;
    asked.current = true;
    mutate(undefined, {
      onSuccess: (made) => {
        useBackupKeyStore.getState().set(made.key, made.created_at);
      },
    });
  }, [key, mutate]);

  const print = () => {
    if (key !== null) window.print();
  };
  useShortcut(SHORTCUTS.print, print);

  const finish = () => {
    useBackupKeyStore.getState().clear();
    onDone();
  };

  if (key === null) {
    if (make.isError) {
      const error = uiErrorOf(make.error);
      // A key is in use already (it locks copies, or the Owner confirmed it): it is not shown again.
      if (hasWireCode(error, 'ERR_BACKUP_KEY_ALREADY_SET')) {
        return (
          <div className="flex flex-col gap-3">
            <Notice tone="info">
              <Localized id="backup-key-already" />
            </Notice>
            <div>
              <Button variant="primary" onClick={onDone}>
                <Localized id="backup-key-continue" />
              </Button>
            </div>
          </div>
        );
      }
      return (
        <ErrorState
          title={<Localized id="backup-key-failed" />}
          error={error}
          onRetry={() => {
            asked.current = true;
            make.mutate(undefined, {
              onSuccess: (made) => {
                useBackupKeyStore.getState().set(made.key, made.created_at);
              },
            });
          }}
        />
      );
    }
    return <LoadingState rows={2} />;
  }

  return (
    <div className="flex flex-col gap-4">
      <Localized id="backup-key-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <p
        aria-label={key}
        className="type-title rounded-2 border border-border bg-surface-sunken px-4 py-3 text-center tracking-code"
      >
        <Code>{key}</Code>
      </p>
      {createdAt === null ? null : (
        <Localized
          id="backup-key-made-on"
          vars={{ date: utcToLocalDisplay(createdAt) }}
          elems={{ num: <Code /> }}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
      <Checkbox
        checked={printed}
        onChange={setPrinted}
        label={<Localized id="backup-key-printed" />}
      />
      <div>
        <Button shortcut={SHORTCUTS.print} onClick={print}>
          <Localized id="backup-key-print" />
        </Button>
      </div>
      <KeyConfirmForm locked={!printed} onConfirmed={finish} />
      <div className="flex flex-col items-start gap-1">
        <Button onClick={finish}>
          <Localized id="backup-key-later" />
        </Button>
        <Localized id="backup-key-later-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {/* Drawn on paper only (backup.css): the sheet that is printed and kept in a safe place. */}
      <div data-key-sheet="" className="hidden print:block">
        <Localized id="backup-key-sheet-title">
          <h1 className="type-title" />
        </Localized>
        <p className="type-title tracking-code">
          <Code>{key}</Code>
        </p>
        {createdAt === null ? null : (
          <Localized
            id="backup-key-made-on"
            vars={{ date: utcToLocalDisplay(createdAt) }}
            elems={{ num: <Code /> }}
          >
            <p className="type-body" />
          </Localized>
        )}
        <Localized id="backup-key-sheet-note">
          <p className="type-body" />
        </Localized>
      </div>
    </div>
  );
}
