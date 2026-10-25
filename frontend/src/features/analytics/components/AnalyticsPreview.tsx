// The print preview of an analytics view (PRD FR-7.36, FR-18.4, FR-19.13): the page that would be
// printed, drawn in the print language, with every row of the filter and the filters in its header
// (by day, month or year for view D). Print and PDF wait for the print job (as the reports do); the
// frame says so. No Excel (D5-Q1).

import { Localized } from '@fluent/react';
import { FileX } from 'lucide-react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import { ExcelExportButton, type ExcelExporter } from '../../../shared/export';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import {
  PrintPreviewFrame,
  ReportPrintDocument,
  usePrintFrameOf,
  usePrintJob,
} from '../../reports';
import { useAnalyticsPrintData, useAnalyticsPrintView } from '../hooks';
import { printTableOf } from '../print/print-table';
import type { AnalyticsFilters, AnalyticsView } from '../types';

interface AnalyticsPreviewProps {
  view: AnalyticsView;
  filters: AnalyticsFilters;
  onBack: () => void;
  /** The Excel export of the screen: its button sits in the frame, with the language chosen here. */
  exporter: ExcelExporter;
}

export function AnalyticsPreview({ view, filters, onBack, exporter }: AnalyticsPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const query = useAnalyticsPrintData({ view, filters, language, paper });
  // The ONE header and footer of every print: the backend builds them from the title and the filters.
  const frame = usePrintFrameOf(language, query.data);
  // "Print" and "Save as PDF": the same view, language and paper as the page on screen.
  const printView = useAnalyticsPrintView();
  const job = usePrintJob((to) =>
    printView.mutateAsync({ view: { view, filters, language, paper }, to, copies: null }),
  );

  let body;
  if (query.isError || frame.isError) {
    const failed = query.isError ? query : frame;
    body = (
      <ErrorState
        title={<Localized id="report-print-failed" />}
        error={uiErrorOf(failed.error)}
        onRetry={() => {
          void failed.refetch();
        }}
      />
    );
  } else if (query.data === undefined || frame.data === undefined) {
    body = <LoadingState rows={8} />;
  } else if (query.data.rows.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="analytics-empty-help" />}
      />
    );
  } else {
    body = <ReportPrintDocument data={printTableOf(query.data)} frame={frame.data} />;
  }

  return (
    <PrintPreviewFrame
      language={language}
      paper={paper}
      onLanguageChange={setLanguage}
      onPaperChange={setPaper}
      onBack={onBack}
      excel={<ExcelExportButton exporter={exporter} language={language} />}
      job={job}
      canPrint={query.data !== undefined && frame.data !== undefined && query.data.rows.length > 0}
    >
      {body}
    </PrintPreviewFrame>
  );
}
