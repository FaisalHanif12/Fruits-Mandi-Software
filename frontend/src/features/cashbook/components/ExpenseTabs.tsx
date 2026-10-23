// The tab strip of the خرچہ area (PRD 16.7). With one tab there is nothing to choose, so nothing is
// drawn; the پھل tab of plan 17d makes it appear (see expense-tabs.ts).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useRouterState } from '@tanstack/react-router';
import { EXPENSE_TABS } from '../expense-tabs';

export function ExpenseTabs() {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  if (EXPENSE_TABS.length < 2) return null;
  return (
    <nav
      aria-label={l10n.getString('cashbook-tabs-label')}
      className="flex flex-wrap gap-2 border-b border-border"
    >
      {EXPENSE_TABS.map((tab) => (
        <button
          key={tab.id}
          type="button"
          aria-current={pathname === tab.path ? 'page' : undefined}
          onClick={() => {
            void navigate({ to: tab.path });
          }}
          className={`type-control h-control rounded-t-2 px-4 transition-calm ${pathname === tab.path ? 'bg-primary-soft text-primary' : 'text-ink-muted'}`}
        >
          <Localized id={tab.labelId} />
        </button>
      ))}
    </nav>
  );
}
