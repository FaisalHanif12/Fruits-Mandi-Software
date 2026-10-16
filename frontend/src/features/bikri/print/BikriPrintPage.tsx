// The hidden print window's route: `#/print?bikri=<id>&doc=<doc>&language=<ur|en>&job=<job>
// [&buyer=<id>]` (plan 14 contract 7). The backend opens a window with this address, waits for
// `print-ready-<job>` (with the page height in CSS pixels, for the 80 mm roll) and then calls the
// WebView2 print on it. A page that cannot be read says `print-error-<job>` instead.
//
// The window is outside the app shell: no sign-in screen, no menus. The Owner is signed in in the
// backend, which every command checks. The capability of this window allows one data command
// only, so this page calls nothing else.

import { Localized } from '@fluent/react';
import { useRouterState } from '@tanstack/react-router';
import { useEffect, useMemo } from 'react';
import { emitEvent } from '../../../shared/api';
import type { PrintReady } from '../../../shared/print';
import { ErrorState } from '../../../shared/ui';
import { BikriPrintView } from './BikriPrintView';
import { jobOf, parsePrintParams } from './params';

/** The event the backend waits for. */
export const printReadyEvent = (job: string): string => `print-ready-${job}`;
/** The event that says the page could not be made. */
export const printErrorEvent = (job: string): string => `print-error-${job}`;

/** What a failed print tells the backend: the error codes. */
interface PrintFailed {
  codes: string[];
}

export function BikriPrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const params = useMemo(() => parsePrintParams(searchStr), [searchStr]);
  const job = jobOf(searchStr);

  // An address that is not a print address is a failed print, told once.
  useEffect(() => {
    if (params === null && job !== null) {
      const failed: PrintFailed = { codes: ['ERR_INTERNAL'] };
      void emitEvent(printErrorEvent(job), failed);
    }
  }, [params, job]);

  if (params === null) {
    return (
      <ErrorState
        title={<Localized id="print-preview-failed" />}
        error={{ ok: false, errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }], ref: '' }}
        onRetry={() => {
          window.location.reload();
        }}
      />
    );
  }
  return (
    <BikriPrintView
      input={params.input}
      mode="window"
      onReady={(ready: PrintReady) => {
        void emitEvent(printReadyEvent(params.job), ready);
      }}
      onFailed={(codes) => {
        const failed: PrintFailed = { codes };
        void emitEvent(printErrorEvent(params.job), failed);
      }}
    />
  );
}
