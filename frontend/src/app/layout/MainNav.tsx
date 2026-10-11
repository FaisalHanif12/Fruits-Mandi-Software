// The main menu on the start side (PRD FR-5.2, design system 6): 224 px with labels from 1366 px
// wide, a 64 px icon rail below that, where each label is the item's name for screen readers and
// a tooltip. ↑ ↓ move between items, Enter opens, and the arrow that points into the screen moves
// focus to the screen (PRD FR-7.17).

import { useLocalization } from '@fluent/react';
import {
  BarChart3,
  BookOpen,
  CloudUpload,
  FileText,
  LayoutDashboard,
  Settings,
  Truck,
  Users,
  Wallet,
  type LucideIcon,
} from 'lucide-react';
import { useFocusRegion, useRovingKeys } from '../../shared/keyboard';

export type MainNavId =
  | 'dashboard'
  | 'index'
  | 'bikri'
  | 'rokar'
  | 'partners'
  | 'analytics'
  | 'reports'
  | 'backup'
  | 'settings';

interface NavEntry {
  id: MainNavId;
  labelId: string;
  icon: LucideIcon;
}

/** PRD FR-5.2, in this order. */
export const MAIN_NAV: readonly NavEntry[] = [
  { id: 'dashboard', labelId: 'layout-nav-dashboard', icon: LayoutDashboard },
  { id: 'index', labelId: 'layout-nav-index', icon: BookOpen },
  { id: 'bikri', labelId: 'layout-nav-bikri', icon: Truck },
  { id: 'rokar', labelId: 'layout-nav-rokar', icon: Wallet },
  { id: 'partners', labelId: 'layout-nav-partners', icon: Users },
  { id: 'analytics', labelId: 'layout-nav-analytics', icon: BarChart3 },
  { id: 'reports', labelId: 'layout-nav-reports', icon: FileText },
  { id: 'backup', labelId: 'layout-nav-backup', icon: CloudUpload },
  { id: 'settings', labelId: 'layout-nav-settings', icon: Settings },
];

interface MainNavProps {
  current: MainNavId;
  onSelect: (id: MainNavId) => void;
  /** Items the user's role cannot open, for example settings for a Munshi (PRD FR-2.4). */
  hidden?: readonly MainNavId[];
}

export function MainNav({ current, onSelect, hidden = [] }: MainNavProps) {
  const { l10n } = useLocalization();
  const region = useFocusRegion<HTMLElement>('menu');
  const onKeyDown = useRovingKeys({ isMenu: true });
  return (
    <nav
      ref={region}
      aria-label={l10n.getString('layout-nav-label')}
      onKeyDown={onKeyDown}
      className="flex w-rail shrink-0 flex-col gap-1 bg-nav px-3 py-4 normal:w-nav"
    >
      <p className="type-section hidden px-3 pb-3 text-on-primary normal:block">
        {l10n.getString('common-app-name')}
      </p>
      {MAIN_NAV.filter((entry) => !hidden.includes(entry.id)).map(({ id, labelId, icon: Icon }) => {
        const label = l10n.getString(labelId);
        const isCurrent = id === current;
        return (
          <button
            key={id}
            type="button"
            data-nav-item
            aria-current={isCurrent ? 'page' : undefined}
            aria-label={label}
            title={label}
            onClick={() => {
              onSelect(id);
            }}
            className={`flex h-control items-center justify-center gap-3 rounded-2 transition-calm normal:justify-start normal:px-3 ${isCurrent ? 'bg-nav-active text-on-primary' : 'text-nav-ink'}`}
          >
            <Icon aria-hidden="true" className="size-icon-md shrink-0" />
            <span className="type-body hidden normal:inline">{label}</span>
          </button>
        );
      })}
    </nav>
  );
}
