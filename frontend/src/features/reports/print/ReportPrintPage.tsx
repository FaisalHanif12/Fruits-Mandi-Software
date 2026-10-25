// The hidden print window's page for a report: `#/print?report=<id>&language=<ur|en>&paper=<a4|a5>
// [&slips=1]&filters=<JSON>&job=<job>` (17e contract). The backend opens a window with this address,
// waits for `print-ready-<job>` and then calls the WebView2 print on it (or saves it as a PDF). A page
// that cannot be read says `print-error-<job>` instead. Like the other print windows it sits outside the
// app shell, and its capability allows the print-data and print-header commands only.

import { Localized } from '@fluent/react';
import { useRouterState } from '@tanstack/react-router';
import { useEffect, useMemo } from 'react';
import { emitEvent } from '../../../shared/api';
import {
  jobOf,
  printErrorEvent,
  printReadyEvent,
  type PrintFailed,
  type PrintReady,
} from '../../../shared/print';
import { ErrorState } from '../../../shared/ui';
import { parseReportPrintParams } from './params';
import { ReportPrintView } from './ReportPrintView';

export function ReportPrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const params = useMemo(() => parseReportPrintParams(searchStr), [searchStr]);
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
    <ReportPrintView
      input={params.input}
      slips={params.slips}
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
