// The top bar on every screen (PRD FR-5.6, design system 6): the search box with its Ctrl+K chip
// (it finds accounts, بکری and vouchers, GlobalSearch), the attention bell (AttentionBell), the
// language switch and the user.

import { useRef, useState } from 'react';
import { ChangePasswordDialog } from '../../features/auth/components/ChangePasswordDialog';
import type { VoucherSearchHit } from '../../shared/accounts';
import { AttentionBell } from './AttentionBell';
import { GlobalSearch } from './GlobalSearch';
import { LanguageSwitch } from './LanguageSwitch';
import { UserMenu } from './UserMenu';

interface TopBarProps {
  /** The signed-in user's name, shown as typed (a name is not translated, PRD FR-7.25). */
  userName: string;
  onLock: () => void;
  onSignOut: () => void;
  /** The user menu's "show the guide": opens the first-run guide again (PRD FR-7.12). */
  onGuide: () => void;
  /** A search result was chosen: open that account. */
  onOpenAccount: (id: number) => void;
  /** A search result was chosen: open that بکری. */
  onOpenBikri: (id: number) => void;
  /** A search result was chosen: open that voucher. */
  onOpenVoucher: (hit: VoucherSearchHit) => void;
}

export function TopBar({
  userName,
  onLock,
  onSignOut,
  onGuide,
  onOpenAccount,
  onOpenBikri,
  onOpenVoucher,
}: TopBarProps) {
  const [changingPassword, setChangingPassword] = useState(false);
  const userButton = useRef<HTMLButtonElement>(null);
  return (
    <header className="flex h-topbar shrink-0 items-center gap-4 border-b border-border bg-surface px-5">
      <GlobalSearch
        onOpenAccount={onOpenAccount}
        onOpenBikri={onOpenBikri}
        onOpenVoucher={onOpenVoucher}
      />
      {/* The search box takes the room it can, up to its widest; the rest of the bar goes to the end
          with the bell. A spacer that shared the room cut the placeholder short (finding 35). */}
      <AttentionBell />
      <LanguageSwitch />
      <UserMenu
        userName={userName}
        buttonRef={userButton}
        onChangePassword={() => {
          setChangingPassword(true);
        }}
        onGuide={onGuide}
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
