// The top bar on every screen (PRD FR-5.6, design system 6): the search box with its Ctrl+K chip
// (it finds accounts, GlobalSearch), the attention bell, the language switch and the user. The
// bell is a placeholder for now: it says "coming soon" (D5 builds it).

import { useLocalization } from '@fluent/react';
import { Bell } from 'lucide-react';
import { GlobalSearch } from './GlobalSearch';
import { LanguageSwitch } from './LanguageSwitch';
import { UserMenu } from './UserMenu';

interface TopBarProps {
  /** The signed-in user's name, shown as typed (a name is not translated, PRD FR-7.25). */
  userName: string;
  onLock: () => void;
  onSignOut: () => void;
  /** A search result was chosen: open that account. */
  onOpenAccount: (id: number) => void;
}

export function TopBar({ userName, onLock, onSignOut, onOpenAccount }: TopBarProps) {
  const { l10n } = useLocalization();
  const comingSoon = l10n.getString('layout-coming-soon');
  return (
    <header className="flex h-topbar shrink-0 items-center gap-4 border-b border-border bg-surface px-5">
      <GlobalSearch onOpenAccount={onOpenAccount} />
      <div className="flex-1" />
      <button
        type="button"
        aria-label={l10n.getString('layout-attention-label')}
        title={comingSoon}
        className="flex size-control-compact items-center justify-center rounded-2 border border-border bg-surface text-ink"
      >
        <Bell aria-hidden="true" className="size-icon-md" />
      </button>
      <LanguageSwitch />
      <UserMenu userName={userName} onLock={onLock} onSignOut={onSignOut} />
    </header>
  );
}
