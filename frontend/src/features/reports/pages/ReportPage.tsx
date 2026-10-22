// One report (route /reports/$id; PRD 19.1 to 19.5, 19.10, 19.11): the ONE screen every report
// uses. The catalogue says which filter boxes a report takes; the backend sends its columns, its
// rows page by page and its totals over the whole filter. The screen lays them out and opens the
// record behind a row (FR-19.4). It works out nothing (CLAUDE.md rule 1).
//
// - The filters are kept here, above the table, so a language switch (which builds the table
//   again) loses nothing, and the focused row is focused again (Code Standards F39).
// - A filter change asks the backend once, 250 ms after the last key (Code Standards F35). A date
//   that is half typed is refused under its own box and the rows already on screen stay.
// - F9 or the button shows the print preview; Esc leaves it, then leaves the report.
//
// Four states: loading, an error with a retry (also for a report that does not exist), nothing
// found (with a way out), and the rows.

import { Localized } from '@fluent/react';
import type { InfiniteData } from '@tanstack/react-query';
import { useNavigate, useParams, useSearch } from '@tanstack/react-router';
import { ArrowLeft, FileX, Printer } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import type { ReportPage as ReportPageData } from '../../../bindings/ReportPage';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useDebouncedValue, useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { ReportFilterBar } from '../components/ReportFilterBar';
import { ReportPreview } from '../components/ReportPreview';
import { ReportResults } from '../components/ReportResults';
import type { ReportRowView } from '../columns';
import {
  EMPTY_FILTERS,
  canRun,
  filtersToWire,
  isUnfiltered,
  type ReportFilterState,
} from '../filters';
import { useReportCatalog, useReportPages } from '../hooks';
import { parseReportSearch, stateFromSearch } from '../search';

/** How long the filters rest before the backend is asked (Code Standards F35). */
const FILTER_SETTLE_MS = 250;

/** True when the refusal points at a filter box: the box shows it and the rows stay. */
function isFilterRefusal(error: UiError): boolean {
  return (
    error.errors.length > 0 &&
    error.errors.every(
      (item) => typeof item.field === 'string' && item.field.startsWith('filters.'),
    )
  );
}

/** Opens the record behind a row; a row that opens nothing (no screen for it yet) does nothing. */
function useOpenSource(catalog: readonly ReportCatalogEntry[]) {
  const navigate = useNavigate();
  return (view: ReportRowView) => {
    const source = view.row.source;
    if (source === null) return;
    switch (source.kind) {
      case 'bikri':
        void navigate({ to: '/bikri/book', search: { id: source.id } });
        return;
      case 'voucher':
        void navigate({ to: '/bikri/voucher', search: { id: source.id } });
        return;
      case 'account':
        // The ledger of the account is the most useful record behind it (R13), when it is built.
        if (catalog.some((entry) => entry.id === 'R13')) {
          void navigate({
            to: '/reports/$id',
            params: { id: 'R13' },
            search: { party: source.id },
          });
        }
        return;
      default:
        return;
    }
  };
}

