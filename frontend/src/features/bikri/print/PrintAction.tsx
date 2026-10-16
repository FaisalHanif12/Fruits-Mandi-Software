// The "Print F9" button of the sale book, on a new بکری and on a saved one (PRD FR-12.12, FR-7.4),
// and the F9 key. It opens the print box for the saved بکری the screen is about: on a saved one,
// that one; on a new, unsaved one, the one saved last (the way back to the bill just made). With
// none saved yet, it says so (the bill is made from saved data, never from what is typed).

import { Localized } from '@fluent/react';
import { Printer } from 'lucide-react';
import { useState } from 'react';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import { Button, showToast } from '../../../shared/ui';
import { PrintBox, type PrintTarget } from './PrintBox';

interface PrintActionProps {
  /** The saved بکری to print, or null when there is none (a new one, nothing saved yet). */
  target: PrintTarget | null;
  /** False while another dialog is open, so F9 does not stack a second one. */
  enabled?: boolean;
}

export function PrintAction({ target, enabled = true }: PrintActionProps) {
  const [open, setOpen] = useState(false);
  const request = () => {
    if (target === null) {
      showToast(
        'error',
        <Localized id="print-save-first">
          <span />
        </Localized>,
      );
      return;
    }
    setOpen(true);
  };
  useShortcut(SHORTCUTS.print, request, { enabled: enabled && !open });
  return (
    <>
      <Button shortcut={SHORTCUTS.print} onClick={request}>
        <Printer aria-hidden="true" className="size-icon-md" />
        <Localized id="print-box-print" />
      </Button>
      {target === null ? null : <PrintBox open={open} onOpenChange={setOpen} target={target} />}
    </>
  );
}
