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
// - The Excel button (D5-Q1) saves every row of the filters as they are in the boxes, in the app
//   language (in the preview: the language chosen there), with its progress shown under the title.
//
// Four states: loading, an error with a retry (also for a report that does not exist), nothing
// found (with a way out), and the rows.

import { Localized } from '@fluent/react';
import { useNavigate, useParams, useSearch } from '@tanstack/react-router';
import { ArrowLeft, FileX, Files, Printer } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import type { ReportPage as ReportPageData } from '../../../bindings/ReportPage';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import {
  ExcelExportButton,
  ExcelExportNotices,
  exportFileName,
  useExcelExport,
} from '../../../shared/export';
import { useDebouncedValue, useFocusFirstField } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import { Remembered, type ScreenMemory } from '../../../shared/screen-state';
import { Button, EmptyState, ErrorState, ListPage, LoadingState } from '../../../shared/ui';
import { PrintAction } from '../../bikri';
import { useOpenArrival } from '../../gardens';
import { DeadMoveButton } from '../components/DeadMoveButton';
import { ReportFilterBar } from '../components/ReportFilterBar';
import { BuyerBillsPreview } from '../components/BuyerBillsPreview';
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
import { reportsExportExcel } from '../api';
import { MOVABLE_REPORTS, partyOfFocusedRow } from '../dead-row';
import { useReportCatalog, useReportPages } from '../hooks';
import { filtersFromRemembered, reportScreen, rememberedFilters } from '../remember';
import { parseReportSearch, stateFromSearch } from '../search';

/** How long the filters rest before the backend is asked (Code Standards F35). */
const FILTER_SETTLE_MS = 250;

/**
 * The reports whose rows are in date order. They ask the backend for a heading row above each
 * month, with the count and sums of the whole month (plan 38 R4); a sort by another column turns
 * the headings off by itself.
 */
