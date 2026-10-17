// The address of the hidden print window: `#/print?bikri=<id>&doc=<doc>&language=<ur|en>&job=<job>
// [&buyer=<id>]` (plan 14 contract 7). The window is opened by the backend, so every value is read
// as plain text and checked here. A value that is not right is a failed print, never a guess.

import type { BikriPrintDataInput } from '../../../bindings/BikriPrintDataInput';
import type { PrintDoc } from '../../../bindings/PrintDoc';
import type { Language } from '../../../i18n';
import { jobOf, positiveInt } from '../../../shared/print';

export { jobOf };

export interface PrintParams {
  /** The job name; the events are `print-ready-<job>` and `print-error-<job>`. */
  job: string;
  input: BikriPrintDataInput;
}

const DOCS: readonly PrintDoc[] = ['seller_bill', 'buyer_slips', 'pukhta_slip'];
const LANGUAGE_VALUES: readonly Language[] = ['ur', 'en'];

/** The parameters of the print window, or null when the address is not a valid print address. */
export function parsePrintParams(searchStr: string): PrintParams | null {
  const search = new URLSearchParams(searchStr);
  const job = jobOf(searchStr);
  const bikriId = positiveInt(search.get('bikri'));
  const doc = DOCS.find((candidate) => candidate === search.get('doc'));
  const language = LANGUAGE_VALUES.find((candidate) => candidate === search.get('language'));
  if (job === null || bikriId === null || doc === undefined || language === undefined) return null;
  const buyerText = search.get('buyer');
  const buyer = positiveInt(buyerText);
  if (buyerText !== null && buyer === null) return null;
  return {
    job,
    input: { bikri_id: bikriId, doc, language, only_buyer_id: buyer },
  };
}
