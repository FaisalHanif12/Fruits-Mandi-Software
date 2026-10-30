// R38, the dead amount (route /reports/dead-amount; PRD FR-10.10, R38): everyone who was moved to
// ڈوبی رقم, with what he owed on that day, what was recovered since, the corrections, and what he
// still owes. The figures and the totals are the backend's, over the whole filter (PRD FR-19.11); the
// rows on screen are never added up. The tile ڈوبی رقم of the dashboard opens this screen, and its
// total is the tile's number (FR-19.9).
//
// - Filters: still owing (the default) or all, and one search box (name in either script, code, tag
//   or phone). The filters live here, above the table, so a language switch loses nothing.
// - ↑ ↓ choose a person; Enter opens his ledger (R13); the button "دوبارہ فعال کریں" makes the chosen
//   person active again (with a reason). Everything works with the keyboard alone.
// - F9 or the button shows the print preview of R38 (A4, A5, Urdu or English, PDF) and the Excel
//   button saves it, both from the report engine (`R38`) with the same "still owing / all" choice
//   as the screen. The search box narrows the screen only: a print is for the whole filter.
// - Four states: loading, an error with a retry, nothing yet (with what to do next), and the rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowLeft, BookOpen, Printer, SearchX, UserRoundX } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { DeadFilter } from '../../../bindings/DeadFilter';
import type { DeadRow } from '../../../bindings/DeadRow';
import type { ReportFilters } from '../../../bindings/ReportFilters';
import { uiErrorOf } from '../../../shared/api';
import {
  ExcelExportButton,
  ExcelExportNotices,
  exportFileName,
  useExcelExport,
} from '../../../shared/export';
import { PagedList, type ListContext } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  Field,
  FilterBar,
  ListPage,
  Select,
  useDeadAmountActions,
} from '../../../shared/ui';
import { reportsExportExcel } from '../api';
import { ReportPreview } from '../components/ReportPreview';
import { DEAD_COLUMNS } from '../dead-columns';
import { useDeadAmount } from '../dead-hooks';
import { useOpenReport } from '../open-report';

/** The move a row stands for, found by the id the list gives it. */
const rowId = (row: DeadRow) => String(row.mark_id);

/**
 * What the report engine is asked for R38: no dates, no party. "Still owing" is the status `due`;
 * "all" sends no status, which is every move (PRD R38).
 */
function wireOf(filter: DeadFilter): ReportFilters {
  return {
    shortcut: null,
    from: '',
    to: '',
    as_of: '',
    party: null,
    seller_id: null,
    zamindar_id: null,
    item_id: null,
    city_id: null,
    adda_id: null,
    detail: false,
    partner_id: null,
    garden_id: null,
    bikri_id: null,
    ...(filter === 'owed' ? { status: 'due' as const } : {}),
  };
}