const MONTH_REPORTS: ReadonlySet<string> = new Set([
  'R2',
  'R3',
  'R4',
  'R11',
  'R12',
  'R13',
  'R14',
  'R15',
  'R23',
  'R25',
]);

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
function useOpenSource(catalog: readonly ReportCatalogEntry[], reportId: ReportCatalogEntry['id']) {
  const navigate = useNavigate();
  const openArrival = useOpenArrival();
  return (view: ReportRowView) => {
    const source = view.row.source;
    if (source === null) return;
    switch (source.kind) {
      case 'arrival':
        // A row of the goods left report that is a garden's goods arrived: its garden screen.
        openArrival(source.id);
        return;
      case 'bikri':
        void navigate({ to: '/bikri/book', search: { id: source.id } });
        return;
      case 'voucher':
        void navigate({ to: '/bikri/voucher', search: { id: source.id } });
        return;
      case 'venture':
        // A row of the gardens report is a garden: its screen has its entries and its closing.
        if (reportId === 'R36') {
          void navigate({ to: '/gardens/$id', params: { id: String(source.id) } });
        }
        return;
      case 'fruit':
        // A row of the fruit register is one fruit entry: its screen opens it in its saved view.
        void navigate({ to: '/rokar/fruit', search: { entry: source.id } });
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

/** What a report that cannot run yet asks for: a partner, a بکری or an account. */
function pickPromptOf(partner: boolean, bikri: boolean): string {
  if (bikri) return 'report-pick-bikri';
  return partner ? 'report-pick-partner' : 'report-pick-party';
}

function ReportBody({
  entry,
  catalog,
  saved,
  remember,
}: {
  entry: ReportCatalogEntry;
  catalog: readonly ReportCatalogEntry[];
  /** What the report kept last time, as it was written (PRD FR-7.45). */
  saved: unknown;
  remember: ScreenMemory['remember'];
}) {
  const navigate = useNavigate();
  const search = parseReportSearch(useSearch({ strict: false }));
  // An address with filters in it (the party card's buttons) wins; otherwise the report starts as it
  // was left (PRD FR-7.45).
  const [state, setState] = useState<ReportFilterState>(() =>
    Object.keys(search).length > 0
      ? stateFromSearch(search)
      : (filtersFromRemembered(saved) ?? stateFromSearch(search)),
  );
  useEffect(() => {
    remember(rememberedFilters(state));
  }, [state, remember]);
  const [view, setView] = useState<'table' | 'preview' | 'slips'>('table');
  const [focusRowId, setFocusRowId] = useState<string | null>(null);
  const settled = useDebouncedValue(state, FILTER_SETTLE_MS);
  const wire = useMemo(
    () => (canRun(settled, entry) ? filtersToWire(settled, entry.filters) : null),
    [settled, entry],
  );
  const monthsOn = MONTH_REPORTS.has(entry.id);
  const query = useReportPages(entry.id, wire, monthsOn);
  const openSource = useOpenSource(catalog, entry.id);
  const language = useLanguage();
  // Excel (D5-Q1): every row of the filters as they are in the boxes NOW, not the settled ones.
  const exporter = useExcelExport({
    defaultName: () => exportFileName(entry.id, new Date()),
    run: ({ path, jobId, language: sheet }) =>
      canRun(state, entry)
        ? reportsExportExcel({
            job_id: jobId,
            id: entry.id,
            filters: filtersToWire(state, entry.filters),
            language: sheet,
            path,
            // The sheet has the order and the month headings that are on screen (plan 38 R4).
            sort: query.sort,
            group_by_month: monthsOn && query.sort === null,
          })
        : Promise.reject(new Error('the report cannot run before its filters are filled')),
  });

  // The page last shown, kept while a half-typed filter is refused (they stay under the box). They
  // are state set while rendering, which React allows when the value really changed.
  const [kept, setKept] = useState<{ pages: ReportPageData[] } | null>(null);
  if (query.data !== undefined && !query.isPlaceholderData && query.data !== kept) {
    setKept(query.data);
  }
  // The person of the row that has focus, for "Move to dead amount" on the dues reports (FR-10.10).
  const focusedParty = MOVABLE_REPORTS.has(entry.id)
    ? partyOfFocusedRow((query.data ?? kept)?.pages, focusRowId)
    : null;
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
    if (view === 'table') backToList();
    else setView('table');
  });
  // The sale book report (R1) prints the seller's bill itself, the shared print job of the بکری picked
  // (`PrintAction`, which has its own F9), not a page of the report.
  const printsBill = entry.id === 'R1';
  useShortcut(SHORTCUTS.print, () => {
    if (wire !== null && !printsBill && view === 'table') setView('preview');
  });

  const change = (part: Partial<ReportFilterState>) => {
    setState((now) => ({ ...now, ...part }));
  };
  // The party day sheet (R35): Enter on a party of the table opens his slip, the same sheet for one
  // party (PRD FR-19.12), where every other report opens the record behind a row. A one-time person
  // is picked with his key, so the slip is his and not the whole account's (FR-13.5).
  const open = (view: ReportRowView) => {
    const cell = view.row.cells.find((candidate) => candidate.type === 'party');
    if (entry.id === 'R35' && state.party === null && cell?.type === 'party') {
      const picked = cell.party;
      change({
        party: {
          id: picked.account_id,
          name_ur: picked.name_ur,
          name_en: picked.name_en,
          code: picked.code,
        },
        person:
          picked.person_key === null
            ? null
            : {
                accountId: picked.account_id,
                personKey: picked.person_key,
                name_ur: picked.name_ur,
                name_en: picked.name_en,
                phone: '',
                last_date: '',
              },
        personText: '',
      });
      return;
    }
    openSource(view);
  };
  // R13 and R14 wait for an account, R31 for a partner (the catalogue says which box it takes).
  const needsPick = !canRun(state, entry);
  const pickingPartner = entry.filters.includes('partner');
  const pickingBikri = entry.filters.includes('bikri');
  const balanceOnly = entry.filters.includes('detail') && !settled.detail;
  const filtering = !isUnfiltered(state, entry.filters);

  let body;
  if (view === 'slips' && wire !== null) {
    body = (
      <BuyerBillsPreview
        filters={wire}
        onBack={() => {
          setView('table');
        }}
      />
    );
  } else if (view === 'preview' && wire !== null) {
    body = (
      <ReportPreview
        id={entry.id}
        filters={wire}
        sort={query.sort}
        groupByMonth={monthsOn && query.sort === null}
        onBack={() => {
          setView('table');
        }}
        exporter={exporter}
      />
    );
  } else if (needsPick) {
    body = (
      <EmptyState
        icon={FileX}
        title={<Localized id={pickPromptOf(pickingPartner, pickingBikri)} />}
        help={<Localized id={`${pickPromptOf(pickingPartner, pickingBikri)}-help`} />}
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

  const tableView = view === 'table';
  return (
    <ListPage
      titleId={entry.title_id}
      help={
        <>
          <Localized id={entry.title_id.replace(/-title$/, '-help')}>
            <p />
          </Localized>
          <Localized id="report-open-hint">
            <p className="text-ink-muted" />
          </Localized>
        </>
      }
      actions={
        <>
          <Button onClick={backToList}>
            <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
            <Localized id="report-back" />
          </Button>
          {MOVABLE_REPORTS.has(entry.id) && tableView ? (
            <DeadMoveButton party={focusedParty} />
          ) : null}
          {printsBill && state.bikri !== null ? (
            <PrintAction target={{ id: state.bikri.id, number: state.bikri.number }} />
          ) : (
            <Button
              shortcut={SHORTCUTS.print}
              disabled={wire === null || !tableView}
              onClick={() => {
                setView('preview');
              }}
            >
              <Printer aria-hidden="true" className="size-icon-md" />
              <Localized id="report-preview" />
            </Button>
          )}
          {/* The credit buyers' details (R5) can print one slip for each buyer (PRD 19.2 R5). */}
          {entry.id === 'R5' ? (
            <Button
              disabled={wire === null || !tableView}
              onClick={() => {
                setView('slips');
              }}
            >
              <Files aria-hidden="true" className="size-icon-md" />
              <Localized id="report-print-all" />
            </Button>
          ) : null}
          {/* In the preview the frame has its own Excel button, with the print language chosen there. */}
          {tableView ? (
            <ExcelExportButton
              exporter={exporter}
              language={language}
              disabled={!canRun(state, entry)}
            />
          ) : null}
        </>
      }
      filters={
        <div className="flex flex-col gap-3">
          <ExcelExportNotices exporter={exporter} />
          {tableView ? (
            <ReportFilterBar
              entry={entry}
              state={{ ...state, party }}
              onChange={change}
              error={filterRefusal}
              containerRef={containerRef}
            />
          ) : null}
          {entry.id === 'R35' && state.party !== null && tableView ? (
            <div>
              <Button
                onClick={() => {
                  change({ party: null, person: null, personText: '' });
                }}
              >
                <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
                <Localized id="report-r35-all-parties" />
              </Button>
            </div>
          ) : null}
        </div>
      }
    >
      {tableView ? body : <div className="min-h-0 flex-1 overflow-auto">{body}</div>}
    </ListPage>
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
  return (
    <Remembered key={entry.id} screen={reportScreen(entry.id)}>
      {({ saved, remember }) => (
        <ReportBody entry={entry} catalog={catalog.data} saved={saved} remember={remember} />
      )}
    </Remembered>
  );
}
