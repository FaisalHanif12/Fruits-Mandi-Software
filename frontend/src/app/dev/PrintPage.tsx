// Development page /dev/print (plans 14 and 17e): the printed documents with the canned numbers of the
// PRD worked day, so they can be opened in a plain browser tab and photographed in both languages
// on A4, A5 and the 80 mm roll, with the print style on (browser "emulate print media"). The flags in
// the hash choose the document: `?doc=seller_bill|buyer_slips|collection_list&language=
// ur|en&paper=a4|a5|mm80&window&cancelled&long=15&lines=60&contacts&mark=RH&grade`. `grade`
// puts a quality on the lines, First, Second, Third in turn (plan 58). The prints of plan 78 are
// `?doc=parchi|seller-bills|sale|adhari` (with `named` the slips carry the party's name in their title).
// `contacts` and
// `mark` set the firm's contact persons and trade mark in the header. Never shipped: the route
// exists only when import.meta.env.DEV is true (app/router.tsx).

import { useRouterState } from '@tanstack/react-router';
import {
  collectionPrintData,
  frameOf,
  longPrintData,
  manyLinesData,
  printDataFor,
} from '../../../tests/helpers/print-data';
import { adhariPrintData, daySlipsData, salePrintData } from '../../../tests/helpers/day-slips';
import { BikriPrintDocument } from '../../features/bikri/print/BikriPrintDocument';
import {
  DaySlipsDocument,
  ReportPrintDocument,
  SalePrintDocument,
  printTableOf,
} from '../../features/reports';
import { CollectionListTemplate } from '../../features/vouchers/print/CollectionListTemplate';
import type { PrintDoc } from '../../bindings/PrintDoc';
import type { Paper } from '../../bindings/Paper';

const DOCS: readonly PrintDoc[] = ['seller_bill', 'buyer_slips', 'collection_list'];
const PAPER_VALUES: readonly Paper[] = ['a4', 'a5', 'mm80'];

export function PrintPage() {
  const searchStr = useRouterState({ select: (state) => state.location.searchStr });
  const search = new URLSearchParams(searchStr);
  const doc = DOCS.find((candidate) => candidate === search.get('doc')) ?? 'seller_bill';
  const language = search.get('language') === 'en' ? 'en' : 'ur';
  const paper = PAPER_VALUES.find((candidate) => candidate === search.get('paper'));
  // The prints of plan 78: the slips of a day, the whole sale print and ادھاری تفصیل.
  const kind = search.get('doc');
  const asWindow = search.has('window');
  if (kind === 'parchi' || kind === 'seller-bills') {
    // The backend's own answer of the worked day: one slip for each buyer (ارشد, اکرم, شاہد), each titled with
    // its party's name. `?many` repeats the first slip seven times to show how a page fills.
    const plain = daySlipsData(language, kind, paper ?? 'a4');
    // `contacts` puts the firm's owners and munshi in the header, as Settings would; `phones=4` gives
    // the firm all 4 phones, the tallest header.
    const phones = ['0300-1112233', '0321-4455667', '0333-7788990', '0345-2233445'].slice(
      0,
      search.get('phones') === '4' ? 4 : 2,
    );
    const slips = search.has('contacts')
      ? {
          ...plain,
          frame: {
            ...plain.frame,
            header: {
              ...plain.frame.header,
              firm: { ...plain.frame.header.firm, phones },
              contacts: frameOf(language, '', [], { contacts: true }).header.contacts,
            },
          },
        }
      : plain;
    const first = slips.buyers[0];
    const filled =
      search.has('many') && first !== undefined
        ? { ...slips, buyers: Array.from({ length: 7 }, () => first) }
        : slips;
    return <DaySlipsDocument data={filled} variant={kind} applyRoot={asWindow} />;
  }
  if (kind === 'sale') {
    return <SalePrintDocument data={salePrintData(language, paper ?? 'a4')} applyRoot={asWindow} />;
  }
  if (kind === 'adhari') {
    const data = adhariPrintData(language, paper ?? 'a4');
    return (
      <ReportPrintDocument
        data={printTableOf(data, 'R5')}
        frame={data.frame}
        applyRoot={asWindow}
      />
    );
  }
  const mark = search.get('mark');
  const options = {
    language,
    cancelled: search.has('cancelled'),
    contacts: search.has('contacts'),
    ...(mark === null || mark === '' ? {} : { tradeMark: mark }),
    ...(paper === undefined ? {} : { paper }),
  } as const;
  if (doc === 'collection_list') {
    return (
      <CollectionListTemplate
        data={collectionPrintData(options)}
        applyRoot={search.has('window')}
      />
    );
  }
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
  // `?grade` puts a quality on the lines, one after the other: First, Second, Third (plan 58).
  const GRADES = ['first', 'second', 'third'] as const;
  const shown = search.has('grade')
    ? {
        ...data,
        pages: data.pages.map((page) => ({
          ...page,
          lines: page.lines.map((line, index) => ({
            ...line,
            grade: GRADES[index % 3] ?? 'first',
          })),
        })),
      }
    : data;
  return <BikriPrintDocument data={shown} applyRoot={search.has('window')} />;
}
