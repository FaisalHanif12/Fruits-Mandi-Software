// The state of one print or PDF job started from a preview (PRD FR-19.3): it runs the job once at a
// time, keeps what it did (the PDF files, or "sent to the printer") or what the backend refused (a bad
// filter at its field, or `ERR_PRINT_WINDOWS_ONLY` off Windows, where the preview stays the way to see
// the page), and forgets both when the print language or the paper changes, because they were about the
// page before. The preview screens of a report, of its slips and of an analytics view share it.

import { useState } from 'react';
import type { PrintOutcome } from '../../../bindings/PrintOutcome';
import type { PrintTo } from '../../../bindings/PrintTo';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';

export interface PrintJob {
  /** Starts the job, to the printer or to a PDF file. Does nothing while one is running. */
  send: (to: PrintTo) => void;
  busy: boolean;
  /** What the last job did, or null. */
  outcome: PrintOutcome | null;
  /** What the backend refused, or null. */
  error: UiError | null;
  /** Forgets the outcome and the error (the page they were about has changed). */
  reset: () => void;
}

export function usePrintJob(run: (to: PrintTo) => Promise<PrintOutcome>): PrintJob {
  const flight = useSingleFlight();
  const [outcome, setOutcome] = useState<PrintOutcome | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const send = (to: PrintTo) => {
    void flight.run(async () => {
      setError(null);
      setOutcome(null);
      try {
        setOutcome(await run(to));
        return true;
      } catch (failure: unknown) {
        setError(uiErrorOf(failure));
        return false;
      }
    }, false);
  };
  const reset = () => {
    setOutcome(null);
    setError(null);
  };
  return { send, busy: flight.busy, outcome, error, reset };
}
