// The hidden print window's page for the collection list: `#/print?collection=<YYYY-MM-DD>&doc=
// collection_list&language=<ur|en>[&collector=<id>][&city=<id>]&job=<job>` (plan 17e). The backend
// opens a window with this address, waits for `print-ready-<job>` (with the page height in CSS
// pixels, for the 80 mm roll) and then calls the WebView2 print on it. A page that cannot be read
// says `print-error-<job>` instead. Like the other print pages it sits outside the app shell, and
// its capability allows the print-data commands only.

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
import { CollectionPrintView } from './CollectionPrintView';
import { parseCollectionPrintParams } from './collection-params';

export function CollectionPrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const params = useMemo(() => parseCollectionPrintParams(searchStr), [searchStr]);
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
    <CollectionPrintView
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
