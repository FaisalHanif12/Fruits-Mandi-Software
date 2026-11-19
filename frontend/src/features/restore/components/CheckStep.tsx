// Step 3 of the restore: what is in the copy (PRD FR-20.11, NFR-BAK-5). The copy has been opened and
// checked in a temporary place; nothing is on this computer yet. The page shows its date, each check
// and the books in it, all read from the copy by the backend (this screen works out nothing). A copy
// that failed a check cannot be restored: the page says so and offers to go back. Pressing Restore
// asks once more, saying exactly what will happen (PRD FR-7.8).

import { Localized } from '@fluent/react';
import { HardDriveDownload, TriangleAlert } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { utcToLocalDisplay } from '../../../shared/format';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button, Card, Code, ConfirmDialog, EmptyState, Notice } from '../../../shared/ui';
import { TotalsTable } from '../../backup';
import { runRestore } from '../flow';
import { STEP_COPY, useRestoreStore } from '../store';
import { CheckList } from './CheckList';

export function CheckStep() {
  const preview = useRestoreStore((state) => state.preview);
  const goTo = useRestoreStore((state) => state.goTo);
  const [confirming, setConfirming] = useState(false);
  // The main button has focus when the page opens, so one Enter carries on (PRD FR-7.1, NFR-USE-7):
  // Restore for a good copy, Back for one that failed a check.
  const main = useRef<HTMLButtonElement>(null);
  const good = preview?.checks.every((check) => check.ok) === true;
  useEffect(() => {
    main.current?.focus();
  }, [good]);

  if (preview === null) {
    // Only reachable by a slip: go back and check again.
    return (
      <Card>
        <EmptyState
          icon={TriangleAlert}
          title={<Localized id="restore-check-missing" />}
          help={<Localized id="restore-check-missing-help" />}
          action={
            <Button
              variant="primary"
              onClick={() => {
                goTo(STEP_COPY);
              }}
            >
              <Localized id="common-back" />
            </Button>
          }
        />
      </Card>
    );
  }

  const date = utcToLocalDisplay(preview.created_at);
  return (
    <Card>
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <Localized id="restore-check-title">
            <h2 className="type-section" />
          </Localized>
          <Localized id="backup-restore-made-on" vars={{ date }} elems={{ num: <Code /> }}>
            <p className="type-body" />
          </Localized>
        </div>
        <Notice tone={good ? 'info' : 'error'}>
          <Localized id={good ? 'restore-check-ok' : 'restore-check-bad'} />
        </Notice>
        {/* On a wide window the checks and the buttons stand beside the books, so the buttons stay in
            view at 1366 x 768. On a small window the books come first, then the checks and the
            buttons: he reads what is in the copy before he restores it. */}
        <div className="grid items-start gap-4 normal:grid-cols-2">
          <TotalsTable variant="copy" copy={preview.totals} />
          <div className="flex flex-col gap-4 normal:order-first">
            <CheckList checks={preview.checks} />
            {preview.needs_upgrade ? (
              <Notice tone="info">
                <Localized id="backup-restore-upgrade" />
              </Notice>
            ) : null}
            <div className="flex flex-wrap items-center justify-between gap-2">
              <Button
                ref={good ? null : main}
                shortcut={SHORTCUTS.back}
                onClick={() => {
                  goTo(STEP_COPY);
                }}
              >
                <Localized id="common-back" />
              </Button>
              {good ? (
                <Button
                  ref={main}
                  variant="primary"
                  shortcut={SHORTCUTS.confirm}
                  onClick={() => {
                    setConfirming(true);
                  }}
                >
                  <HardDriveDownload aria-hidden="true" className="size-icon-md" />
                  <Localized id="backup-restore-button" />
                </Button>
              ) : null}
            </div>
          </div>
        </div>
      </div>
      <ConfirmDialog
        open={confirming}
        onOpenChange={setConfirming}
        title={<Localized id="restore-confirm-title" />}
        description={
          <Localized id="restore-confirm-description" vars={{ date }} elems={{ num: <Code /> }}>
            <span />
          </Localized>
        }
        confirmLabel={<Localized id="backup-restore-button" />}
        onConfirm={() => {
          setConfirming(false);
          void runRestore();
        }}
      />
    </Card>
  );
}
