// The print preview of پچھلی بکری (plan 72 P8, PRD FR-7.36, FR-19.3, FR-19.13): the same page that is
// printed, drawn in the print language, which can differ from the app language. It holds every row of the
// filter, not one page of it, and follows the switch "show every بکری" because the filter it is given
// already has it. The language and the paper are chosen in the shared frame; Print and Save as PDF go
// through the backend's print job, Excel through the screen's exporter with the language chosen here.

import { Localized } from '@fluent/react';
import { FileX } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { uiErrorOf } from '../../../shared/api';
import { ExcelExportButton, type ExcelExporter } from '../../../shared/export';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { PrintPreviewFrame, usePrintJob } from '../../reports';
import { usePastPrintBundle, usePrintPastList } from '../hooks/usePastPrint';
import type { BikriListPrintInput } from '../../../bindings/BikriListPrintInput';
import type { PastPrintFilter } from '../past-print-filter';
import { PastPrintDocument } from '../print/PastPrintDocument';
import { pastPrintIsEmpty } from '../print/past-print-table';

interface PastPreviewProps {
  /** What the list asks for now: today, the window, the filters and the switch. */
  filter: PastPrintFilter;
  onBack: () => void;
  /** The Excel export of the screen: its button sits in the frame, with the language chosen here. */
  exporter: ExcelExporter;
}

export function PastPreview({ filter, onBack, exporter }: PastPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const input = useMemo<BikriListPrintInput>(
    () => ({ ...filter, language, paper }),
    [filter, language, paper],
  );
  const query = usePastPrintBundle(input);
  // "Print" and "Save as PDF": the same list, language and paper as the page on screen.
  const printList = usePrintPastList();
  const job = usePrintJob((to) => printList.mutateAsync({ list: input, to, copies: null }));

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
  } else if (pastPrintIsEmpty(query.data.data)) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <PastPrintDocument data={query.data.data} frame={query.data.frame} />;
  }
  // There is something to send once the page is drawn: not while it loads, fails or is empty.
  const isPrintable = query.data !== undefined && !pastPrintIsEmpty(query.data.data);

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
