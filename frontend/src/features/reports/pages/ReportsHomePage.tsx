// The رپورٹ area (route /reports; PRD FR-5.2, 19.1, 19.2): the catalogue of reports grouped as the
// PRD lists them. ↑ ↓ move between the reports and Enter opens one. Four states: loading, an error
// with a retry, nothing built yet, and the groups.

import { Localized, useLocalization } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import { FileText, FileX } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useRovingKeys } from '../../../shared/keyboard';
import { EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { groupCatalog, helpIdOf } from '../catalog';
import { useReportCatalog } from '../hooks';

export function ReportsHomePage() {
  const { l10n } = useLocalization();
  const catalog = useReportCatalog();
  const onKeyDown = useRovingKeys();
  const ref = useRef<HTMLElement>(null);
  const ready = catalog.data !== undefined;
  // The first report has focus when the area opens, so ↓ and Enter work at once.
  useEffect(() => {
    if (ready) ref.current?.querySelector<HTMLElement>('[data-nav-item]')?.focus();
  }, [ready]);

  let body;
  if (catalog.isPending) {
    body = <LoadingState />;
  } else if (catalog.isError) {
    body = (
      <ErrorState
        title={<Localized id="reports-catalog-failed" />}
        error={uiErrorOf(catalog.error)}
        onRetry={() => {
          void catalog.refetch();
        }}
      />
    );
  } else if (catalog.data.length === 0) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="reports-catalog-empty" />}
        help={<Localized id="reports-catalog-empty-help" />}
      />
    );
  } else {
    body = (
      <nav ref={ref} onKeyDown={onKeyDown} className="flex flex-col gap-5">
        {groupCatalog(catalog.data).map((group) => (
          <section
            key={group.group}
            aria-label={l10n.getString('reports-group-label', {
              group: l10n.getString(group.titleId),
            })}
            className="flex flex-col gap-3"
          >
            <h2 className="type-section text-ink">{l10n.getString(group.titleId)}</h2>
            <div className="grid grid-cols-1 gap-3 normal:grid-cols-2">
              {group.entries.map((entry) => (
                <Link
                  key={entry.id}
                  to="/reports/$id"
                  params={{ id: entry.id }}
                  data-nav-item
                  className="flex min-h-control items-center gap-4 rounded-3 border border-border bg-surface p-4"
                >
                  <span className="flex size-badge shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary">
                    <FileText aria-hidden="true" className="size-icon-lg" />
                  </span>
                  <span className="flex min-w-0 flex-col">
                    <span className="type-section text-ink">{l10n.getString(entry.title_id)}</span>
                    {l10n.getBundle(helpIdOf(entry)) === null ? null : (
                      <span className="type-body text-ink-muted">
                        {l10n.getString(helpIdOf(entry))}
                      </span>
                    )}
                  </span>
                </Link>
              ))}
            </div>
          </section>
        ))}
      </nav>
    );
  }

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-1">
        <Localized id="reports-title">
          <h1 className="type-title" />
        </Localized>
        <Localized id="reports-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {body}
    </div>
  );
}
