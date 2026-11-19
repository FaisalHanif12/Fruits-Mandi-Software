// The print preview of the three prints of the sale book's menu (PRD FR-7.36, FR-19.3, FR-19.15, plan 76
// point 3, plan 78 A1): the buyer slips, the seller bills and the whole sale print. The same page that is
// printed, drawn in the print language, which can differ from the app language. It holds every slip of the
// day (or of the بکری), not one page of it. The language and the paper are chosen in the frame; Print and
// Save as PDF go through the backend's print job with the same scope, language and paper. Four states:
// loading, an error with a retry, nothing to print (Print stays dimmed), and the page.

import { Localized } from '@fluent/react';
import { useQuery } from '@tanstack/react-query';
import { FileX } from 'lucide-react';
import { useState } from 'react';
import type { DaySlipsInput } from '../../../bindings/DaySlipsInput';
import type { DaySlipsPrintData } from '../../../bindings/DaySlipsPrintData';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { SalePrintData } from '../../../bindings/SalePrintData';
import { uiErrorOf } from '../../../shared/api';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import {
  reportsBuyerParchiPrint,
  reportsBuyerParchiPrintData,
  reportsSalePrint,
  reportsSalePrintData,
  reportsSellerBillsPrint,
  reportsSellerBillsPrintData,
} from '../api';
import { DaySlipsDocument, type DaySlipsVariant } from '../print/DaySlipsDocument';
import { SalePrintDocument } from '../print/SalePrintDocument';
import { PrintPreviewFrame } from './PrintPreviewFrame';
import { usePrintJob } from './usePrintJob';

/** What the slips are about: the day, and the one بکری or the chosen buyers when the print is narrower. */
export type DaySlipsScope = Pick<DaySlipsInput, 'date' | 'bikri_id' | 'bikri_ids' | 'buyer_ids'>;

/** The prints of the menu: the buyer slips, the seller bills, and the whole sale print. */
export type SlipsPreviewVariant = DaySlipsVariant | 'sale';

/** What the preview holds: the slips, or the two tables of the whole sale print. */
type Printed = { kind: 'slips'; data: DaySlipsPrintData } | { kind: 'sale'; data: SalePrintData };

/** How many things the print has to show: the slips of the variant, or the rows of the sale tables. */
function countOf(variant: SlipsPreviewVariant, printed: Printed | undefined): number {
  if (printed === undefined) return 0;
  if (printed.kind === 'sale')
    return printed.data.sellers.rows.length + printed.data.buyers.rows.length;
  return variant === 'seller-bills' ? printed.data.sellers.length : printed.data.buyers.length;
}

const LABEL_ID: Record<SlipsPreviewVariant, string> = {
  parchi: 'print-parchi-label',
  'seller-bills': 'print-seller-bills-label',
  sale: 'print-sale-label',
};

const EMPTY_ID: Record<SlipsPreviewVariant, string> = {
  parchi: 'print-parchi-empty',
  'seller-bills': 'print-seller-bills-empty',
  sale: 'print-sale-empty',
};

interface DaySlipsPreviewProps {
  variant: SlipsPreviewVariant;
  scope: DaySlipsScope;
  onBack: () => void;
}

export function DaySlipsPreview({ variant, scope, onBack }: DaySlipsPreviewProps) {
  const appLanguage = useLanguage();
  const [language, setLanguage] = useState<Language>(appLanguage);
  const [paper, setPaper] = useState<Paper>('a4');
  const input: DaySlipsInput = {
    ...scope,
    // The buyer slips are the buyers only, the seller bills the sellers only; the whole sale print has both.
    sections: variant === 'parchi' ? 'buyers' : variant === 'seller-bills' ? 'sellers' : 'both',
    language,
    paper,
  };
  // A print is a snapshot of what is on screen: it is read again for a new language or paper only.
  const query = useQuery({
    queryKey: ['day-slips-print', variant, scope, language, paper],
    queryFn: async (): Promise<Printed> => {
      if (variant === 'sale') return { kind: 'sale', data: await reportsSalePrintData(input) };
      const data =
        variant === 'parchi'
          ? await reportsBuyerParchiPrintData(input)
          : await reportsSellerBillsPrintData(input);
      return { kind: 'slips', data };
    },
    staleTime: Infinity,
    gcTime: 0,
  });
  const job = usePrintJob((to) => {
    const send = { slips: input, to, copies: null };
    if (variant === 'parchi') return reportsBuyerParchiPrint(send);
    return variant === 'seller-bills' ? reportsSellerBillsPrint(send) : reportsSalePrint(send);
  });

  const count = countOf(variant, query.data);
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
        title={<Localized id={EMPTY_ID[variant]} />}
        help={<Localized id="report-empty-help" />}
      />
    );
  } else {
    body =
      query.data.kind === 'sale' ? (
        <SalePrintDocument data={query.data.data} />
      ) : (
        <DaySlipsDocument
          data={query.data.data}
          variant={variant === 'sale' ? 'parchi' : variant}
        />
      );
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
      title={<Localized id={LABEL_ID[variant]} />}
    >
      {body}
    </PrintPreviewFrame>
  );
}
