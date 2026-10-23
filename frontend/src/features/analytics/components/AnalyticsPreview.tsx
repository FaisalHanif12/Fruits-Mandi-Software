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
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { PrintPreviewFrame, ReportPrintDocument } from '../../reports';
import { useAnalyticsPrintData } from '../hooks';
import { printTableOf } from '../print/print-table';
import type { AnalyticsFilters, AnalyticsView } from '../types';

interface AnalyticsPreviewProps {
  view: AnalyticsView;
  filters: AnalyticsFilters;
  onBack: () => void;
}

export function AnalyticsPreview({ view, filters, onBack }: AnalyticsPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const query = useAnalyticsPrintData({ view, filters, language, paper });

  let body;
  if (query.isPending) {
    body = <LoadingState rows={8} />;
  } else if (query.isError) {
    body = (
      <ErrorState
        title={<Localized id="report-print-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  } else if (query.data.rows.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="analytics-empty-help" />}
      />
    );
  } else {
    body = <ReportPrintDocument data={printTableOf(query.data)} />;
  }

  return (
    <PrintPreviewFrame
      language={language}
      paper={paper}
      onLanguageChange={setLanguage}
      onPaperChange={setPaper}
      onBack={onBack}
    >
      {body}
    </PrintPreviewFrame>
  );
}
