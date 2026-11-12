// The print of پچھلی بکری as the hidden print window and the in-app preview draw it (plan 72 P8): it
// reads the print data and the shared header and footer together, draws the page, and says when it is
// ready. The four states are the shared `PrintView`'s: loading, an error with a retry, nothing to print,
// and the document.

import { PrintView, type PrintDescription, type PrintReady } from '../../../shared/print';
import { usePastPrintBundle } from '../hooks/usePastPrint';
import type { BikriListPrintInput } from '../../../bindings/BikriListPrintInput';
import { PastPrintDocument } from './PastPrintDocument';
import { pastPrintIsEmpty } from './past-print-table';

interface PastPrintViewProps {
  input: BikriListPrintInput;
  onReady?: (ready: PrintReady) => void;
  onFailed?: (codes: string[]) => void;
}

export function PastPrintView({ input, onReady, onFailed }: PastPrintViewProps) {
  const query = usePastPrintBundle(input);
  return (
    <PrintView
      query={query}
      describe={({ data, frame }): PrintDescription => ({
        empty: pastPrintIsEmpty(data),
        // What the print window loads the fonts for: the firm of the header.
        sample: `${frame.header.firm.name} ${frame.header.firm.mandi}`,
      })}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={({ data, frame }, ref) => (
        <PastPrintDocument data={data} frame={frame} applyRoot ref={ref} />
      )}
    />
  );
}
