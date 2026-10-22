// The backup key on the backup screen (PRD FR-20.10, NFR-BAK-7). Nothing is drawn once the key is
// made and confirmed. With no key yet (the wizard step was left for later, or the first sign-in
// failed), the Owner makes it here and prints it. With a key that was never confirmed, he types it
// from his printed sheet; if the sheet is lost he can make a new one, as long as no USB or cloud
// copy is locked with the old key (the backend refuses otherwise, and says so).

import { Localized } from '@fluent/react';
import { KeyRound } from 'lucide-react';
import { useState } from 'react';
import { Button, Card, Notice } from '../../../shared/ui';
import { useBackupKeyStore } from '../key-store';
import type { BackupStatus } from '../types';
import { BackupKeyCard, KeyConfirmForm } from './BackupKeyCard';

interface KeyPanelProps {
  status: BackupStatus;
}

export function KeyPanel({ status }: KeyPanelProps) {
  const [making, setMaking] = useState(false);
  // A key being shown stays on screen even after the state says it is set, until the step is over.
  const showing = useBackupKeyStore((state) => state.key !== null);
  const needsKey = !status.key_set;
  const needsConfirm = status.key_set && !status.key_confirmed;
  if (!making && !showing && !needsKey && !needsConfirm) return null;

  const done = () => {
    setMaking(false);
  };

  return (
    <Card title={<Localized id="backup-key-title" />}>
      {making || showing ? (
        <BackupKeyCard onDone={done} />
      ) : needsKey ? (
        <>
          <Notice
            tone="warn"
            action={
              <Button
                variant="primary"
                onClick={() => {
                  setMaking(true);
                }}
              >
                <KeyRound aria-hidden="true" className="size-icon-md" />
                <Localized id="backup-key-make" />
              </Button>
            }
          >
            <Localized id="backup-key-missing" />
          </Notice>
        </>
      ) : (
        <>
          <Notice tone={status.key_confirm_due ? 'warn' : 'info'}>
            <Localized
              id={status.key_confirm_due ? 'backup-key-confirm-due' : 'backup-key-unconfirmed'}
            />
          </Notice>
          <div className="max-w-dialog">
            <KeyConfirmForm
              onConfirmed={() => {
                useBackupKeyStore.getState().clear();
              }}
            />
          </div>
          <div className="flex flex-col items-start gap-1">
            <Button
              onClick={() => {
                setMaking(true);
              }}
            >
              <Localized id="backup-key-lost" />
            </Button>
            <Localized id="backup-key-lost-help">
              <p className="type-body text-ink-muted" />
            </Localized>
          </div>
        </>
      )}
    </Card>
  );
}
