// The root of every printed document (PRD FR-19.13, FR-7.36): the print language's Fluent bundle,
// `lang`, `dir`, the paper class and the digit style as attributes, and, in the hidden print
// window, the footer of the page margin boxes. The documents of a بکری or a voucher
// (`PrintDocument`), the collection list and the reports of D5 all sit inside it, so every print
// of the app gets its language, direction and footer in one place. It is given the frame the
// backend built (`language`, `dir`, `urdu_digits`, `footer`) and the paper, and draws its children.
//
// `applyRoot` is true only in the hidden print window: the page margin boxes inherit from the
// document element, which then takes the print language, direction and footer. The in-app preview
// must not change the app's own language (Code Standards F37).

import { useLocalization } from '@fluent/react';
import { useEffect, type ReactNode, type Ref } from 'react';
import { applyPrintRoot, footerContent } from './footer';
import { PrintProvider } from './PrintProvider';
import type { Paper, PrintFrame } from './types';

export interface PrintRootProps {
  frame: Pick<PrintFrame, 'language' | 'dir' | 'urdu_digits' | 'footer'>;
  paper: Paper;
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
  children: ReactNode;
}

function PrintRootBody({ frame, paper, applyRoot = false, ref, children }: PrintRootProps) {
  const { l10n } = useLocalization();
  // The footer words are the ids the backend sends with the footer, resolved in the print language.
  const words = {
    printed: l10n.getString(frame.footer.printed_label_id),
    page: l10n.getString(frame.footer.page_label_id),
    of: l10n.getString(frame.footer.of_label_id),
  };
  const footer = footerContent(words, frame.footer.printed_at, frame.urdu_digits, paper !== 'mm80');
  useEffect(() => {
    if (!applyRoot) return undefined;
    return applyPrintRoot({ language: frame.language, dir: frame.dir, footer });
  }, [applyRoot, frame.language, frame.dir, footer]);
  return (
    <div
      ref={ref}
      className="print-doc"
      lang={frame.language}
      dir={frame.dir}
      data-paper={paper}
      data-digits={frame.urdu_digits ? 'urdu' : 'latin'}
      data-window={applyRoot ? 'true' : undefined}
    >
      {children}
    </div>
  );
}

export function PrintRoot(props: PrintRootProps) {
  return (
    <PrintProvider language={props.frame.language}>
      <PrintRootBody {...props} />
    </PrintProvider>
  );
}
