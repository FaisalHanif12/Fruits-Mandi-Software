// A view of the analytics on screen as the hidden print window draws it (PRD FR-18.4, FR-19.13): it
// reads the print data and the shared header and footer together, draws the page, and says when it is
// ready. The four states are the shared `PrintView`'s: loading, an error with a retry, nothing to
// print, and the document.

import type { AnalyticsPrintInput } from '../../../bindings/AnalyticsPrintInput';
import { PrintView, type PrintReady } from '../../../shared/print';
import { ReportPrintDocument } from '../../reports';
import { measureOfFilters } from '../filters';
import { useAnalyticsPrintBundle } from '../hooks';
import { AnalyticsPrintCharts } from './AnalyticsPrintCharts';
import { printTableOf } from './print-table';

interface AnalyticsPrintViewProps {
  input: AnalyticsPrintInput;
  onReady?: (ready: PrintReady) => void;
  onFailed?: (codes: string[]) => void;
}

export function AnalyticsPrintView({ input, onReady, onFailed }: AnalyticsPrintViewProps) {
  const query = useAnalyticsPrintBundle(input);
  // The print data names the measure in its header facts; the chart and the row total need its key (PRD FR-18.10).
  const measure = measureOfFilters(input.filters);
  return (
    <PrintView
      query={query}
      describe={({ data, frame }) => ({
        empty: data.rows.length === 0,
        sample: `${frame.header.firm.name} ${frame.header.firm.mandi}`,
      })}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={({ data, frame }, ref) => (
        <ReportPrintDocument
          data={printTableOf(data, measure)}
          frame={frame}
          applyRoot
          figures={<AnalyticsPrintCharts data={data} measure={measure} />}
          ref={ref}
        />
      )}
    />
  );
}
