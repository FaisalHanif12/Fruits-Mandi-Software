// The print commands as hooks (Code Standards F6).

import { useMutation, useQuery } from '@tanstack/react-query';
import type { BikriPrintDataInput } from '../../bindings/BikriPrintDataInput';
import type { PrintInput } from '../../bindings/PrintInput';
import type { PrintVoucherInput } from '../../bindings/PrintVoucherInput';
import type { VoucherPrintDataInput } from '../../bindings/VoucherPrintDataInput';
import {
  reportsBikriPrintData,
  reportsListPrinters,
  reportsPrint,
  reportsPrintVoucher,
  reportsVoucherPrintData,
} from './api';

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

/** The print data of the receipt slip of one saved voucher. A print is a snapshot, like the above. */
export function useVoucherPrintData(input: VoucherPrintDataInput | null) {
  return useQuery({
    queryKey: ['reports', 'voucher-print-data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return reportsVoucherPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** Print the receipt slip of a voucher, or save it as PDF. The caller shows the outcome or the error. */
export function usePrintVoucher() {
  return useMutation({ mutationFn: (input: PrintVoucherInput) => reportsPrintVoucher(input) });
}
