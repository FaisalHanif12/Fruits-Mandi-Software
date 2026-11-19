// Step 4 of the restore: the books are put on this computer (PRD FR-20.11, NFR-BAK-12). While it
// runs the page shows the stage it is in, a list of all stages and a bar when the stage has a measure,
// and asks the Owner not to close the app. When it is done the page shows the books that came back and
// the two things to do after signing in; "Go to sign in" then opens the sign-in screen, where the
// recovery code is the way in if the password is forgotten (PRD FR-8.5). A failure never shows here:
// it sends the Owner back to the first screen with the reason (see the store).

import { Localized } from '@fluent/react';
import { useQueryClient } from '@tanstack/react-query';
import { CircleCheck, Circle, LoaderCircle, LogIn } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { utcToLocalDisplay } from '../../../shared/format';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button, Card, Code, Notice } from '../../../shared/ui';
import { AUTH_STATUS_KEY } from '../../auth/hooks';
import { TotalsTable } from '../../backup';
import { useRestoreStore } from '../store';
import { ProgressBar } from './ProgressBar';
import { RESTORE_STAGES, type RestoreStage } from '../types';

export function RunStep() {
  const done = useRestoreStore((state) => state.done);
  return done === null ? <Running /> : <Done />;
}

function Running() {
  const progress = useRestoreStore((state) => state.progress);
  const needsUpgrade = useRestoreStore((state) => state.preview?.needs_upgrade === true);
  // A copy of this version has nothing to upgrade, so its line is left out.
  const stages = RESTORE_STAGES.filter((stage) => stage !== 'upgrading' || needsUpgrade);
  const at = progress === null ? 0 : stages.findIndex((stage) => stage === progress.stage);
  const current = progress?.stage === 'done' ? stages.length : Math.max(0, at);
  return (
    <Card>
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <Localized id="restore-running-title">
            <h2 className="type-section" />
          </Localized>
          <Localized id="restore-running-help">
            <p role="status" className="type-body text-ink-muted" />
          </Localized>
        </div>
        <ol className="flex flex-col gap-2">
          {stages.map((stage, index) => (
            <StageRow
              key={stage}
              stage={stage}
              state={index < current ? 'done' : index === current ? 'current' : 'next'}
              percent={index === current ? (progress?.percent ?? null) : null}
            />
          ))}
        </ol>
      </div>
    </Card>
  );
}

interface StageRowProps {
  stage: Exclude<RestoreStage, 'done'>;
  state: 'done' | 'current' | 'next';
  percent: number | null;
}

function StageRow({ stage, state, percent }: StageRowProps) {
  return (
    <li
      aria-current={state === 'current' ? 'step' : undefined}
      className={`flex flex-col gap-1 ${state === 'next' ? 'text-ink-muted' : 'text-ink'}`}
    >
      <span className="flex items-center gap-2 type-body">
        {state === 'done' ? (
          <CircleCheck aria-hidden="true" className="size-icon-md shrink-0 text-money-in" />
        ) : state === 'current' ? (
          <LoaderCircle aria-hidden="true" className="size-icon-md shrink-0 text-primary" />
        ) : (
          <Circle aria-hidden="true" className="size-icon-md shrink-0" />
        )}
        <Localized id={`restore-stage-${stage.replaceAll('_', '-')}`} />
      </span>
      {state === 'current' && percent !== null ? <ProgressBar percent={percent} /> : null}
    </li>
  );
}

function Done() {
  const done = useRestoreStore((state) => state.done);
  const reset = useRestoreStore((state) => state.reset);
  const queryClient = useQueryClient();
  const signIn = useRef<HTMLButtonElement>(null);
  // The sign-in button has focus, so one Enter carries on.
  useEffect(() => {
    signIn.current?.focus();
  }, []);
  if (done === null) return null;

  // The app now has a data file, so who is signed in is read again first, while these pages still
  // show; only then are they left, and the gate shows the sign-in screen. No flash of the first screen.
  const goToSignIn = () => {
    void queryClient.invalidateQueries({ queryKey: AUTH_STATUS_KEY }).then(reset, reset);
  };
  return (
    <Card>
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <Localized id="restore-done-title">
            <h2 className="type-section" />
          </Localized>
          <Localized
            id="restore-done-help"
            vars={{ date: utcToLocalDisplay(done.restored_from) }}
            elems={{ num: <Code /> }}
          >
            <p role="status" className="type-body" />
          </Localized>
        </div>
        {/* What to do next, with the button, comes first on a small window and beside the books on a
            wider one, so the sign-in button is in view without scrolling at 1366 x 768. */}
        <div className="grid items-start gap-4 normal:grid-cols-2">
          <div className="flex flex-col gap-4 normal:order-last">
            <Notice tone="info">
              <Localized id="restore-done-signin-help" />
            </Notice>
            <div className="flex flex-col gap-1">
              <Localized id="restore-done-notes-title">
                <h3 className="type-body" />
              </Localized>
              <ul className="flex list-disc flex-col gap-1 ps-5 type-body text-ink-muted">
                <Localized id="restore-done-note-backup">
                  <li />
                </Localized>
                {done.cloud_needs_keys ? (
                  <Localized id="restore-done-note-cloud-keys">
                    <li />
                  </Localized>
                ) : null}
                {done.assistant_was_on ? (
                  <Localized id="restore-done-note-openrouter">
                    <li />
                  </Localized>
                ) : null}
              </ul>
            </div>
            <div className="flex justify-end">
              <Button
                ref={signIn}
                variant="primary"
                shortcut={SHORTCUTS.confirm}
                onClick={goToSignIn}
              >
                <LogIn aria-hidden="true" className="size-icon-md" />
                <Localized id="restore-done-signin" />
              </Button>
            </div>
          </div>
          <TotalsTable variant="copy" copy={done.totals} />
        </div>
      </div>
    </Card>
  );
}
