// One printed document on screen: reads its print data, draws it, and says when the page is ready
// (plan 14, plan 15). It is the body of the hidden print window and also the in-app preview, so
// what the user previews is exactly what is printed (the module's `render` decides whether the
// document takes over the root of the page, which only the hidden window does). Every document
// of the app uses it (the sale book's three, the receipt slip, the reports of D5): the module
// gives the query for its print data and the template that draws it.
//
// The four states: loading, an error with a retry, nothing to print, and the document.

import { Localized } from '@fluent/react';
import type { UseQueryResult } from '@tanstack/react-query';
import { FileX } from 'lucide-react';
import { useEffect, useRef, type ReactNode, type Ref } from 'react';
import { uiErrorOf } from '../api';
import { useLatest } from '../hooks';
import { EmptyState } from '../ui/EmptyState';
import { ErrorState } from '../ui/ErrorState';
import { LoadingState } from '../ui/LoadingState';
import type { PrintData, PrintFrame } from './types';
import { fontFamiliesOf, waitForPrintReady, type PrintReady } from './wait-ready';

/** What every printed document gives: the frame the backend built, whatever its body is. */
type PrintedFrame = Pick<PrintFrame, 'language' | 'urdu_digits'>;

/** What the view needs to know of a document that its own type says: is it empty, and some text to load the fonts for. */
export interface PrintDescription {
  empty: boolean;
  sample: string;
}

/** The sheets of a بکری or a voucher: empty when it has no page, the party names are the sample. */
export function describePrintData(data: PrintData): PrintDescription {
  return {
    empty: data.pages.length === 0,
    sample: data.pages.map((page) => page.party.name).join(' '),
  };
}

interface PrintViewProps<D extends PrintedFrame> {
  query: UseQueryResult<D>;
  /** Says whether the document has nothing to print, and gives a sample of its text (for the fonts). */
  describe: (data: D) => PrintDescription;
  /** The document is drawn, its fonts are loaded and its layout is final. Called once. */
  onReady?: (ready: PrintReady) => void;
  /** The print data could not be read. The codes of the error. Called once. */
  onFailed?: (codes: string[]) => void;
  /** Draws the document with the module's template. `ref` goes on the element that is measured. */
  render: (data: D, ref: Ref<HTMLDivElement>) => ReactNode;
}

export function PrintView<D extends PrintedFrame>({
  query,
  describe,
  onReady,
  onFailed,
  render,
}: PrintViewProps<D>) {
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
    const { sample } = describe(data);
    void waitForPrintReady(fontFamiliesOf(data), sample, documentRef.current).then((result) => {
      if (!current || told.current) return;
      told.current = true;
      ready()?.(result);
    });
    return () => {
      current = false;
    };
  }, [data, describe, ready]);

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
  if (describe(query.data).empty) {
    return (
      <EmptyState
        icon={FileX}
        title={<Localized id="print-preview-empty" />}
        help={<Localized id="print-preview-help" />}
      />
    );
  }
  return <>{render(query.data, documentRef)}</>;
}

export type { PrintViewProps };
