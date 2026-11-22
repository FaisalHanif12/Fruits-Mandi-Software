// The "Print F9" button of the sale book, on a new بکری and on a saved one (PRD FR-12.12, FR-7.4),
// and the F9 key. It opens the print box for the saved بکری the screen is about: on a saved one,
// that one; on a new, unsaved one, the one saved last (the way back to the bill just made). With
// none saved yet, it says so (the bill is made from saved data, never from what is typed).
// F9 opens the print menu (buyer slips, seller bills, whole sale print; plan 78 A1). Next to it is the second
// print button, ادھاری تفصیل (R5) with Shift+F9 (plan 68, plan 78 A5): it prints only that, for the whole day.
// Both follow the same rule: a saved بکری.

import { Localized, useLocalization } from '@fluent/react';
import { FileText, Printer } from 'lucide-react';
import { useState } from 'react';
import { SHORTCUTS, useShortcut, type Shortcut } from '../../../shared/keyboard';
import { Button, showToast } from '../../../shared/ui';
import { useCompactControls } from '../components/compact-controls';
import { AdhariSheetBox } from './AdhariSheetBox';
import { PrintMenuBox } from './PrintMenuBox';
import type { PrintTarget } from './target';

interface PrintActionProps {
  /** The saved بکری to print, or null when there is none (a new one, nothing saved yet). */
  target: PrintTarget | null;
  /** False while another dialog is open, so F9 does not stack a second one. */
  enabled?: boolean;
}

export function PrintAction({ target, enabled = true }: PrintActionProps) {
  const { l10n } = useLocalization();
  // The row is short of room: the buttons show an icon and a word, and say their key in their tip and name.
  const compact = useCompactControls();
  const [open, setOpen] = useState(false);
  const [adhariOpen, setAdhariOpen] = useState(false);
  /** Opens a box for the saved بکری, or says to save first. */
  const requestOf = (show: () => void) => () => {
    if (target === null) {
      showToast(
        'error',
        <Localized id="print-save-first">
          <span />
        </Localized>,
      );
      return;
    }
    show();
  };
  const request = requestOf(() => {
    setOpen(true);
  });
  const requestAdhari = requestOf(() => {
    setAdhariOpen(true);
  });
  // One box at a time: a key never stacks a second one on the first.
  const free = enabled && !open && !adhariOpen;
  useShortcut(SHORTCUTS.print, request, { enabled: free });
  useShortcut(SHORTCUTS.printAdhari, requestAdhari, { enabled: free });
  // The tip and the name of a button in the short row: its words and its key, so the key is never lost with the chip.
  const named = (words: string, shortcut: Shortcut) => `${words} (${shortcut.label})`;
  return (
    <>
      {/* When the row of the sale book is short of room for its tabs, the key chip gives way and the print button
          shows its short word (`BikriToolbar`, plan 95); the key stays in the tip and the name. */}
      <Button
        shortcut={SHORTCUTS.print}
        tightInCompactRow
        onClick={request}
        {...(compact
          ? {
              title: named(l10n.getString('print-box-print'), SHORTCUTS.print),
              accessibleName: named(l10n.getString('print-box-print'), SHORTCUTS.print),
            }
          : {})}
      >
        <Printer aria-hidden="true" className="size-icon-md" />
        <Localized id="print-box-print">
          <span className="group-data-[compact]/toolbar:hidden" />
        </Localized>
        <Localized id="print-box-print-short">
          <span className="hidden group-data-[compact]/toolbar:inline" />
        </Localized>
      </Button>
      <Button
        shortcut={SHORTCUTS.printAdhari}
        tightInCompactRow
        onClick={requestAdhari}
        {...(compact
          ? {
              title: named(l10n.getString('report-r5-title'), SHORTCUTS.printAdhari),
              accessibleName: named(l10n.getString('report-r5-title'), SHORTCUTS.printAdhari),
            }
          : {})}
      >
        <FileText aria-hidden="true" className="size-icon-md" />
        <Localized id="report-r5-title">
          <span />
        </Localized>
      </Button>
      {target === null ? null : (
        <>
          <PrintMenuBox open={open} onOpenChange={setOpen} target={target} />
          <AdhariSheetBox open={adhariOpen} onOpenChange={setAdhariOpen} target={target} />
        </>
      )}
    </>
  );
}
