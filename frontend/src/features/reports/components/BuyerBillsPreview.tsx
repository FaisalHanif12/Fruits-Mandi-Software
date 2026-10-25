// The print preview of all the credit buyers' slips of R5, one for each buyer (PRD 19.2 R5 "Print
// All", FR-7.36). It is the same frame as the preview of a report (PrintPreviewFrame): the language
// and the paper are chosen there and can differ from the app's, and the app's own language and filters
// stay as they are. The slips hold every buyer of the filter, not one page of it.

import { Localized } from '@fluent/react';
import { FileX } from 'lucide-react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { ReportFilters } from '../../../bindings/ReportFilters';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { useBuyerBillsPrintData } from '../hooks';
import { BuyerBillsPrintDocument } from '../print/BuyerBillsPrintDocument';
import { PrintPreviewFrame } from './PrintPreviewFrame';

interface BuyerBillsPreviewProps {
  filters: ReportFilters;
  onBack: () => void;
}

export function BuyerBillsPreview({ filters, onBack }: BuyerBillsPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const query = useBuyerBillsPrintData({ id: 'R5', filters, language, paper });

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
  } else if (query.data.slips.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <BuyerBillsPrintDocument data={query.data} />;
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
