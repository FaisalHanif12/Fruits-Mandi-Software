// The print preview of a report (PRD FR-7.36, FR-19.3, FR-19.13): the same page that is printed,
// drawn in the print language, which can differ from the app language. It holds every row of the
// filter, not one page of it. The language and the paper are chosen in the frame
// (PrintPreviewFrame); the app's language and the filters stay as they are.

import { Localized } from '@fluent/react';
import { FileX } from 'lucide-react';
import { useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { SortInput } from '../../../bindings/SortInput';
import type { ReportFilters } from '../../../bindings/ReportFilters';
import type { ReportId } from '../../../bindings/ReportId';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import { ExcelExportButton, type ExcelExporter } from '../../../shared/export';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { usePrintFrameOf, usePrintReport, useReportPrintData } from '../hooks';
import { ReportPrintDocument } from '../print/ReportPrintDocument';
import { PrintPreviewFrame } from './PrintPreviewFrame';
import { usePrintJob } from './usePrintJob';

interface ReportPreviewProps {
  id: ReportId;
  filters: ReportFilters;
  /** The order and the month headings of the screen: the print has the same (plan 38 R4). */
  sort: SortInput | null;
  groupByMonth: boolean;
  onBack: () => void;
  /** The Excel export of the screen: its button sits in the frame, with the language chosen here. */
  exporter: ExcelExporter;
}

export function ReportPreview({
  id,
  filters,
  sort,
  groupByMonth,
  onBack,
  exporter,
}: ReportPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const query = useReportPrintData({
    id,
    filters,
    language,
    paper,
    sort,
    group_by_month: groupByMonth,
  });
  // The ONE header and footer of every print: the backend builds them from the title and the filters.
  const frame = usePrintFrameOf(language, query.data);
  // "Print" and "Save as PDF": the same report, language and paper as the page on screen.
  const printReport = usePrintReport();
  const job = usePrintJob((to) =>
    printReport.mutateAsync({
      report: { id, filters, language, paper, sort, group_by_month: groupByMonth },
      slips: false,
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
  } else if (query.data.rows.length === 0 && query.data.summary.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <ReportPrintDocument data={query.data} frame={frame.data} />;
  }

  // There is something to send once the page is drawn: not while it loads, fails or is empty.
  const isPrintable =
    query.data !== undefined &&
    frame.data !== undefined &&
    (query.data.rows.length > 0 || query.data.summary.length > 0);

  return (
    <PrintPreviewFrame
      language={language}
      paper={paper}
      onLanguageChange={setLanguage}
      onPaperChange={setPaper}
      onBack={onBack}
      excel={<ExcelExportButton exporter={exporter} language={language} />}
      job={job}
      canPrint={isPrintable}
    >
      {body}
    </PrintPreviewFrame>
  );
}
