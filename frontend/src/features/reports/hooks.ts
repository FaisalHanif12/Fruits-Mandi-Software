// The print commands as hooks (Code Standards F6).

import { useMutation, useQuery } from '@tanstack/react-query';
import type { BikriPrintDataInput } from '../../bindings/BikriPrintDataInput';
import type { PrintInput } from '../../bindings/PrintInput';
import { reportsBikriPrintData, reportsListPrinters, reportsPrint } from './api';

/** The print data of one document. A print is a snapshot, so it is never read again by itself. */
export function useBikriPrintData(input: BikriPrintDataInput | null) {
  return useQuery({
    queryKey: ['reports', 'bikri-print-data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsBikriPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** The printers of this computer (Windows). An empty list is an answer, not an error. */
export function usePrinters() {
  return useQuery({ queryKey: ['reports', 'printers'], queryFn: reportsListPrinters });
}

/** Print or save as PDF. The caller shows the outcome or the error. */
export function usePrintDocs() {
  return useMutation({ mutationFn: (input: PrintInput) => reportsPrint(input) });
}
