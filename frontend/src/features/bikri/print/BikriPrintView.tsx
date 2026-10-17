// One printed document of a saved بکری on screen: reads its print data and draws the template (the
// shared PrintView does the waiting, the readiness signal and the four states). It is the body of
// the hidden print window (mode `window`) and also the in-app preview (mode `preview`), so what the
// user previews is exactly what is printed.
//
// It calls ONE command, `reports_bikri_print_data` (the capability of the hidden window allows no
// other for a بکری).

import type { BikriPrintDataInput } from '../../../bindings/BikriPrintDataInput';
import { PrintView, type PrintReady } from '../../../shared/print';
import { useBikriPrintData } from '../../reports';
import { BikriPrintDocument } from './BikriPrintDocument';

interface BikriPrintViewProps {
  input: BikriPrintDataInput;
  /** `window`: the hidden print window. `preview`: inside the app, which keeps its own language. */
  mode: 'window' | 'preview';
  /** The document is drawn, its fonts are loaded and its layout is final. Called once. */
  onReady?: (ready: PrintReady) => void;
  /** The print data could not be read. The codes of the error. Called once. */
  onFailed?: (codes: string[]) => void;
}

export function BikriPrintView({ input, mode, onReady, onFailed }: BikriPrintViewProps) {
  const query = useBikriPrintData(input);
  return (
    <PrintView
      query={query}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={(data, ref) => (
        <BikriPrintDocument ref={ref} data={data} applyRoot={mode === 'window'} />
      )}
    />
  );
}
