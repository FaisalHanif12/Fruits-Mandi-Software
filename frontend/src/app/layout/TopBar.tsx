// The top bar on every screen (PRD FR-5.6, design system 6): the search box with its Ctrl+K chip,
// the attention bell, the language switch and the user. The search box and the bell are
// placeholders for now: they say "coming soon" (D2 and D5 build them). Ctrl+K focuses the box.

import { useLocalization } from '@fluent/react';
import { Bell, Search } from 'lucide-react';
import { useRef } from 'react';
import { SHORTCUTS, useShortcut } from '../../shared/keyboard';
import { KeyChip } from '../../shared/ui';
import { LanguageSwitch } from './LanguageSwitch';

interface TopBarProps {
  /** The signed-in user's name, shown as typed (a name is not translated, PRD FR-7.25). */
  userName: string;
  onUserMenu?: () => void;
}

export function TopBar({ userName, onUserMenu }: TopBarProps) {
  const { l10n } = useLocalization();
  const searchRef = useRef<HTMLInputElement>(null);
  useShortcut(SHORTCUTS.globalSearch, () => {
    searchRef.current?.focus();
  });
  const comingSoon = l10n.getString('layout-coming-soon');
  const initial = Array.from(userName)[0] ?? '';
  return (
    <header className="flex h-topbar shrink-0 items-center gap-4 border-b border-border bg-surface px-5">
      <label
        title={comingSoon}
        className="flex h-control-compact max-w-xl flex-1 items-center gap-2 rounded-2 border border-border-control bg-bg px-3"
      >
        <Search aria-hidden="true" className="size-icon-sm shrink-0 text-ink-muted" />
        <input
          ref={searchRef}
          type="search"
          aria-label={l10n.getString('layout-search-label')}
          placeholder={l10n.getString('layout-search-placeholder')}
          autoComplete="off"
          spellCheck={false}
          className="type-field min-w-0 flex-1 bg-transparent text-ink placeholder:text-ink-placeholder"
        />
        <KeyChip shortcut={SHORTCUTS.globalSearch} tone="on-light" />
      </label>
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
      <button
        type="button"
        aria-label={l10n.getString('layout-user-menu')}
        title={userName}
        onClick={onUserMenu}
        className="flex h-control-compact items-center gap-2 rounded-2 border border-border bg-surface ps-2 pe-3 text-ink"
      >
        <span className="type-body flex size-5 items-center justify-center rounded-full bg-primary leading-chip text-on-primary">
          {initial}
        </span>
        <span className="type-body">{userName}</span>
      </button>
    </header>
  );
}
