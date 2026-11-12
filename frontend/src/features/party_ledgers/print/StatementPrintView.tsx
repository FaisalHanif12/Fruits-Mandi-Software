// The statement print as the hidden print window draws it (plan 74 T3): it reads the print data and the
// shared header and footer together, draws the page, and says when it is ready. The four states are the
// shared `PrintView`'s: loading, an error with a retry, nothing to print, and the document.

import { PrintView, type PrintDescription, type PrintReady } from '../../../shared/print';
import { hasNothingToPrint } from './statement-empty';
import { useStatementPrintBundle } from './statement-hooks';
import { StatementPrintDocument } from './StatementPrintDocument';
import type { StatementPrintInput } from './statement-params';

interface StatementPrintViewProps {
  /** The party, the period, the print language and the paper of the address. */
  input: StatementPrintInput;
  onReady?: (ready: PrintReady) => void;
  onFailed?: (codes: string[]) => void;
}

export function StatementPrintView({ input, onReady, onFailed }: StatementPrintViewProps) {
  const { paper, ...read } = input;
  const query = useStatementPrintBundle(read);
  return (
    <PrintView
      query={query}
      describe={({ data, frame }): PrintDescription => ({
        empty: hasNothingToPrint(data),
        // What the print window loads the fonts for: the firm of the header and the party.
        sample: `${frame.header.firm.name} ${frame.header.firm.mandi} ${data.party.name}`,
      })}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={({ data, frame }, ref) => (
        // The paper is the one of the address: the page is laid out for it, whatever the data assumed.
        <StatementPrintDocument data={{ ...data, paper }} frame={frame} applyRoot ref={ref} />
      )}
    />
  );
}
