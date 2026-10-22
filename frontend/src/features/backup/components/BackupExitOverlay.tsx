// While the copy at exit runs (PRD FR-20.3), the backend holds the first close request and says so
// with the event `backup-exit`. The Owner sees why the window has not closed yet: a message over
// the screen, with progress as a busy state (there is no percentage). The window closes by itself
// when the copy is done, or after 120 seconds. Mounted once, outside the sign-in gates, because the
// window can be closed from any screen.

import { Localized } from '@fluent/react';
import { useEffect, useState } from 'react';
import { listenEvent } from '../../../shared/api';
import { LoadingState } from '../../../shared/ui';

/** The event the backend sends while the copy at exit runs (plan 18 contract). */
export const BACKUP_EXIT_EVENT = 'backup-exit';

/** The payload is `{ phase: 'running' }` and then `{ phase: 'done' }`; anything else is ignored. */
function phaseOf(payload: unknown): 'running' | 'done' | null {
  if (typeof payload !== 'object' || payload === null || !('phase' in payload)) return null;
  return payload.phase === 'running' || payload.phase === 'done' ? payload.phase : null;
}

export function BackupExitOverlay() {
  const [running, setRunning] = useState(false);
  useEffect(() => {
    let stop: (() => void) | undefined;
    let gone = false;
    void listenEvent(BACKUP_EXIT_EVENT, (payload) => {
      const phase = phaseOf(payload);
      if (phase !== null) setRunning(phase === 'running');
    }).then((unlisten) => {
      // The screen may have gone while the listener was being made.
      if (gone) unlisten();
      else stop = unlisten;
    });
    return () => {
      gone = true;
      stop?.();
    };
  }, []);
  if (!running) return null;
  return (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="backup-exit-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-scrim p-6"
    >
      <div className="flex w-full max-w-dialog flex-col gap-3 rounded-3 bg-surface p-6 text-ink shadow-overlay">
        <Localized id="backup-exit-title">
          <h2 id="backup-exit-title" className="type-section" />
        </Localized>
        <Localized id="backup-exit-help">
          <p className="type-body text-ink-muted" />
        </Localized>
        <LoadingState rows={1} />
      </div>
    </div>
  );
}
