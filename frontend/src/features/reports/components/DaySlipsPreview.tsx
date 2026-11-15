// The print preview of the buyer parchi and of the overall sale print (PRD FR-7.36, FR-19.3, FR-19.15, plan 76
// point 3): the same page that is printed, drawn in the print language, which can differ from the app
// language. It holds every slip of the day (or of the بکری), not one page of it. The language and the
// paper are chosen in the frame; Print and Save as PDF go through the backend's print job with the same
// slips, language and paper. Four states: loading, an error with a retry, nothing to print (Print stays
// dimmed), and the slips.

import { Localized } from '@fluent/react';
import { useQuery } from '@tanstack/react-query';
import { FileX } from 'lucide-react';
import { useState } from 'react';
import type { DaySlipsInput } from '../../../bindings/DaySlipsInput';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import {
  reportsBuyerParchiPrint,
  reportsBuyerParchiPrintData,
  reportsSalePrint,
  reportsSalePrintData,
} from '../api';
import { DaySlipsDocument, type DaySlipsVariant } from '../print/DaySlipsDocument';
import { PrintPreviewFrame } from './PrintPreviewFrame';
import { usePrintJob } from './usePrintJob';

/** What the slips are about: the day, and the one بکری or the chosen buyers when the print is narrower. */
export type DaySlipsScope = Pick<DaySlipsInput, 'date' | 'bikri_id' | 'bikri_ids' | 'buyer_ids'>;

interface DaySlipsPreviewProps {
  variant: DaySlipsVariant;
  scope: DaySlipsScope;
  onBack: () => void;
}

export function DaySlipsPreview({ variant, scope, onBack }: DaySlipsPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const input: DaySlipsInput = {
    ...scope,
    // The parchi is the buyers only; the overall sale print has both parts.
    sections: variant === 'parchi' ? 'buyers' : 'both',
    language,
    paper,
  };
  // A print is a snapshot of what is on screen: it is read again for a new language or paper only.
  const query = useQuery({
    queryKey: ['day-slips-print', variant, scope, language, paper],
    queryFn: () =>
      variant === 'parchi' ? reportsBuyerParchiPrintData(input) : reportsSalePrintData(input),
    staleTime: Infinity,
    gcTime: 0,
  });
  const job = usePrintJob((to) =>
    variant === 'parchi'
      ? reportsBuyerParchiPrint({ slips: input, to, copies: null })
      : reportsSalePrint({ slips: input, to, copies: null }),
  );

  const count = query.data === undefined ? 0 : query.data.sellers.length + query.data.buyers.length;
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
  } else if (count === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="print-parchi-empty" />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body = <DaySlipsDocument data={query.data} variant={variant} />;
  }

  return (
    <PrintPreviewFrame
      language={language}
      paper={paper}
      onLanguageChange={setLanguage}
      onPaperChange={setPaper}
      onBack={onBack}
      job={job}
      canPrint={query.data !== undefined && count > 0}
      title={<Localized id={variant === 'parchi' ? 'print-parchi-label' : 'print-sale-label'} />}
    >
      {body}
    </PrintPreviewFrame>
  );
}