export function DeadAmountPage() {
  const navigate = useNavigate();
  const open = useOpenReport();
  const actions = useDeadAmountActions();
  const [filter, setFilter] = useState<DeadFilter>('owed');
  const [search, setSearch] = useState('');
  const [focusId, setFocusId] = useState<string | null>(null);
  const [view, setView] = useState<'table' | 'preview'>('table');
  const language = useLanguage();
  const list = useDeadAmount({ filter, search: search.trim() });
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const wire = useMemo(() => wireOf(filter), [filter]);
  // Excel: every move of the filter as it is in the box now, in the app language (or the one chosen
  // in the preview), with its progress shown under the title.
  const exporter = useExcelExport({
    defaultName: () => exportFileName('R38', new Date()),
    run: ({ path, jobId, language: sheet }) =>
      reportsExportExcel({
        job_id: jobId,
        id: 'R38',
        filters: wire,
        language: sheet,
        path,
        sort: null,
        group_by_month: false,
      }),
  });
  useEscape('leave', () => {
    if (view === 'preview') setView('table');
    else void navigate({ to: '/' });
  });
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const focused = rows?.find((row) => rowId(row) === focusId);
  const filtering = filter !== 'owed' || search.trim() !== '';

  const openLedger = useCallback(
    (row: DeadRow) => {
      open?.('R13', { party: row.account_id });
    },
    [open],
  );
  const makeActive = (row: DeadRow) => {
    actions?.undo({
      markId: row.mark_id,
      names:
        row.pukhta_person_key === null
          ? row
          : {
              name_ur: row.pukhta_name_ur ?? row.name_ur,
              name_en: row.pukhta_name_en ?? row.name_en,
            },
    });
  };

  // The totals row is the backend's, over every move of the filter and not over the rows loaded.
  const totals = useCallback(
    (context: ListContext) => {
      const sums = first?.totals;
      return {
        labelField: 'party',
        values:
          sums === undefined
            ? {}
            : {
                party: context.text('glossary-meezan'),
                owed_then: context.money(sums.owed_then_paisa),
                recovered: context.money(sums.recovered_paisa),
                corrections: context.signedMoney(sums.corrections_paisa),
                still_owed: context.money(sums.still_owed_paisa),
              },
      };
    },
    [first],
  );

  const empty = useMemo(
    () =>
      filtering ? (
        <EmptyState
          icon={SearchX}
          title={<Localized id="dead-report-no-match" />}
          help={<Localized id="dead-report-no-match-help" />}
          action={
            <Button
              onClick={() => {
                setFilter('owed');
                setSearch('');
              }}
            >
              <Localized id="dead-report-clear" />
            </Button>
          }
        />
      ) : (
        <EmptyState
          icon={UserRoundX}
          title={<Localized id="dead-report-empty" />}
          help={<Localized id="dead-report-empty-help" />}
        />
      ),
    [filtering],
  );

  return (
    <ListPage
      titleId="dead-report-title"
      help={
        <Localized id="dead-report-help">
          <p />
        </Localized>
      }
      actions={
        <>
          <Button
            onClick={() => {
              if (view === 'preview') setView('table');
              else void navigate({ to: '/' });
            }}
          >
            <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
            <Localized id="dead-report-back" />
          </Button>
          {view === 'table' ? (
            <Button
              disabled={focused === undefined || open === null}
              onClick={() => {
                if (focused !== undefined) openLedger(focused);
              }}
            >
              <BookOpen aria-hidden="true" className="size-icon-md" />
              <Localized id="dead-open-ledger" />
            </Button>
          ) : null}
          {view === 'table' ? (
            <Button
              shortcut={SHORTCUTS.print}
              onClick={() => {
                setView('preview');
              }}
            >
              <Printer aria-hidden="true" className="size-icon-md" />
              <Localized id="report-preview" />
            </Button>
          ) : null}
          {view === 'table' ? <ExcelExportButton exporter={exporter} language={language} /> : null}
          {view === 'table' ? (
            <Button
              variant="primary"
              disabled={focused === undefined || focused.status === 'undone' || actions === null}
              onClick={() => {
                if (focused !== undefined) makeActive(focused);
              }}
            >
              <Localized id="dead-undo-button" />
            </Button>
          ) : null}
        </>
      }
      filters={
        <div className="flex flex-col gap-3">
          <ExcelExportNotices exporter={exporter} />
          {view === 'table' ? (
            <div ref={containerRef} {...formNavigationProps}>
              <FilterBar
                screen="reports.dead_amount"
                search={{
                  value: search,
                  onChange: setSearch,
                  placeholderId: 'dead-report-search',
                }}
              >
                <Field label={<Localized id="dead-report-filter" />}>
                  {(control) => (
                    <Select
                      {...control}
                      value={filter}
                      onChange={setFilter}
                      options={[
                        { value: 'owed', label: <Localized id="dead-filter-owed" /> },
                        { value: 'all', label: <Localized id="dead-filter-all" /> },
                      ]}
                    />
                  )}
                </Field>
              </FilterBar>
            </div>
          ) : null}
        </div>
      }
    >
      {view === 'preview' ? (
        <ReportPreview
          id="R38"
          filters={wire}
          sort={null}
          groupByMonth={false}
          onBack={() => {
            setView('table');
          }}
          exporter={exporter}
        />
      ) : (
        <PagedList<DeadRow>
          columns={DEAD_COLUMNS}
          rows={rows}
          getRowId={rowId}
          labelId="dead-report-label"
          failedId="dead-report-failed"
          error={list.isError ? uiErrorOf(list.error) : null}
          onRetry={() => {
            void list.refetch();
          }}
          pager={list.pager}
          empty={empty}
          onOpen={openLedger}
          onFocusRow={setFocusId}
          focusRowId={focusId}
          isInactive={(row) => row.status === 'undone'}
          totals={totals}
        />
      )}
    </ListPage>
  );
}
