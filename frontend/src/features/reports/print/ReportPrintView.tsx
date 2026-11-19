// A report on screen as the hidden print window and the in-app preview draw it (PRD FR-19.3, FR-19.13):
// it reads the print data and the shared header and footer together, draws the page, and says when it
// is ready. It is the body of the hidden print window (mode `window`), so what the Owner previews is
// what is printed. The four states are the shared `PrintView`'s: loading, an error with a retry,
// nothing to print, and the document.

import { PrintView, type PrintDescription, type PrintReady } from '../../../shared/print';
import type { ReportPrintInput } from '../../../bindings/ReportPrintInput';
import { useReportPrintBundle, useSlipsPrintBundle } from '../hooks';
import { BuyerBillsPrintDocument } from './BuyerBillsPrintDocument';
import { ReportPrintDocument } from './ReportPrintDocument';

interface ReportPrintViewProps {
  input: ReportPrintInput;
  /** True for the slips of all the credit buyers of R5. */
  slips: boolean;
  onReady?: (ready: PrintReady) => void;
  onFailed?: (codes: string[]) => void;
}

/** What the print window loads the fonts for: the firm and its contact persons of the header. */
function sampleOf(frame: { header: { firm: { name: string; mandi: string } } }): string {
  return `${frame.header.firm.name} ${frame.header.firm.mandi}`;
}

function ReportBody({ input, onReady, onFailed }: Omit<ReportPrintViewProps, 'slips'>) {
  const query = useReportPrintBundle(input);
  return (
    <PrintView
      query={query}
      describe={({ data, frame }): PrintDescription => ({
        empty: data.rows.length === 0 && data.summary.length === 0,
        sample: sampleOf(frame),
      })}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={({ data, frame }, ref) => (
        <ReportPrintDocument data={data} frame={frame} applyRoot ref={ref} />
      )}
    />
  );
}

function SlipsBody({ input, onReady, onFailed }: Omit<ReportPrintViewProps, 'slips'>) {
  const query = useSlipsPrintBundle(input);
  return (
    <PrintView
      query={query}
      describe={({ data, frame }): PrintDescription => ({
        empty: data.slips.length === 0,
        sample: sampleOf(frame),
      })}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={({ data, frame }, ref) => (
        <BuyerBillsPrintDocument data={data} frame={frame} applyRoot ref={ref} />
      )}
    />
  );
}

export function ReportPrintView({ slips, ...rest }: ReportPrintViewProps) {
  return slips ? <SlipsBody {...rest} /> : <ReportBody {...rest} />;
}
