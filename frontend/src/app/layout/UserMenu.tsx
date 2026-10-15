// The user in the top bar (PRD FR-5.6, design system 6): his name opens a small menu with "change
// password", "lock the app" and "sign out". Built on Radix DropdownMenu, so arrows, Enter and Esc work, and focus goes
// back to the button when it closes.

import { Localized, useLocalization } from '@fluent/react';
import { KeyRound, Lock, LogOut } from 'lucide-react';
import { DropdownMenu } from 'radix-ui';
import type { Ref } from 'react';
import { useUiLanguage } from '../../shared/i18n';
import { scriptOf } from '../../shared/text/scriptHint';

interface UserMenuProps {
  /** The signed-in user's name, shown as typed (a name is not translated, PRD FR-7.25). */
  userName: string;
  /** The button, so a dialog opened from the menu can give focus back to it. */
  buttonRef?: Ref<HTMLButtonElement> | undefined;
  onChangePassword: () => void;
  onLock: () => void;
  onSignOut: () => void;
}

const ITEM_CLASSES =
  'type-body flex h-control cursor-pointer items-center gap-3 rounded-1 px-3 text-ink data-[highlighted]:bg-primary-soft';

export function UserMenu({
  userName,
  buttonRef,
  onChangePassword,
  onLock,
  onSignOut,
}: UserMenuProps) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const initial = Array.from(userName)[0] ?? '';
  return (
    <DropdownMenu.Root dir={direction}>
      <DropdownMenu.Trigger
        ref={buttonRef}
        aria-label={l10n.getString('layout-user-menu')}
        title={userName}
        className="flex h-control-compact items-center gap-2 rounded-2 border border-border bg-surface ps-2 pe-3 text-ink"
      >
        {/* The name and its first letter carry the script they are written in, so an Urdu name
            in the English app is drawn at the Urdu size and line height (NFR-USE-20, NFR-URD-11). The
            circle is 26 px, so a Nastaliq letter at that size fits in it. */}
        <span
          lang={scriptOf(initial)}
          className="type-body flex size-icon-lg items-center justify-center rounded-full bg-primary text-on-primary"
        >
          {initial}
        </span>
        <span lang={scriptOf(userName)} className="type-body">
          {userName}
        </span>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content
          lang={language}
          align="end"
          sideOffset={4}
          className="z-50 min-w-nav rounded-2 border border-border bg-surface p-1 shadow-overlay"
        >
          <DropdownMenu.Item className={ITEM_CLASSES} onSelect={onChangePassword}>
            <KeyRound aria-hidden="true" className="size-icon-md" />
            <Localized id="layout-user-change-password" />
          </DropdownMenu.Item>
          <DropdownMenu.Item className={ITEM_CLASSES} onSelect={onLock}>
            <Lock aria-hidden="true" className="size-icon-md" />
            <Localized id="layout-user-lock" />
          </DropdownMenu.Item>
          <DropdownMenu.Item className={ITEM_CLASSES} onSelect={onSignOut}>
            <LogOut aria-hidden="true" className="size-icon-md" />
            <Localized id="layout-user-signout" />
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}
