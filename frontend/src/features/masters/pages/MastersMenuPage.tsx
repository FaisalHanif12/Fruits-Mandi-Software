// The انڈکس menu (PRD FR-5.2, 10.3): the nine master lists, one big card each. ↑ ↓ move between
// them, Enter opens one. The accounts and the commission rules join it in plans 07 and 09.

import { Localized } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import { Landmark, ListTree, Scale, type LucideIcon } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { useRovingKeys } from '../../../shared/keyboard';
import { MASTER_CONFIGS } from '../configs';

/** The lists that are not one of the nine masters: the accounts, the chart and the rules (plans 07, 09). */
const STATIC_ENTRIES = [
  {
    to: '/index/accounts',
    icon: Landmark,
    titleId: 'masters-accounts-title',
    helpId: 'masters-accounts-help',
  },
  {
    to: '/index/chart',
    icon: ListTree,
    titleId: 'masters-chart-title',
    helpId: 'masters-chart-help',
  },
  {
    to: '/index/rules',
    icon: Scale,
    titleId: 'masters-rules-title',
    helpId: 'masters-rules-help',
  },
] as const;

type MenuCardProps = {
  icon: LucideIcon;
  titleId: string;
  helpId: string;
} & (
  | { to: '/index/accounts' | '/index/chart' | '/index/rules' }
  | { to: '/index/$kind'; params: { kind: string } }
);

function MenuCard(props: MenuCardProps) {
  const { icon: Icon, titleId, helpId } = props;
  const className =
    'flex min-h-control items-center gap-4 rounded-3 border border-border bg-surface p-4';
  const body = (
    <>
      <span className="flex size-badge shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary">
        <Icon aria-hidden="true" className="size-icon-lg" />
      </span>
      <span className="flex min-w-0 flex-col">
        <Localized id={titleId}>
          <span className="type-section text-ink" />
        </Localized>
        <Localized id={helpId}>
          <span className="type-body text-ink-muted" />
        </Localized>
      </span>
    </>
  );
  return props.to === '/index/$kind' ? (
    <Link to={props.to} params={props.params} data-nav-item className={className}>
      {body}
    </Link>
  ) : (
    <Link to={props.to} data-nav-item className={className}>
      {body}
    </Link>
  );
}

export function MastersMenuPage() {
  const onKeyDown = useRovingKeys();
  const ref = useRef<HTMLElement>(null);
  // The first list has focus when the menu opens, so ↓ and Enter work at once.
  useEffect(() => {
    ref.current?.querySelector<HTMLElement>('[data-nav-item]')?.focus();
  }, []);
  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-1">
        <Localized id="masters-menu-title">
          <h1 className="type-title" />
        </Localized>
        <Localized id="masters-menu-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <nav ref={ref} onKeyDown={onKeyDown} className="grid grid-cols-1 gap-3 normal:grid-cols-2">
        {STATIC_ENTRIES.map(({ to, icon: Icon, titleId, helpId }) => (
          <MenuCard key={to} to={to} icon={Icon} titleId={titleId} helpId={helpId} />
        ))}
        {MASTER_CONFIGS.map(({ kind, path, icon: Icon, titleId, helpId }) => (
          <MenuCard
            key={kind}
            to="/index/$kind"
            params={{ kind: path }}
            icon={Icon}
            titleId={titleId}
            helpId={helpId}
          />
        ))}
      </nav>
    </div>
  );
}
