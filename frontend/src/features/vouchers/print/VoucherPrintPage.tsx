// The hidden print window's page for a voucher: `#/print?voucher=<id>&doc=receipt_slip&language=
// <ur|en>&job=<job>` (plan 15 contract 9). The backend opens a window with this address, waits for
// `print-ready-<job>` (with the page height in CSS pixels, for the 80 mm roll) and then calls the
// WebView2 print on it. A page that cannot be read says `print-error-<job>` instead. Like the sale
// book's, it sits outside the app shell, and its capability allows the print-data commands only.

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
import { VoucherPrintView } from './VoucherPrintView';
import { parseVoucherPrintParams } from './params';

export function VoucherPrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const params = useMemo(() => parseVoucherPrintParams(searchStr), [searchStr]);
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
    <VoucherPrintView
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
