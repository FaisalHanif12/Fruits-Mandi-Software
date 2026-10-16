// A whole printed document: every sheet of one print job, in the print language. The templates
// of the modules (features/<module>/print/) choose the columns; everything else is here, so every
// print of the app looks like one family (Code Standards F12).
//
// `lang`, `dir`, the paper class and the digit style are attributes of the root. print.css reads
// them: the `page` property picks the one @page rule of the paper, the tokens of `:lang()` pick
// the font and the size steps, `data-digits` puts Naskh first for numbers.

import { useLocalization } from '@fluent/react';
import { useEffect, type Ref } from 'react';
import { applyPrintRoot, footerContent } from './footer';
import { PrintProvider } from './PrintProvider';
import { PrintSheet } from './PrintSheet';
import {
  FOOTER_LABEL_IDS,
  LAYOUT_IDS,
  type PrintColumn,
  type PrintData,
  type PrintPage,
} from './types';

export interface PrintDocumentProps {
  data: PrintData;
  /** The goods columns of each page. */
  columnsOf: (page: PrintPage, data: PrintData) => readonly PrintColumn[];
  /**
   * True in the hidden print window: the root element then takes the print language, direction
   * and footer, so the page margin boxes (which inherit from the root) are right. False in the
   * in-app preview, which must not change the app's own language.
   */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

function PrintDocumentBody({ data, columnsOf, applyRoot = false, ref }: PrintDocumentProps) {
  const { l10n } = useLocalization();
  const words = {
    printed: l10n.getString(FOOTER_LABEL_IDS.printed),
    page: l10n.getString(FOOTER_LABEL_IDS.page),
    of: l10n.getString(FOOTER_LABEL_IDS.of),
  };
  const footer = footerContent(words, data.printed_at, data.urdu_digits, data.paper !== 'mm80');
  useEffect(() => {
    if (!applyRoot) return undefined;
    return applyPrintRoot({ language: data.language, dir: data.dir, footer });
  }, [applyRoot, data.language, data.dir, footer]);
  return (
    <div
      ref={ref}
      className="print-doc"
      lang={data.language}
      dir={data.dir}
      data-paper={data.paper}
      data-digits={data.urdu_digits ? 'urdu' : 'latin'}
      data-window={applyRoot ? 'true' : undefined}
    >
      {data.pages.length === 0 ? (
        <p className="print-empty">{l10n.getString(LAYOUT_IDS.empty)}</p>
      ) : (
        data.pages.map((page, index) => (
          <PrintSheet key={index} data={data} page={page} columns={columnsOf(page, data)} />
        ))
      )}
    </div>
  );
}

export function PrintDocument(props: PrintDocumentProps) {
  return (
    <PrintProvider language={props.data.language}>
      <PrintDocumentBody {...props} />
    </PrintProvider>
  );
}
