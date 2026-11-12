// The button "اسٹیٹمنٹ پرنٹ" for the top of a party's statement (plan 74 T3, PRD FR-12.47), and its box. The
// page of the statement places this one component among its actions and gives it the party: the box and the
// preview ask their own commands, so nothing of the page's window or range is needed. F9 opens it too.

import { Localized } from '@fluent/react';
import { Printer } from 'lucide-react';
import { useState } from 'react';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import { Button } from '../../../shared/ui';
import { StatementPrintBox } from './StatementPrintBox';

interface StatementPrintActionProps {
  accountId: number;
}

export function StatementPrintAction({ accountId }: StatementPrintActionProps) {
  const [open, setOpen] = useState(false);
  useShortcut(SHORTCUTS.print, () => {
    setOpen(true);
  });
  return (
    <>
      <Button
        shortcut={SHORTCUTS.print}
        onClick={() => {
          setOpen(true);
        }}
      >
        <Printer aria-hidden="true" className="size-icon-md" />
        <Localized id="statement-print-button" />
      </Button>
      <StatementPrintBox open={open} onOpenChange={setOpen} accountId={accountId} />
    </>
  );
}
