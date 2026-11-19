// The side menus of the بکری area (PRD FR-5.3) and of the روکڑ area (PRD 5, plan 17), one look
// (design system 6): 200 px, sunken ground. Each item has a small icon; the current item is a white card
// with primary text and a thin border, so it does not compete with the dark main menu (plan 75, item 3).
// Next to the main menu, so the arrow that points into the screen leads to the screen, and the other
// arrow leads back to the menu (PRD FR-7.17).

import { useLocalization } from '@fluent/react';
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  BookText,
  BookUser,
  Coins,
  FileSignature,
  Handshake,
  History,
  NotebookPen,
  NotebookTabs,
  Package,
  Receipt,
  Wallet,
  type LucideIcon,
} from 'lucide-react';
import { useFocusRegion, useRovingKeys } from '../../shared/keyboard';

export type ModuleMenuId =
  'bikri-book' | 'receipt' | 'payment' | 'advance' | 'today' | 'party-ledgers' | 'stock';

interface ModuleEntry {
  id: ModuleMenuId;
  labelId: string;
  icon: LucideIcon;
}

/** PRD FR-5.3, in this order. پارٹی کھاتے sits above باقی مال (PRD FR-12.43). */
export const BIKRI_MENU: readonly ModuleEntry[] = [
  { id: 'bikri-book', labelId: 'layout-module-bikri-book', icon: NotebookPen },
  { id: 'receipt', labelId: 'layout-module-receipt', icon: ArrowDownToLine },
  { id: 'payment', labelId: 'layout-module-payment', icon: ArrowUpFromLine },
  { id: 'advance', labelId: 'layout-module-advance', icon: Coins },
  { id: 'today', labelId: 'layout-module-today', icon: History },
  { id: 'party-ledgers', labelId: 'layout-module-party-ledgers', icon: BookUser },
  { id: 'stock', labelId: 'layout-module-stock', icon: Package },
];

/** The side menu of the روکڑ area (PRD 5, Architecture 5): the cash book and the entry screens. */
export type RokarMenuId = 'cashbook' | 'expense' | 'lent' | 'khata' | 'general' | 'voucher';

/**
 * PRD 5 (the روکڑ branch), in this order. پھل (fruit taken) comes next to خرچہ with plan 17d, and
 * is one more entry here and one more route. جنرل is the fifth screen (plan 17).
 */
export const ROKAR_MENU: readonly { id: RokarMenuId; labelId: string; icon: LucideIcon }[] = [
  { id: 'cashbook', labelId: 'layout-rokar-cashbook', icon: Wallet },
  { id: 'expense', labelId: 'layout-rokar-expense', icon: Receipt },
  { id: 'lent', labelId: 'layout-rokar-lent', icon: Handshake },
  { id: 'khata', labelId: 'layout-rokar-khata', icon: NotebookTabs },
  { id: 'general', labelId: 'layout-rokar-general', icon: BookText },
  { id: 'voucher', labelId: 'layout-rokar-voucher', icon: FileSignature },
];

interface SideMenuProps<Id extends string> {
  entries: readonly { id: Id; labelId: string; icon: LucideIcon }[];
  /** The Fluent message ID of the menu's name, read by screen readers. */
  labelId: string;
  current: Id;
  onSelect: (id: Id) => void;
}

function SideMenu<Id extends string>({ entries, labelId, current, onSelect }: SideMenuProps<Id>) {
  const { l10n } = useLocalization();
  const region = useFocusRegion<HTMLElement>('menu');
  const onKeyDown = useRovingKeys({ isMenu: true });
  return (
    <nav
      ref={region}
      aria-label={l10n.getString(labelId)}
      onKeyDown={onKeyDown}
      className="flex w-module-menu shrink-0 flex-col gap-1 border-e border-border bg-surface-sunken px-3 py-4"
    >
      {entries.map(({ id, labelId: entryLabelId, icon: Icon }) => {
        const isCurrent = id === current;
        return (
          <button
            key={id}
            type="button"
            data-nav-item
            aria-current={isCurrent ? 'page' : undefined}
            onClick={() => {
              onSelect(id);
            }}
            className={`flex min-h-control items-center gap-3 rounded-2 border px-3 py-1 text-start transition-calm ${isCurrent ? 'border-border bg-surface text-primary' : 'border-transparent text-ink hover:bg-bg'}`}
          >
            <Icon
              aria-hidden="true"
              className={`size-icon-md shrink-0 ${isCurrent ? 'text-primary' : 'text-ink-muted'}`}
            />
            <span className="type-body min-w-0">{l10n.getString(entryLabelId)}</span>
          </button>
        );
      })}
    </nav>
  );
}

interface ModuleMenuProps {
  current: ModuleMenuId;
  onSelect: (id: ModuleMenuId) => void;
}

export function ModuleMenu({ current, onSelect }: ModuleMenuProps) {
  return (
    <SideMenu
      entries={BIKRI_MENU}
      labelId="layout-module-label"
      current={current}
      onSelect={onSelect}
    />
  );
}

interface RokarMenuProps {
  current: RokarMenuId;
  onSelect: (id: RokarMenuId) => void;
}

export function RokarMenu({ current, onSelect }: RokarMenuProps) {
  return (
    <SideMenu
      entries={ROKAR_MENU}
      labelId="layout-rokar-label"
      current={current}
      onSelect={onSelect}
    />
  );
}
