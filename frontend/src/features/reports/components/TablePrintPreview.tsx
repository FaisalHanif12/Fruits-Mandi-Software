// The print preview of the table prints of plan 76 (PRD FR-7.36, FR-19.3, FR-19.13): the اگراہی sheet (R5),
// the buyer dues page, the commission page and a partner's month statement. The same page that is printed,
// drawn in the print language, which can differ from the app language. It holds every row of the filter,
// not one page of it. The backend sends it ready as `TablePrintData` (header, footer, columns, rows, totals,
// all in the print language); this reads it again when the language or the paper changes, draws it with the
// shared print layout and offers Print, Save as PDF and, where the screen has it, Excel through the shared
// frame. Four states: loading, an error with a retry, nothing to print (Print stays dimmed), and the page.

import { Localized } from '@fluent/react';
import { useQuery } from '@tanstack/react-query';
import { FileX } from 'lucide-react';
import { useState, type ReactNode } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { PrintOutcome } from '../../../bindings/PrintOutcome';
import type { PrintTo } from '../../../bindings/PrintTo';
import type { ReportId } from '../../../bindings/ReportId';
import type { TablePrintData } from '../../../bindings/TablePrintData';
import { uiErrorOf } from '../../../shared/api';
import { ExcelExportButton, ExcelExportNotices, type ExcelExporter } from '../../../shared/export';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { ReportPrintDocument } from '../print/ReportPrintDocument';
import { printTableOf } from '../print/table-print';
import { PrintPreviewFrame } from './PrintPreviewFrame';
import { usePrintJob } from './usePrintJob';

interface TablePrintPreviewProps {
  /** What the print is about (its kind and its filter), part of the key the data is kept under. */
  scope: readonly unknown[];
  /** Reads every row in the print language and on the paper chosen in the frame. */
  read: (language: Language, paper: Paper) => Promise<TablePrintData>;
  /** Sends the page to the printer or to a PDF file, in the language and on the paper chosen. */
  send: (to: PrintTo, language: Language, paper: Paper) => Promise<PrintOutcome>;
  /** The report the page is, when the layout has words of its own for it (R5). */
  report?: ReportId;
  /** The Excel export of the screen: its button sits in the frame, with the language chosen here. */
  exporter?: ExcelExporter;
  /** The heading of the preview. Left out, it is the words of a report print. */
  title?: ReactNode;
  onBack: () => void;
}

export function TablePrintPreview({
  scope,
  read,
  send,
  report,
  exporter,
  title,
  onBack,
}: TablePrintPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  // A print is a snapshot of the filter on screen: it is read again for a new language or paper only.
  const query = useQuery({
    queryKey: ['table-print', ...scope, language, paper],
    queryFn: () => read(language, paper),
    staleTime: Infinity,
    gcTime: 0,
  });
  const job = usePrintJob((to) => send(to, language, paper));

  let body;
  if (query.isError) {
    body = (
      <ErrorState
        title={<Localized id="report-print-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  } else if (query.data === undefined) {
    body = <LoadingState rows={8} />;
  } else if (query.data.rows.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <ReportPrintDocument data={printTableOf(query.data, report)} frame={query.data.frame} />;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      {exporter === undefined ? null : <ExcelExportNotices exporter={exporter} />}
      <PrintPreviewFrame
        language={language}
        paper={paper}
        onLanguageChange={setLanguage}
        onPaperChange={setPaper}
        onBack={onBack}
        {...(exporter === undefined
          ? {}
          : { excel: <ExcelExportButton exporter={exporter} language={language} /> })}
        job={job}
        canPrint={query.data !== undefined && query.data.rows.length > 0}
        {...(title === undefined ? {} : { title })}
      >
        {body}
      </PrintPreviewFrame>
    </div>
  );
}
