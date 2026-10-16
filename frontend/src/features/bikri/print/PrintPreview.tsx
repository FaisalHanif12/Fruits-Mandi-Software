// The in-app preview of the printed documents (plan 14): the same page the hidden print window
// draws, inside the app, one document after another. It keeps the app's own language: only the
// document is in the print language. Used where printing is not available (not Windows), and
// whenever the user wants to see a bill before it prints.

import type { Language } from '../../../bindings/Language';
import type { PrintDoc } from '../../../bindings/PrintDoc';
import { BikriPrintView } from './BikriPrintView';

interface PrintPreviewProps {
  bikriId: number;
  docs: readonly PrintDoc[];
  language: Language;
  onlyBuyerId: number | null;
}

export function PrintPreview({ bikriId, docs, language, onlyBuyerId }: PrintPreviewProps) {
  return (
    <div className="flex flex-col gap-4">
      {docs.map((doc) => (
        <BikriPrintView
          key={`${doc}-${language}`}
          mode="preview"
          input={{
            bikri_id: bikriId,
            doc,
            language,
            only_buyer_id: doc === 'buyer_slips' ? onlyBuyerId : null,
          }}
        />
      ))}
    </div>
  );
}
