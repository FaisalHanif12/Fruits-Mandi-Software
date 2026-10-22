// Test restore (PRD NFR-BAK-11, FR-20.8): the backend restores a copy into a separate place, runs
// its checks and compares the books of the copy with the books now. The live data is not touched.
// It starts as soon as the dialog opens. A copy that fails a check is a result ("not good"), not an
// error: the failing check is named. A copy locked with the key on a computer with no key asks for
// the printed key, and a wrong key is shown at its field.

import { Localized } from '@fluent/react';
import { CircleCheck, CircleX } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { utcToLocalDisplay } from '../../../shared/format';
import { SHORTCUTS, useFormNavigation } from '../../../shared/keyboard';
import {
  Button,
  Code,
  ErrorNotice,
  ErrorState,
  Field,
  FormDialog,
  LoadingState,
  Notice,
  TextInput,
  fieldError,
} from '../../../shared/ui';
import { useTestRestore } from '../hooks';
import type { BackupRef, TestRestoreResult } from '../types';
import { TotalsTable } from './TotalsTable';

const KEY_FIELDS = ['key'];

interface TestRestoreDialogProps {
  /** The copy to test, or null for the newest one (cloud first, then USB, then local). */
  backup: BackupRef | null;
  onClose: () => void;
}

export function TestRestoreDialog({ backup, onClose }: TestRestoreDialogProps) {
  const test = useTestRestore();
  const [typedKey, setTypedKey] = useState('');
  const started = useRef(false);
  const { mutate } = test;
  useEffect(() => {
    if (started.current) return;
    started.current = true;
    mutate({ backup, key: null });
  }, [backup, mutate]);

  const error = test.isError ? uiErrorOf(test.error) : null;
  const keyProblem = error?.errors.some((item) => item.field === 'key') ?? false;
  const again = () => {
    mutate({ backup, key: typedKey.trim() === '' ? null : typedKey.trim() });
  };
  const keyNavigation = useFormNavigation(again);

  let body;
  if (test.isPending || test.isIdle) {
    body = (
      <div className="flex flex-col gap-2">
        <Localized id="backup-test-running">
          <p className="type-body text-ink-muted" />
        </Localized>
        <LoadingState rows={2} />
      </div>
    );
  } else if (error !== null && keyProblem) {
    body = (
      <div {...keyNavigation} className="flex flex-col gap-3">
        <Localized id="backup-restore-key-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <ErrorNotice error={error} fieldsShown={KEY_FIELDS} />
        <Field
          label={<Localized id="backup-restore-key-label" />}
          hint={<Localized id="backup-restore-key-hint" />}
          error={fieldError(error, 'key')}
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
            onClick={again}
          >
            <Localized id="backup-test-again" />
          </Button>
        </div>
      </div>
    );
  } else if (error !== null) {
    body = (
      <ErrorState title={<Localized id="backup-test-failed" />} error={error} onRetry={again} />
    );
  } else if (test.data !== undefined) {
    body = <Result result={test.data} />;
  }

  return (
    <FormDialog
      open
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      title={<Localized id="backup-test-title" />}
      description={<Localized id="backup-test-help" />}
    >
      {body}
      <div className="sticky -bottom-6 -mb-6 mt-2 flex justify-end gap-2 bg-surface pb-6 pt-2">
        <Button variant="primary" shortcut={SHORTCUTS.back} onClick={onClose}>
          <Localized id="backup-test-close" />
        </Button>
      </div>
    </FormDialog>
  );
}

function Result({ result }: { result: TestRestoreResult }) {
  return (
    <div className="flex flex-col gap-3">
      <Notice tone={result.ok ? 'info' : 'error'}>
        <Localized id={result.ok ? 'backup-test-ok' : 'backup-test-not-ok'} />
      </Notice>
      <Localized
        id="backup-restore-made-on"
        vars={{ date: utcToLocalDisplay(result.created_at) }}
        elems={{ num: <Code /> }}
      >
        <p className="type-body" />
      </Localized>
      <ul className="flex flex-col gap-1">
        {result.checks.map((check) => (
          <li key={check.name} className="flex items-center gap-2 type-body">
            {check.ok ? (
              <CircleCheck aria-hidden="true" className="size-icon-md shrink-0 text-money-in" />
            ) : (
              <CircleX aria-hidden="true" className="size-icon-md shrink-0 text-due" />
            )}
            <Localized id={`backup-check-${check.name}`} />
            <Localized id={check.ok ? 'backup-check-passed' : 'backup-check-failed'}>
              <span className={check.ok ? 'text-money-in' : 'text-due'} />
            </Localized>
          </li>
        ))}
      </ul>
      <TotalsTable variant="test" copy={result.restored} now={result.live} />
    </div>
  );
}
