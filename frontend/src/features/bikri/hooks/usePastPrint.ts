// The reads and the print job of the print of پچھلی بکری (plan 72 P8). The print data and the shared
// header and footer are read together, so the hidden print window says "ready" only when both are there.
// A print is a snapshot: it is never read again by itself and never kept.

import { useMutation, useQuery } from '@tanstack/react-query';
import { reportsPrintHeader } from '../../reports';
import { bikriListPrint, bikriListPrintData } from '../past-print-api';
import type { BikriListPrintInput } from '../../../bindings/BikriListPrintInput';
import type { PrintBikriListInput } from '../../../bindings/PrintBikriListInput';

/** The print data of the list and its header and footer, in the print language of the input. */
export function usePastPrintBundle(input: BikriListPrintInput | null) {
  return useQuery({
    queryKey: ['bikri', 'past-print-bundle', input],
    queryFn: async () => {
      if (input === null) throw new Error('no print input');
      const data = await bikriListPrintData(input);
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

/** Sends the list to the printer or saves it as a PDF. The caller shows the outcome or the error. */
export function usePrintPastList() {
  return useMutation({ mutationFn: (input: PrintBikriListInput) => bikriListPrint(input) });
}
