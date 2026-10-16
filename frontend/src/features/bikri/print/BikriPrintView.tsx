// One printed document of a saved بکری on screen: reads its print data, draws the template, and
// says when the page is ready. It is the body of the hidden print window (mode `window`) and also
// the in-app preview (mode `preview`), so what the user previews is exactly what is printed.
//
// It calls ONE command, `reports_bikri_print_data` (the capability of the hidden window allows no
// other). The four states: loading, an error with a retry, nothing to print, and the document.

import { Localized } from '@fluent/react';
import { FileX } from 'lucide-react';
import { useEffect, useRef } from 'react';
import type { BikriPrintDataInput } from '../../../bindings/BikriPrintDataInput';
import { uiErrorOf } from '../../../shared/api';
import { useLatest } from '../../../shared/hooks';
import { fontFamiliesOf, waitForPrintReady, type PrintReady } from '../../../shared/print';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { useBikriPrintData } from '../../reports';
import { BikriPrintDocument } from './BikriPrintDocument';

interface BikriPrintViewProps {
  input: BikriPrintDataInput;
  /** `window`: the hidden print window. `preview`: inside the app, which keeps its own language. */
  mode: 'window' | 'preview';
  /** The document is drawn, its fonts are loaded and its layout is final. Called once. */
  onReady?: (ready: PrintReady) => void;
  /** The print data could not be read. The codes of the error. Called once. */
  onFailed?: (codes: string[]) => void;
}

export function BikriPrintView({ input, mode, onReady, onFailed }: BikriPrintViewProps) {
  const query = useBikriPrintData(input);
  const ready = useLatest(onReady);
  const failed = useLatest(onFailed);
  const told = useRef(false);
  // The document itself, to measure its height for the 80 mm roll.
  const documentRef = useRef<HTMLDivElement>(null);
  const data = query.data;
  const error = query.error;

  // Once the document is drawn, wait for the fonts and the layout, then say so. One time only: a
  // second tell (a development double render) would print twice.
  useEffect(() => {
    if (data === undefined) return undefined;
    let current = true;
    const sample = data.pages.map((page) => page.party.name).join(' ');
    void waitForPrintReady(fontFamiliesOf(data), sample, documentRef.current).then((result) => {
      if (!current || told.current) return;
      told.current = true;
      ready()?.(result);
    });
    return () => {
      current = false;
    };
  }, [data, ready]);

  useEffect(() => {
    if (error === null || told.current) return;
    told.current = true;
    failed()?.(uiErrorOf(error).errors.map((item) => item.code));
  }, [error, failed]);

  if (query.isPending) return <LoadingState rows={8} />;
  if (query.isError) {
    return (
      <ErrorState
        title={<Localized id="print-preview-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          told.current = false;
          void query.refetch();
        }}
      />
    );
  }
  if (query.data.pages.length === 0) {
    return (
      <EmptyState
        icon={FileX}
        title={<Localized id="print-preview-empty" />}
        help={<Localized id="print-preview-help" />}
      />
    );
  }
  return <BikriPrintDocument ref={documentRef} data={query.data} applyRoot={mode === 'window'} />;
}
