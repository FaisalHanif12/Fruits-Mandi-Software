// The "Print F9" button of the sale book, on a new بکری and on a saved one (PRD FR-12.12, FR-7.4),
// and the F9 key. It opens the print box for the saved بکری the screen is about: on a saved one,
// that one; on a new, unsaved one, the one saved last (the way back to the bill just made). With
// none saved yet, it says so (the bill is made from saved data, never from what is typed).
// Next to it is the second print button, ادھاری تفصیل (R5) with Shift+F9 (plan 68): the whole day's
// buyers by default, or this بکری only. It follows the same rule: a saved بکری.

import { Localized } from '@fluent/react';
import { FileText, Printer } from 'lucide-react';
import { useState } from 'react';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import { Button, showToast } from '../../../shared/ui';
import { AdhariPrintBox } from './AdhariPrintBox';
import { PrintBox, type PrintTarget } from './PrintBox';

interface PrintActionProps {
  /** The saved بکری to print, or null when there is none (a new one, nothing saved yet). */
  target: PrintTarget | null;
  /** False while another dialog is open, so F9 does not stack a second one. */
  enabled?: boolean;
}

export function PrintAction({ target, enabled = true }: PrintActionProps) {
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
  return (
    <>
      <Button shortcut={SHORTCUTS.print} onClick={request}>
        <Printer aria-hidden="true" className="size-icon-md" />
        <Localized id="print-box-print" />
      </Button>
      <Button shortcut={SHORTCUTS.printAdhari} onClick={requestAdhari}>
        <FileText aria-hidden="true" className="size-icon-md" />
        <Localized id="report-r5-title" />
      </Button>
      {target === null ? null : (
        <>
          <PrintBox open={open} onOpenChange={setOpen} target={target} />
          <AdhariPrintBox open={adhariOpen} onOpenChange={setAdhariOpen} target={target} />
        </>
      )}
    </>
  );
}
