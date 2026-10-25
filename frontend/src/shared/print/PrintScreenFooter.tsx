// The footer line of the in-app preview. On paper the footer is the page margin box (print.css,
// footer.ts); on screen a line under each sheet says the same: the print time with its label.

import { useLocalization } from '@fluent/react';
import { Num } from '../ui/Num';
import type { PrintFooter } from './types';

export function PrintScreenFooter({ footer }: { footer: PrintFooter }) {
  const { l10n } = useLocalization();
  return (
    <p className="print-screen-footer">
      {l10n.getString(footer.printed_label_id)} <Num>{footer.printed_at}</Num>
    </p>
  );
}
