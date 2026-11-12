// The reads and the print job of the statement print (plan 74 T3). The print data and the shared header and
// footer are read together, so the hidden print window says "ready" only when both are there. A print is a
// snapshot: it is never read again by itself and never kept.

import { useMutation, useQuery } from '@tanstack/react-query';
import { reportsPrintHeader } from '../../reports';
import { partyLedgersStatementPrint, partyLedgersStatementPrintData } from './statement-api';
import type { PrintStatementInput } from '../../../bindings/PrintStatementInput';
import type { StatementPrintDataInput } from '../../../bindings/StatementPrintDataInput';

/** The statement and its header and footer, in the print language of the input. */
export function useStatementPrintBundle(input: StatementPrintDataInput | null) {
  return useQuery({
    queryKey: ['party_ledgers', 'statement-print-bundle', input],
    queryFn: async () => {
      if (input === null) throw new Error('no print input');
      const data = await partyLedgersStatementPrintData(input);
      const frame = await reportsPrintHeader({
        language: input.language,
        title_id: data.title_id,
        facts: data.filters,
      });
      return { language: frame.language, data, frame };
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** Sends the statement to the printer or saves it as a PDF. The caller shows the outcome or the error. */
export function usePrintStatement() {
  return useMutation({
    mutationFn: (input: PrintStatementInput) => partyLedgersStatementPrint(input),
  });
}
