// The quick action buttons of the dashboard (PRD 9.3, NFR-USE-9): a new بکری, وصولی, ادائیگی, پیشگی
// and خرچہ, then پچھلی بکری, پارٹی کھاتے and آج کی نقدی (plan 93 T5, plan 100 B: they open their screens and have no
// key of their own, so they show no key), and the day book. Each shows its key where it has one (PRD FR-7.4); the keys themselves work
// from every screen (the app shell registers them). Plain buttons of the shared kit, at their natural width in ONE
// row: the ones that do not fit the window go behind "More" at the end of the row (plan 93 T6, Umar 22-11-2026).

import { Localized, useLocalization } from '@fluent/react';
import { SHORTCUTS } from '../../../shared/keyboard';
import type { Shortcut } from '../../../shared/keyboard';
import { Button, OverflowRow, type OverflowItem } from '../../../shared/ui';
import { useOpenDestination, type Destination } from '../destinations';

interface QuickAction {
  id: string;
  /** The Fluent message ID of the words of the button. */
  labelId: string;
  to: Destination;
  /** The key of the action, where there is one: the screens at the end have none. */
  shortcut?: Shortcut;
  primary?: boolean;
}

/** In this order (PRD 9.3): the five entries, then the two screens. */
const ACTIONS: readonly QuickAction[] = [
  {
    id: 'bikri',
    labelId: 'dashboard-new-bikri',
    to: { kind: 'bikri-book' },
    shortcut: SHORTCUTS.newBikri,
    primary: true,
  },
  {
    id: 'receipt',
    labelId: 'dashboard-new-receipt',
    to: { kind: 'receipt' },
    shortcut: SHORTCUTS.newReceipt,
  },
  {
    id: 'payment',
    labelId: 'dashboard-new-payment',
    to: { kind: 'payment' },
    shortcut: SHORTCUTS.newPayment,
  },
  {
    id: 'advance',
    labelId: 'dashboard-new-advance',
    to: { kind: 'advance' },
    shortcut: SHORTCUTS.newAdvance,
  },
  {
    id: 'expense',
    labelId: 'dashboard-new-expense',
    to: { kind: 'shop-expenses' },
    shortcut: SHORTCUTS.newExpense,
  },
  // No key exists for these two, so none is shown (PRD FR-7.4).
  { id: 'past-sales', labelId: 'dashboard-open-past-sales', to: { kind: 'bikri-today' } },
  { id: 'party-ledgers', labelId: 'dashboard-open-party-ledgers', to: { kind: 'party-ledgers' } },
  // The نقدی page lost its tile to کل مالیت and keeps its way in here (plan 100 B).
  { id: 'cash-sales', labelId: 'dashboard-open-cash-sales', to: { kind: 'cash-sales' } },
];

export function QuickActions() {
  const { l10n } = useLocalization();
  const open = useOpenDestination();
  const items: OverflowItem[] = ACTIONS.map((action) => ({
    id: action.id,
    label: l10n.getString(action.labelId),
    onSelect: () => {
      open(action.to);
    },
    node: (
      <Button
        {...(action.primary === true ? { variant: 'primary' as const } : {})}
        {...(action.shortcut === undefined ? {} : { shortcut: action.shortcut })}
        onClick={() => {
          open(action.to);
        }}
      >
        <Localized id={action.labelId} />
      </Button>
    ),
  }));
  return <OverflowRow items={items} aria-labelledby="dashboard-quick-title" />;
}

/** The day book, at the end of the greeting's row: the buttons above are then one row (plan 38). */
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
