// The quick action buttons of the dashboard (PRD 9.3, NFR-USE-9): a new بکری, وصولی, ادائیگی, پیشگی
// and خرچہ, and the day book. Each shows its key (PRD FR-7.4); the keys themselves work from every
// screen (the app shell registers them). Plain buttons of the shared kit.

import { Localized } from '@fluent/react';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button } from '../../../shared/ui';
import { useOpenDestination } from '../destinations';

export function QuickActions() {
  const open = useOpenDestination();
  return (
    <div role="group" aria-labelledby="dashboard-quick-title" className="flex flex-wrap gap-2">
      <Button
        variant="primary"
        shortcut={SHORTCUTS.newBikri}
        onClick={() => {
          open({ kind: 'bikri-book' });
        }}
      >
        <Localized id="dashboard-new-bikri" />
      </Button>
      <Button
        shortcut={SHORTCUTS.newReceipt}
        onClick={() => {
          open({ kind: 'receipt' });
        }}
      >
        <Localized id="dashboard-new-receipt" />
      </Button>
      <Button
        shortcut={SHORTCUTS.newPayment}
        onClick={() => {
          open({ kind: 'payment' });
        }}
      >
        <Localized id="dashboard-new-payment" />
      </Button>
      <Button
        shortcut={SHORTCUTS.newAdvance}
        onClick={() => {
          open({ kind: 'advance' });
        }}
      >
        <Localized id="dashboard-new-advance" />
      </Button>
      <Button
        shortcut={SHORTCUTS.newExpense}
        onClick={() => {
          open({ kind: 'shop-expenses' });
        }}
      >
        <Localized id="dashboard-new-expense" />
      </Button>
    </div>
  );
}

/** The day book, at the end of the greeting's row: the five buttons above are then one row (plan 38). */
export function DayBookButton() {
  const open = useOpenDestination();
  return (
    <Button
      onClick={() => {
        open({ kind: 'report', id: 'R11' });
      }}
    >
      <Localized id="dashboard-open-daybook" />
    </Button>
  );
}
