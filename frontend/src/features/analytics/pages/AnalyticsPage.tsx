// The تجزیات area (routes /analytics and /analytics/$view; PRD 18, FR-18.1 to FR-18.8): the views A to
// E and H on one screen, in tabs. Each view has its filter boxes (the catalogue says which), a chart,
// a table page by page with the backend's totals, drill-down on any figure, saved views and a print
// preview. It works out nothing (CLAUDE.md rule 1) and writes nothing but the saved views (FR-18.8).
//
// - The filters are kept above the screen (store.ts), so they stay between views and through a
//   language switch (which builds the grid and the chart again), and the focused row is focused again
//   (Code Standards F39).
// - A filter change asks the backend once, 250 ms after the last key (F35). A refusal that points at a
//   filter box is shown under that box and the rows already on screen stay.
// - Enter on a figure opens the lines behind it, Esc returns; F9 shows the print preview, Esc leaves it.
//
// Four states: loading, an error with a retry (also for a view that does not exist), nothing found
// (with a way out), and the data.

import { Localized, useLocalization } from '@fluent/react';
import type { InfiniteData } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import { FileX, Printer, Rows3 } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { AnalyticsCatalogEntry } from '../../../bindings/AnalyticsCatalogEntry';
import type { AnalyticsPage as AnalyticsPageData } from '../../../bindings/AnalyticsPage';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useDebouncedValue } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import type { PickerValue } from '../../../shared/ui';
import { Button, EmptyState, ErrorState, LoadingState, Tabs } from '../../../shared/ui';
import { AnalyticsFilterBar } from '../components/AnalyticsFilterBar';
import { AnalyticsLines } from '../components/AnalyticsLines';
import { AnalyticsPreview } from '../components/AnalyticsPreview';
import { AnalyticsResults, WHOLE_FILTER_KEY } from '../components/AnalyticsResults';
import { SavedViews } from '../components/SavedViews';
import {
  EMPTY_ANALYTICS_FILTERS,
  filtersToWire,
  isUnfiltered,
  stateFromWire,
  type AnalyticsFilterState,
} from '../filters';
import { useAnalyticsCatalog, useAnalyticsPages, type LinesRequest } from '../hooks';
import { useAnalyticsStore } from '../store';
import type { AnalyticsKey, AnalyticsSavedView, AnalyticsView } from '../types';

/** How long the filters rest before the backend is asked (Code Standards F35). */
const FILTER_SETTLE_MS = 250;

/** The view the area opens on. */
const DEFAULT_VIEW: AnalyticsView = 'A';

/** True when the refusal points at a filter box: the box shows it and the rows stay. */
function isFilterRefusal(error: UiError): boolean {
  return (
    error.errors.length > 0 &&
    error.errors.every(
      (item) => typeof item.field === 'string' && item.field.startsWith('filters.'),
    )
  );
}

/** A picker value whose names were not known yet (a saved view), filled from the answer's echo. */
function named(
  value: PickerValue | null,
  echo: { ur: string; en: string; code?: string | undefined } | null,
): PickerValue | null {
  if (value === null || echo === null || value.name_ur !== '') return value;
  return { id: value.id, name_ur: echo.ur, name_en: echo.en, code: echo.code ?? value.code };
}

interface Drill {
  request: LinesRequest;
  about: string;
}

