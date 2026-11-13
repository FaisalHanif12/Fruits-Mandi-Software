// The reminders right after the first sign-in on a restored computer (plan 56 Section 3, step 6):
// make a new backup now, so this computer has its own first copy, and type the OpenRouter key again
// if the assistant was on, because that key never travels in a copy. It is shown once, over the app,
// until the Owner closes it. The backup runs through the normal backup command; this card only asks.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { CircleCheck, DatabaseBackup, Settings } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { SHORTCUTS, useEscape } from '../../../shared/keyboard';
import { Button, Card, ErrorNotice, showToast } from '../../../shared/ui';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { useBackupNow } from '../../backup';
import { useAfterRestoreStore } from '../after-restore-store';

export function AfterRestoreCard() {
  const { l10n } = useLocalization();
  const pending = useAfterRestoreStore((state) => state.pending);
  const clear = useAfterRestoreStore((state) => state.clear);
  const navigate = useNavigate();
  const now = useBackupNow();
  const flight = useSingleFlight();
  const [error, setError] = useState<UiError | null>(null);
  const [madeBackup, setMadeBackup] = useState(false);
  useEscape('leave', clear, pending !== null);
  if (pending === null) return null;

  const backUp = () => {
    setError(null);
    void flight.run(
      () =>
        now.mutateAsync({ local: true, usb: false, cloud: true }).then(
          (report) => {
            if (report.results.every((result) => result.ok)) {
              setMadeBackup(true);
              showToast('success', l10n.getString('backup-now-done'));
            }
          },
          (failure: unknown) => {
            setError(uiErrorOf(failure));
          },
        ),
      undefined,
    );
  };
  return (
    <div className="fixed inset-x-0 bottom-statusbar z-10 flex justify-center px-4 pb-4 pe-corner normal:pe-4 print:hidden">
      <div className="w-full max-w-form shadow-overlay">
        <Card icon={CircleCheck} title={<Localized id="restore-after-title" />}>
          <div className="flex flex-col gap-3">
            <Localized id="restore-after-help">
              <p className="type-body text-ink-muted" />
            </Localized>
            {error === null ? null : <ErrorNotice error={error} />}
            <div className="flex flex-wrap items-center gap-2">
              <Button variant="primary" busy={flight.busy} disabled={madeBackup} onClick={backUp}>
                <DatabaseBackup aria-hidden="true" className="size-icon-md" />
                <Localized id="restore-after-backup" />
              </Button>
              {pending.cloudNeedsKeys ? (
                <Button
                  onClick={() => {
                    void navigate({ to: '/backup' });
                  }}
                >
                  <Settings aria-hidden="true" className="size-icon-md" />
                  <Localized id="restore-after-cloud-keys" />
                </Button>
              ) : null}
              {pending.assistantWasOn ? (
                <Button
                  onClick={() => {
                    void navigate({ to: '/settings' });
                  }}
                >
                  <Settings aria-hidden="true" className="size-icon-md" />
                  <Localized id="restore-after-openrouter" />
                </Button>
              ) : null}
              <Button shortcut={SHORTCUTS.back} onClick={clear}>
                <Localized id="restore-after-close" />
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
