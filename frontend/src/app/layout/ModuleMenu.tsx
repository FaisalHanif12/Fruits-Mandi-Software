// The side menu of the بکری area (PRD FR-5.3, design system 6): 200 px, sunken ground, the
// current item in primary. Next to the main menu, so the arrow that points into the screen
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

interface ModuleMenuProps {
  current: ModuleMenuId;
  onSelect: (id: ModuleMenuId) => void;
}

export function ModuleMenu({ current, onSelect }: ModuleMenuProps) {
  const { l10n } = useLocalization();
  const region = useFocusRegion<HTMLElement>('menu');
  const onKeyDown = useRovingKeys({ isMenu: true });
  return (
    <nav
      ref={region}
      aria-label={l10n.getString('layout-module-label')}
      onKeyDown={onKeyDown}
      className="flex w-module-menu shrink-0 flex-col gap-1 border-e border-border bg-surface-sunken px-3 py-4"
    >
      {BIKRI_MENU.map(({ id, labelId }) => (
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
          <span className="type-body">{l10n.getString(labelId)}</span>
        </button>
      ))}
    </nav>
  );
}
