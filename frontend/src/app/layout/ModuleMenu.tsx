// The side menus of the بکری area (PRD FR-5.3) and of the روکڑ area (PRD 5, plan 17), one look
// (design system 6): 200 px, sunken ground, the current item in primary. Next to the main menu, so the arrow that points into the screen
// leads to the screen, and the other arrow leads back to the menu (PRD FR-7.17).

import { useLocalization } from '@fluent/react';
import { useFocusRegion, useRovingKeys } from '../../shared/keyboard';

export type ModuleMenuId =
  'bikri-book' | 'receipt' | 'payment' | 'advance' | 'pukhta' | 'today' | 'stock';

interface ModuleEntry {
  id: ModuleMenuId;
  labelId: string;
}

/** PRD FR-5.3, in this order. */
export const BIKRI_MENU: readonly ModuleEntry[] = [
  { id: 'bikri-book', labelId: 'layout-module-bikri-book' },
  { id: 'receipt', labelId: 'layout-module-receipt' },
  { id: 'payment', labelId: 'layout-module-payment' },
  { id: 'advance', labelId: 'layout-module-advance' },
  { id: 'pukhta', labelId: 'layout-module-pukhta' },
  { id: 'today', labelId: 'layout-module-today' },
  { id: 'stock', labelId: 'layout-module-stock' },
];

/** The side menu of the روکڑ area (PRD 5, Architecture 5): the cash book and the entry screens. */
export type RokarMenuId = 'cashbook' | 'expense' | 'lent' | 'khata' | 'general' | 'voucher';

/**
 * PRD 5 (the روکڑ branch), in this order. پھل (fruit taken) comes next to خرچہ with plan 17d, and
 * is one more entry here and one more route. جنرل is the fifth screen (plan 17).
 */
export const ROKAR_MENU: readonly { id: RokarMenuId; labelId: string }[] = [
  { id: 'cashbook', labelId: 'layout-rokar-cashbook' },
  { id: 'expense', labelId: 'layout-rokar-expense' },
  { id: 'lent', labelId: 'layout-rokar-lent' },
  { id: 'khata', labelId: 'layout-rokar-khata' },
  { id: 'general', labelId: 'layout-rokar-general' },
  { id: 'voucher', labelId: 'layout-rokar-voucher' },
];

interface SideMenuProps<Id extends string> {
  entries: readonly { id: Id; labelId: string }[];
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
      {entries.map(({ id, labelId: entryLabelId }) => (
        <button
          key={id}
          type="button"
          data-nav-item
          aria-current={id === current ? 'page' : undefined}
          onClick={() => {
            onSelect(id);
          }}
          className={`flex h-control items-center rounded-2 px-3 text-start transition-calm ${id === current ? 'bg-primary text-on-primary' : 'text-ink'}`}
        >
          <span className="type-body">{l10n.getString(entryLabelId)}</span>
        </button>
      ))}
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
