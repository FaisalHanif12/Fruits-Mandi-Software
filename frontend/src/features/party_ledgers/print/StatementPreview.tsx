// The print preview of a party's statement (plan 74 T3, PRD FR-7.36, FR-12.47, FR-19.3, FR-19.13): the same page
// that is printed, drawn in the print language, which can differ from the app language. It holds every date of
// the period, not one page of it. The language and the paper are chosen in the shared frame; Print and Save as
// PDF go through the backend's print job with the language and the paper chosen there. Four states: loading,
// an error with a retry, nothing to print (no entry and no balance: Print stays dimmed), and the page.

import { Localized } from '@fluent/react';
import { FileX } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { PrintPreviewFrame, usePrintJob } from '../../reports';
import { hasNothingToPrint } from './statement-empty';
import { usePrintStatement, useStatementPrintBundle } from './statement-hooks';
import { StatementPrintDocument } from './StatementPrintDocument';
import type { StatementPrintDataInput } from '../../../bindings/StatementPrintDataInput';

interface StatementPreviewProps {
  accountId: number;
  /** The period as typed, DD-MM-YYYY. */
  from: string;
  to: string;
  onBack: () => void;
}

export function StatementPreview({ accountId, from, to, onBack }: StatementPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const input = useMemo<StatementPrintDataInput>(
    () => ({ account_id: accountId, from, to, language }),
    [accountId, from, to, language],
  );
  const query = useStatementPrintBundle(input);
  // "Print" and "Save as PDF": the same statement, period, language and paper as the page on screen.
  const printStatement = usePrintStatement();
  const job = usePrintJob((target) =>
    printStatement.mutateAsync({ statement: { ...input, paper }, to: target, copies: null }),
  );

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
  } else if (hasNothingToPrint(query.data.data)) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-print-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    // The page is drawn at the paper the user chose, not the paper the backend assumed.
    body = <StatementPrintDocument data={{ ...query.data.data, paper }} frame={query.data.frame} />;
  }
  // There is something to send once the page is drawn: not while it loads, fails or is empty.
  const isPrintable = query.data !== undefined && !hasNothingToPrint(query.data.data);

  return (
    <PrintPreviewFrame
      language={language}
      paper={paper}
      onLanguageChange={setLanguage}
      onPaperChange={setPaper}
      onBack={onBack}
      job={job}
      canPrint={isPrintable}
      title={<Localized id="statement-print-title" />}
    >
      {body}
    </PrintPreviewFrame>
  );
}