function AnalyticsBody({
  entry,
  catalog,
}: {
  entry: AnalyticsCatalogEntry;
  catalog: readonly AnalyticsCatalogEntry[];
}) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const filters = useAnalyticsStore((now) => now.filters);
  const windowStart = useAnalyticsStore((now) => now.windowStart);
  const otherColumns = useAnalyticsStore((now) => now.otherColumns);
  const savedName = useAnalyticsStore((now) => now.savedName);
  const { change, replace, setWindowStart, setOtherColumns, setSavedName } =
    useAnalyticsStore.getState();
  const [mode, setMode] = useState<'table' | 'preview'>('table');
  const [drill, setDrill] = useState<Drill | null>(null);
  const [focusRowId, setFocusRowId] = useState<string | null>(null);
  const settled = useDebouncedValue(filters, FILTER_SETTLE_MS);
  const wire = useMemo(() => filtersToWire(settled, entry.filters), [settled, entry]);
  const query = useAnalyticsPages(entry.id, wire);

  // The pages last shown, kept while a half-typed filter is refused (they stay under the box). They
  // are state set while rendering, which React allows when the value really changed.
  const [kept, setKept] = useState<InfiniteData<AnalyticsPageData> | null>(null);
  if (query.data !== undefined && !query.isPlaceholderData && query.data !== kept) {
    setKept(query.data);
  }
  const refusal = query.isError ? uiErrorOf(query.error) : null;
  const filterRefusal = refusal !== null && isFilterRefusal(refusal) ? refusal : null;

  // A record opened from a saved view has no names until the answer names it.
  const echo = (query.data ?? kept)?.pages[0]?.filters ?? null;
  const shown: AnalyticsFilterState = {
    ...filters,
    item: named(filters.item, echo?.item ?? null),
    seller: named(
      filters.seller,
      echo?.seller == null
        ? null
        : { ur: echo.seller.name_ur, en: echo.seller.name_en, code: echo.seller.code },
    ),
    buyer: named(
      filters.buyer,
      echo?.buyer == null
        ? null
        : { ur: echo.buyer.name_ur, en: echo.buyer.name_en, code: echo.buyer.code },
    ),
    city: named(filters.city, echo?.city ?? null),
    adda: named(filters.adda, echo?.adda ?? null),
  };

  // The first box has focus when the area opens, so the user can start at once (PRD FR-7.1,
  // NFR-USE-7). A drop-down is a button, which the shared first-field hook does not look for. Nothing is
  // taken from where focus already is: moving between the tabs with the arrow keys keeps focus on them.
  const containerRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (document.activeElement !== document.body) return;
    containerRef.current
      ?.querySelector<HTMLElement>('[role="combobox"], input:not([type="hidden"])')
      ?.focus({ preventScroll: true });
  }, []);
  useEscape(
    'leave',
    () => {
      if (mode === 'preview') setMode('table');
      else setDrill(null);
    },
    mode === 'preview' || drill !== null,
  );
  useShortcut(SHORTCUTS.print, () => {
    if (drill === null) setMode('preview');
  });

  const filtering = !isUnfiltered(filters, entry.filters);
  const openDrill = (rowKey: AnalyticsKey, columnKey: AnalyticsKey | null, about: string) => {
    setDrill({ request: { view: entry.id, filters: wire, rowKey, columnKey }, about });
  };
  const openSaved = (saved: AnalyticsSavedView) => {
    replace(stateFromWire(saved.filters), saved.name);
    setDrill(null);
    setMode('table');
    if (saved.view !== entry.id) {
      void navigate({ to: '/analytics/$view', params: { view: saved.view } });
    }
  };

  let body;
  if (drill !== null) {
    body = (
      <AnalyticsLines
        request={drill.request}
        about={drill.about}
        onBack={() => {
          setDrill(null);
        }}
      />
    );
  } else if (mode === 'preview') {
    body = (
      <AnalyticsPreview
        view={entry.id}
        filters={wire}
        onBack={() => {
          setMode('table');
        }}
      />
    );
  } else {
    body = (
      <>
        <SavedViews
          current={{ view: entry.id, filters: filtersToWire(filters, entry.filters) }}
          savedName={savedName}
          onOpen={openSaved}
          onSaved={setSavedName}
        />
        <AnalyticsFilterBar
          takes={entry.filters}
          state={shown}
          onChange={change}
          error={filterRefusal}
          containerRef={containerRef}
        />
        <AnalyticsResults
          view={entry.id}
          query={query}
          kept={filterRefusal === null ? null : kept}
          filtering={filtering}
          onClear={() => {
            replace(EMPTY_ANALYTICS_FILTERS);
          }}
          windowStart={windowStart}
          onWindow={setWindowStart}
          otherColumns={otherColumns}
          onOtherColumns={setOtherColumns}
          onDrill={openDrill}
          focusRowId={focusRowId}
          onFocusRow={setFocusRowId}
        />
      </>
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
          <Button
            disabled={drill !== null || mode === 'preview'}
            onClick={() => {
              openDrill(WHOLE_FILTER_KEY, null, l10n.getString('analytics-drill-all'));
            }}
          >
            <Rows3 aria-hidden="true" className="size-icon-md" />
            <Localized id="analytics-all-lines" />
          </Button>
          <Button
            shortcut={SHORTCUTS.print}
            disabled={drill !== null || mode === 'preview'}
            onClick={() => {
              setMode('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
        </div>
      </div>
      <Tabs
        label={l10n.getString('analytics-views-label')}
        value={entry.id}
        onValueChange={(view) => {
          setDrill(null);
          setMode('table');
          void navigate({ to: '/analytics/$view', params: { view } });
        }}
        items={catalog.map((candidate) => ({
          id: candidate.id,
          label: l10n.getString(candidate.title_id),
          content: null,
        }))}
      />
      {body}
    </div>
  );
}

export function AnalyticsPage() {
  const { view } = useParams({ strict: false });
  const catalog = useAnalyticsCatalog();
  const navigate = useNavigate();
  if (catalog.isPending) return <LoadingState />;
  if (catalog.isError) {
    return (
      <ErrorState
        title={<Localized id="analytics-catalog-failed" />}
        error={uiErrorOf(catalog.error)}
        onRetry={() => {
          void catalog.refetch();
        }}
      />
    );
  }
  const wanted = (view ?? DEFAULT_VIEW).toUpperCase();
  const entry = catalog.data.find((candidate) => candidate.id === wanted);
  if (entry === undefined) {
    return (
      <EmptyState
        icon={FileX}
        title={<Localized id="analytics-view-unknown" />}
        help={<Localized id="analytics-view-unknown-help" />}
        action={
          <Button
            variant="primary"
            onClick={() => {
              void navigate({ to: '/analytics/$view', params: { view: DEFAULT_VIEW } });
            }}
          >
            <Localized id="analytics-back-to-views" />
          </Button>
        }
      />
    );
  }
  // A key per view: moving from one view to another starts with its own table (the filters stay).
  return <AnalyticsBody key={entry.id} entry={entry} catalog={catalog.data} />;
}