function ReportBody({
  entry,
  catalog,
}: {
  entry: ReportCatalogEntry;
  catalog: readonly ReportCatalogEntry[];
}) {
  const navigate = useNavigate();
  const search = parseReportSearch(useSearch({ strict: false }));
  const [state, setState] = useState<ReportFilterState>(() => stateFromSearch(search));
  const [view, setView] = useState<'table' | 'preview'>('table');
  const [focusRowId, setFocusRowId] = useState<string | null>(null);
  const settled = useDebouncedValue(state, FILTER_SETTLE_MS);
  const wire = useMemo(
    () => (canRun(settled, entry) ? filtersToWire(settled, entry.filters) : null),
    [settled, entry],
  );
  const query = useReportPages(entry.id, wire);
  const open = useOpenSource(catalog);

  // The pages last shown, kept while a half-typed filter is refused (they stay under the box). They
  // are state set while rendering, which React allows when the value really changed.
  const [kept, setKept] = useState<InfiniteData<ReportPageData> | null>(null);
  if (query.data !== undefined && !query.isPlaceholderData && query.data !== kept) {
    setKept(query.data);
  }
  const refusal = query.isError ? uiErrorOf(query.error) : null;
  const filterRefusal = refusal !== null && isFilterRefusal(refusal) ? refusal : null;

  // A party that came with the address has no names yet; the backend's answer carries them.
  const echoed = (query.data ?? kept)?.pages[0]?.filters.party ?? null;
  const party =
    state.party !== null &&
    state.party.name_ur === '' &&
    echoed !== null &&
    echoed.account_id === state.party.id
      ? {
          id: echoed.account_id,
          name_ur: echoed.name_ur,
          name_en: echoed.name_en,
          code: echoed.code,
        }
      : state.party;

  const containerRef = useFocusFirstField<HTMLDivElement>(view);
  const backToList = () => {
    void navigate({ to: '/reports' });
  };
  useEscape('leave', () => {
    if (view === 'preview') setView('table');
    else backToList();
  });
  useShortcut(SHORTCUTS.print, () => {
    if (wire !== null) setView('preview');
  });

  const change = (part: Partial<ReportFilterState>) => {
    setState((now) => ({ ...now, ...part }));
  };
  const needsParty = entry.party_required && state.party === null;
  const balanceOnly = entry.filters.includes('detail') && !settled.detail;
  const filtering = !isUnfiltered(state, entry.filters);

  let body;
  if (view === 'preview' && wire !== null) {
    body = (
      <ReportPreview
        id={entry.id}
        filters={wire}
        onBack={() => {
          setView('table');
        }}
      />
    );
  } else if (needsParty) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-pick-party" />}
        help={<Localized id="report-pick-party-help" />}
      />
    );
  } else {
    body = (
      <ReportResults
        takes={entry.filters}
        query={query}
        kept={filterRefusal === null ? null : kept}
        filtering={filtering}
        balanceOnly={balanceOnly}
        onClear={() => {
          setState(EMPTY_FILTERS);
        }}
        onOpen={open}
        focusRowId={focusRowId}
        onFocusRow={setFocusRowId}
      />
    );
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id={entry.title_id}>
            <h1 className="type-title" />
          </Localized>
          <Localized id={entry.title_id.replace(/-title$/, '-help')}>
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button onClick={backToList}>
            <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
            <Localized id="report-back" />
          </Button>
          <Button
            shortcut={SHORTCUTS.print}
            disabled={wire === null || view === 'preview'}
            onClick={() => {
              setView('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
        </div>
      </div>
      {view === 'preview' ? null : (
        <ReportFilterBar
          entry={entry}
          state={{ ...state, party }}
          onChange={change}
          error={filterRefusal}
          containerRef={containerRef}
        />
      )}
      {body}
    </div>
  );
}

export function ReportPage() {
  const { id } = useParams({ strict: false });
  const catalog = useReportCatalog();
  const navigate = useNavigate();
  if (catalog.isPending) return <LoadingState />;
  if (catalog.isError) {
    return (
      <ErrorState
        title={<Localized id="reports-catalog-failed" />}
        error={uiErrorOf(catalog.error)}
        onRetry={() => {
          void catalog.refetch();
        }}
      />
    );
  }
  const entry = catalog.data.find((candidate) => candidate.id === id);
  if (entry === undefined) {
    return (
      <EmptyState
        icon={FileX}
        title={<Localized id="report-load-failed" />}
        help={<Localized id="reports-catalog-empty-help" />}
        action={
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/reports' });
            }}
          >
            <Localized id="report-back" />
          </Button>
        }
      />
    );
  }
  // A key per report: moving from one report to another starts with clean filters.
  return <ReportBody key={entry.id} entry={entry} catalog={catalog.data} />;
}
