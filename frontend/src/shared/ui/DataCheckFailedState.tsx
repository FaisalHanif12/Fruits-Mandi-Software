// Fills the window when the start-up data check fails (plan 03 contract 4.3, NFR-REL-7).
// Saving is stopped until the check passes, so the app has nothing safe to show. The words are
// calm: what happened, what to do, the reference number (NFR-OBS-3), and one button that runs
// the check again. The shell places it (plan 04). The restore offer comes with backups (D5).

import { Localized } from '@fluent/react';
import { ShieldAlert } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { Button } from './Button';
import { Code } from './Code';
import { StateBadge } from './StateBadge';

interface DataCheckFailedStateProps {
  /** The reference number of the failed check. It matches a log line. Empty when there is none. */
  reference: string;
  onRetry: () => void;
  /** The check is running again. The button waits and cannot be pressed twice. */
  retrying?: boolean;
  /** True when the shell shows this alone in the window. False inside a card, as in the gallery. */
  fullWindow?: boolean;
}

export function DataCheckFailedState({
  reference,
  onRetry,
  retrying = false,
  fullWindow = true,
}: DataCheckFailedStateProps) {
  const retryButton = useRef<HTMLButtonElement>(null);
  // Alone in the window, the button is the one thing to do, so the keyboard starts on it.
  useEffect(() => {
    if (fullWindow) retryButton.current?.focus();
  }, [fullWindow]);
  const Heading = fullWindow ? 'h1' : 'h2';
  return (
    <div
      className={
        fullWindow
          ? 'flex min-h-screen items-center justify-center bg-bg p-6'
          : 'flex items-center justify-center p-4'
      }
    >
      <div
        role="alert"
        className="flex w-full max-w-reading flex-col items-center gap-3 text-center"
      >
        <StateBadge icon={ShieldAlert} tone="due" />
        <Localized id="common-data-check-title">
          <Heading className="type-title text-ink" />
        </Localized>
        <Localized id="err-data-check-failed">
          <p className="type-body text-ink" />
        </Localized>
        {reference === '' ? null : (
          <Localized id="err-reference" vars={{ ref: reference }} elems={{ code: <Code /> }}>
            <p className="type-body text-ink-muted" />
          </Localized>
        )}
        <Button variant="primary" busy={retrying} onClick={onRetry} ref={retryButton}>
          <Localized id="common-retry" />
        </Button>
      </div>
    </div>
  );
}
