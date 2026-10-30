// The رپورٹ area (route /reports; PRD FR-5.2, 19.1, 19.2): the catalogue of reports grouped as the
// PRD lists them. A search box finds a report by its name, its line or its number (R20), and a row
// of buttons jumps to a group (plan 38). ↑ ↓ move between the reports and Enter opens one. Four
// states: loading, an error with a retry, nothing built yet (or nothing found), and the groups.

import { Localized, useLocalization } from '@fluent/react';
import { Link } from '@tanstack/react-router';
import { FileText, FileX, SearchX } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import { uiErrorOf } from '../../../shared/api';
import { formNavigationProps, useRovingKeys } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  ErrorState,
  Field,
  FilterBar,
  ListPage,
  Select,
  LoadingState,
} from '../../../shared/ui';
import { groupCatalog, helpIdOf } from '../catalog';
import { useReportCatalog } from '../hooks';

const ISOLATION_MARKS = /[\u2066-\u2069]/g;

export function ReportsHomePage() {
  const { l10n } = useLocalization();
  const catalog = useReportCatalog();
  const onKeyDown = useRovingKeys();
  const ref = useRef<HTMLElement>(null);
  // The group chosen in "Go to"; the focus moves to it once the list has closed.
  const jumpGroup = useRef<string | null>(null);
  const [text, setText] = useState('');
  const ready = catalog.data !== undefined;
  // The first report has focus when the area opens, so ↓ and Enter work at once.
  useEffect(() => {
    if (ready) ref.current?.querySelector<HTMLElement>('[data-nav-item]')?.focus();
  }, [ready]);

  const needle = text.trim().toLowerCase();
  // A report is found by its name, its line of help or its number, in the language of the screen.
  const matches = (entry: ReportCatalogEntry): boolean => {
    if (needle === '') return true;
    const haystack = [
      entry.id,
      l10n.getString(entry.title_id),
      l10n.getBundle(helpIdOf(entry)) === null ? '' : l10n.getString(helpIdOf(entry)),
    ];
    // Fluent puts invisible isolation marks round a term (like Buyer); the user does not type them.
    return haystack.some((part) =>
      part.replace(ISOLATION_MARKS, '').toLowerCase().includes(needle),
    );
  };
  const groups = useMemo(() => groupCatalog(catalog.data ?? []), [catalog.data]);
  const shown = groups
    .map((group) => ({ ...group, entries: group.entries.filter(matches) }))
    .filter((group) => group.entries.length > 0);

  /** Takes the user to the first report of a group: it scrolls into view and has focus. */
  const jumpTo = (group: string) => {
    const first = ref.current?.querySelector<HTMLElement>(
      `[data-group="${group}"] [data-nav-item]`,
    );
    first?.scrollIntoView({ block: 'start' });
    first?.focus();
  };

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
  } else if (shown.length === 0) {
    body = (
      <EmptyState
        icon={SearchX}
        title={<Localized id="reports-search-none" />}
        help={<Localized id="reports-search-none-help" />}
        action={
          <Button
            onClick={() => {
              setText('');
            }}
          >
            <Localized id="reports-search-clear" />
          </Button>
        }
      />
    );
  } else {
    body = (
      <nav
        ref={ref}
        onKeyDown={onKeyDown}
        className="flex min-h-0 flex-1 flex-col gap-5 overflow-auto"
      >
        {shown.map((group) => (
          <section
            key={group.group}
            data-group={group.group}
            aria-label={l10n.getString('reports-group-label', {
              group: l10n.getString(group.titleId),
            })}
            className="flex flex-col gap-3"
          >
            <h2 className="type-section text-ink">{l10n.getString(group.titleId)}</h2>
            <div className="grid grid-cols-2 gap-3 normal:grid-cols-3">
              {group.entries.map((entry) => (
                <Link
                  key={entry.id}
                  to="/reports/$id"
                  params={{ id: entry.id }}
                  data-nav-item
                  className="flex min-h-control items-center gap-3 rounded-3 border border-border bg-surface p-3"
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
    <ListPage
      titleId="reports-title"
      help={
        <Localized id="reports-help">
          <p />
        </Localized>
      }
      filters={
        ready && catalog.data.length > 0 ? (
          <div {...formNavigationProps}>
            <FilterBar
              screen="reports.home"
              search={{
                value: text,
                onChange: setText,
                labelId: 'reports-search-label',
                placeholderId: 'reports-search-placeholder',
              }}
            >
              {/* A jump to a group: a choice that takes the user to its first report (plan 38). */}
              <Field label={<Localized id="reports-jump-label" />}>
                {(control) => (
                  <Select<string>
                    {...control}
                    value={undefined}
                    placeholder={<Localized id="reports-jump-placeholder" />}
                    onChange={(group) => {
                      jumpGroup.current = group;
                    }}
                    afterChoose={() => {
                      if (jumpGroup.current !== null) jumpTo(jumpGroup.current);
                    }}
                    options={groups.map((group) => ({
                      value: group.group,
                      label: l10n.getString(group.titleId),
                    }))}
                  />
                )}
              </Field>
            </FilterBar>
          </div>
        ) : undefined
      }
    >
      {body}
    </ListPage>
  );
}
