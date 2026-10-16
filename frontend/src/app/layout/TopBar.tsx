// The top bar on every screen (PRD FR-5.6, design system 6): the search box with its Ctrl+K chip
// (it finds accounts and بکری, GlobalSearch), the attention bell, the language switch and the user. The
// bell is a placeholder for now: it says "coming soon" (D5 builds it).

import { useLocalization } from '@fluent/react';
import { Bell } from 'lucide-react';
import { useRef, useState } from 'react';
import { ChangePasswordDialog } from '../../features/auth/components/ChangePasswordDialog';
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
  /** A search result was chosen: open that بکری. */
  onOpenBikri: (id: number) => void;
}

export function TopBar({ userName, onLock, onSignOut, onOpenAccount, onOpenBikri }: TopBarProps) {
  const { l10n } = useLocalization();
  const comingSoon = l10n.getString('layout-coming-soon');
  const [changingPassword, setChangingPassword] = useState(false);
  const userButton = useRef<HTMLButtonElement>(null);
  return (
    <header className="flex h-topbar shrink-0 items-center gap-4 border-b border-border bg-surface px-5">
      <GlobalSearch onOpenAccount={onOpenAccount} onOpenBikri={onOpenBikri} />
      {/* The search box takes the room it can, up to its widest; the rest of the bar goes to the end
          with the bell. A spacer that shared the room cut the placeholder short (finding 35). */}
      <button
        type="button"
        aria-label={l10n.getString('layout-attention-label')}
        title={comingSoon}
        className="ms-auto flex size-control-compact items-center justify-center rounded-2 border border-border bg-surface text-ink"
      >
        <Bell aria-hidden="true" className="size-icon-md" />
      </button>
      <LanguageSwitch />
      <UserMenu
        userName={userName}
        buttonRef={userButton}
        onChangePassword={() => {
          setChangingPassword(true);
        }}
        onLock={onLock}
        onSignOut={onSignOut}
      />
      <ChangePasswordDialog
        open={changingPassword}
        onOpenChange={setChangingPassword}
        returnFocusTo={userButton}
      />
    </header>
  );
}
