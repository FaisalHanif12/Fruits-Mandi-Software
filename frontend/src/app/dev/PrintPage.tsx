// Development page /dev/print (plan 14): the three printed documents with the canned numbers of the
// PRD worked day, so they can be opened in a plain browser tab and photographed in both languages
// on A4, A5 and the 80 mm roll, with the print style on (browser "emulate print media"). The flags in
// the hash choose the document: `?doc=seller_bill|buyer_slips|pukhta_slip&language=ur|en
// &paper=a4|a5|mm80&window&digits&cancelled&long=15&lines=60`. Never shipped: the route exists only when
// import.meta.env.DEV is true (app/router.tsx).

import { useRouterState } from '@tanstack/react-router';
import { longPrintData, manyLinesData, printDataFor } from '../../../tests/helpers/print-data';
import { BikriPrintDocument } from '../../features/bikri/print/BikriPrintDocument';
import type { PrintDoc } from '../../bindings/PrintDoc';
import type { Paper } from '../../bindings/Paper';

const DOCS: readonly PrintDoc[] = ['seller_bill', 'buyer_slips', 'pukhta_slip'];
const PAPER_VALUES: readonly Paper[] = ['a4', 'a5', 'mm80'];

export function PrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const search = new URLSearchParams(searchStr);
  const doc = DOCS.find((candidate) => candidate === search.get('doc')) ?? 'seller_bill';
  const language = search.get('language') === 'en' ? 'en' : 'ur';
  const paper = PAPER_VALUES.find((candidate) => candidate === search.get('paper'));
  const options = {
    language,
    urduDigits: search.has('digits'),
    cancelled: search.has('cancelled'),
    ...(paper === undefined ? {} : { paper }),
  } as const;
  const long = Number(search.get('long'));
  const lines = Number(search.get('lines'));
  const data =
    Number.isInteger(long) && long > 0
      ? longPrintData(long, 12, options)
      : Number.isInteger(lines) && lines > 0
        ? manyLinesData(doc, lines, options)
        : printDataFor(doc, options);
  // `?window` shows the page as the hidden print window makes it: no ground and no frame, the html
  // language and direction of the print, and the page footer. Without it the sheets are shown on
  // a grey ground, as in the in-app preview.
  return <BikriPrintDocument data={data} applyRoot={search.has('window')} />;
}
