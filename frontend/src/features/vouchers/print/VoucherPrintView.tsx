// The receipt slip of a saved voucher on screen: reads its print data and draws the template. It is
// the body of the hidden print window (mode `window`) and also the in-app preview (mode `preview`).
// It calls ONE command, `reports_voucher_print_data` (the capability of the hidden window allows
// no other for a voucher).

import type { VoucherPrintDataInput } from '../../../bindings/VoucherPrintDataInput';
import { PrintView, describePrintData, type PrintReady } from '../../../shared/print';
import { useVoucherPrintData } from '../../reports';
import { ReceiptSlipTemplate } from './ReceiptSlipTemplate';

interface VoucherPrintViewProps {
  input: VoucherPrintDataInput;
  /** `window`: the hidden print window. `preview`: inside the app, which keeps its own language. */
  mode: 'window' | 'preview';
  onReady?: (ready: PrintReady) => void;
  onFailed?: (codes: string[]) => void;
}

export function VoucherPrintView({ input, mode, onReady, onFailed }: VoucherPrintViewProps) {
  const query = useVoucherPrintData(input);
  return (
    <PrintView
      query={query}
      describe={describePrintData}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={(data, ref) => (
        <ReceiptSlipTemplate ref={ref} data={data} applyRoot={mode === 'window'} />
      )}
    />
  );
}
