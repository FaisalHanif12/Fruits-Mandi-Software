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
import { useBuyerBillsPrintData, usePrintFrameOf, usePrintReport } from '../hooks';
import { BuyerBillsPrintDocument } from '../print/BuyerBillsPrintDocument';
import { PrintPreviewFrame } from './PrintPreviewFrame';
import { usePrintJob } from './usePrintJob';

interface BuyerBillsPreviewProps {
  filters: ReportFilters;
  onBack: () => void;
}

export function BuyerBillsPreview({ filters, onBack }: BuyerBillsPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  // One slip for each buyer: the slips are in the report's own order.
  const query = useBuyerBillsPrintData({
    id: 'R5',
    filters,
    language,
    paper,
    sort: null,
    group_by_month: false,
  });
  // The ONE header and footer of every print: the backend builds them from the title and the filters.
  const frame = usePrintFrameOf(language, query.data);
  // "Print" and "Save as PDF": the slips of all the credit buyers, in the language and paper chosen here.
  const printReport = usePrintReport();
  const job = usePrintJob((to) =>
    printReport.mutateAsync({
      report: { id: 'R5', filters, language, paper, sort: null, group_by_month: false },
      slips: true,
      to,
      copies: null,
    }),
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
  } else if (query.data.slips.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <BuyerBillsPrintDocument data={query.data} frame={frame.data} />;
  }

  return (
    <PrintPreviewFrame
      language={language}
      paper={paper}
      onLanguageChange={setLanguage}
      onPaperChange={setPaper}
      onBack={onBack}
      job={job}
      canPrint={query.data !== undefined && frame.data !== undefined && query.data.slips.length > 0}
    >
      {body}
    </PrintPreviewFrame>
  );
}
