// The اگراہی sheet (route /reports/collection-sheet, found by R5; PRD R5, FR-19.11, FR-19.15, plan 76 point 2): the
// day's list of buyers on one compact page. One row for each buyer who owes or bought that day, also one
// who paid in full, sorted by name: his code, his name with tag, what he bought in short ("(Banana 20 .
// 600)"), what he owed at the start of the day, the day's bill, what he paid on the spot and what he owes
// now. There is no وصولی column: a وصولی of the day shows in the next day's previous due. A person in the
// dead amount is marked and listed apart in the totals. The rows, the page counts and the totals are the
// backend's, over every buyer of the filter; the rows on screen are never added up (CLAUDE.md rule 1).
//
// - The day is in the address (`?date=2026-10-10`), so a language switch, a reload and the way back from a
//   ledger land on the same day. One search box (code, running number, name in either script).
// - ↑ ↓ choose a buyer; Enter opens his ledger (R13). Page Up and Page Down turn the pages. F9 or the
//   button shows the print preview (A4 or A5, Urdu or English, PDF), with the same day and search.
// - Four states: loading, an error with a retry, nothing for the day (or no match, with a way out), and
//   the rows.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowLeft, FileText, Files, Printer, SearchX } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { CollectionSheetRow } from '../../../bindings/CollectionSheetRow';
import { uiErrorOf } from '../../../shared/api';
import { isIsoDay, isoToDisplayDate } from '../../../shared/format';
import { ExcelExportButton, ExcelExportNotices } from '../../../shared/export';
import { PagedList, type ListContext } from '../../../shared/grid';
import { useFocusFirstField, useWindowParam } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Balance,
  Button,
  EmptyState,
  FilterBar,
  GoToDate,
  ListPage,
  Num,
  RUNNING_NUMBER,
} from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { reportsCollectionSheetPrint, reportsCollectionSheetPrintData } from '../api';
import { DaySlipsPreview } from '../components/DaySlipsPreview';
import { TablePrintPreview } from '../components/TablePrintPreview';
import { COLLECTION_COLUMNS } from '../collection-columns';
import { useCollectionSheet } from '../collection-hooks';
import { useOpenReport } from '../open-report';
import { useTableExcel } from '../table-excel';

/**
 * The id of a row. The grid also asks for the id of the pinned totals row, which has no buyer: it gets a
 * name of its own, so the totals row is never taken for a buyer.
 */
function rowId(row: CollectionSheetRow): string {
  const { party } = row as { party?: CollectionSheetRow['party'] };
  if (party === undefined) return 'totals';
  return party.person_key ?? String(party.account_id);
}

export function CollectionSheetPage() {
  const navigate = useNavigate();
  const open = useOpenReport();
  const { l10n } = useLocalization();
  const today = useDayStatus().data?.today ?? null;
  const [asked, setAsked] = useWindowParam('date', isIsoDay);
  const [search, setSearch] = useState('');
  const [focusId, setFocusId] = useState<string | null>(null);
  const [view, setView] = useState<'table' | 'preview' | 'parchi'>('table');
  // Nothing asked is today, which the backend knows: an empty day is read as today.
  const day = asked ?? '';
  const list = useCollectionSheet({ date: day, search: search.trim() });
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const language = useLanguage();
  // Excel: every buyer of the day and the search on screen.
  const exporter = useTableExcel('collection-sheet', (sheet) => ({
    kind: 'collection_sheet',
    date: day,
    search: search.trim() === '' ? null : search.trim(),
    sort: null,
    language: sheet,
    paper: null,
  }));
  useEscape('leave', () => {
    if (view !== 'table') setView('table');
    else void navigate({ to: '/reports' });
  });
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the day before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const filtering = search.trim() !== '';
  const shown = first?.date ?? asked ?? today;
  const isToday = asked === null || asked === today;

  const openLedger = useCallback(
    (row: CollectionSheetRow) => {
      open?.('R13', { party: row.party.account_id });
    },
    [open],
  );

  // The totals row is the backend's, over every buyer of the filter who is not in the dead amount.
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
                previous: context.money(sums.previous_due.amount_paisa),
                bill: context.money(sums.bill_paisa),
                paid: context.money(sums.paid_paisa),
                total_due: context.money(sums.total_due.amount_paisa),
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
          title={<Localized id="collection-no-match" />}
          help={<Localized id="collection-no-match-help" />}
          action={
            <Button
              onClick={() => {
                setSearch('');
              }}
            >
              <Localized id="collection-clear" />
            </Button>
          }
        />
      ) : (
        <EmptyState
          icon={FileText}
          title={<Localized id="collection-empty" />}
          help={<Localized id="collection-empty-help" />}
        />
      ),
    [filtering],
  );

  if (view === 'parchi') {
    return (
      <ListPage icon={FileText} titleId="collection-title">
        <DaySlipsPreview
          variant="parchi"
          scope={{ date: day, bikri_id: null, bikri_ids: null, buyer_ids: null }}
          onBack={() => {
            setView('table');
          }}
        />
      </ListPage>
    );
  }

  if (view === 'preview') {
    return (
      <ListPage icon={FileText} titleId="collection-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['collection_sheet', day, search.trim()]}
          report="R5"
          read={(printLanguage, paper) =>
            reportsCollectionSheetPrintData({
              date: day,
              search: search.trim() === '' ? null : search.trim(),
              sort: null,
              language: printLanguage,
              paper,
            })
          }
          send={(to, printLanguage, paper) =>
            reportsCollectionSheetPrint({
              sheet: {
                date: day,
                search: search.trim() === '' ? null : search.trim(),
                sort: null,
                language: printLanguage,
                paper,
              },
              to,
              copies: null,
            })
          }
          onBack={() => {
            setView('table');
          }}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={FileText}
      titleId="collection-title"
      help={
        <Localized id="collection-help">
          <p />
        </Localized>
      }
      actions={
        <>
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              void navigate({ to: '/reports' });
            }}
          >
            <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
            <Localized id="collection-back" />
          </Button>
          <Button
            shortcut={SHORTCUTS.print}
            onClick={() => {
              setView('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
          <Button
            onClick={() => {
              setView('parchi');
            }}
          >
            <Files aria-hidden="true" className="size-icon-md" />
            <Localized id="collection-parchi" />
          </Button>
          <ExcelExportButton exporter={exporter} language={language} />
        </>
      }
      filters={
        <div ref={containerRef} {...formNavigationProps} className="flex flex-col gap-3">
          <FilterBar
            screen="reports.collection_sheet"
            search={{ value: search, onChange: setSearch, placeholderId: 'collection-search' }}
          >
            <GoToDate
              today={today}
              onGo={(next) => {
                // Today is the address with no day, so a reload on it is always the latest one.
                setAsked(next === today ? null : next);
              }}
              label={<Localized id="collection-goto" />}
              info={<Localized id="collection-goto-hint" />}
              placeholder={l10n.getString('collection-goto-placeholder')}
            />
          </FilterBar>
          <p className="flex flex-wrap items-center gap-3 type-body text-ink-muted">
            {shown === null ? null : isToday ? (
              <Localized id="report-day-now">
                <span />
              </Localized>
            ) : (
              <Localized
                id="report-day-shown"
                vars={{ date: isoToDisplayDate(shown) }}
                elems={RUNNING_NUMBER}
              >
                <span />
              </Localized>
            )}
            {isToday ? null : (
              <Button
                variant="ghost"
                onClick={() => {
                  setAsked(null);
                }}
              >
                <Localized id="collection-today" />
              </Button>
            )}
          </p>
        </div>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      <PagedList<CollectionSheetRow>
        columns={COLLECTION_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId="collection-label"
        failedId="collection-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        empty={empty}
        onOpen={openLedger}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        totals={totals}
      />
      {first?.dead_totals === null || first?.dead_totals === undefined ? null : (
        <Localized
          id="collection-dead-line"
          elems={{
            prev: <Num paisa={first.dead_totals.previous_due.amount_paisa} />,
            bill: <Num paisa={first.dead_totals.bill_paisa} />,
            due: <Num paisa={first.dead_totals.total_due.amount_paisa} />,
          }}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
      {first?.credit_totals === null || first?.credit_totals === undefined ? null : (
        // The buyers whose total due ends in جمع are in the rows with their side word and are not in the main
        // total, which stays what the buyers owe (the R20 total). Their own total is the backend's.
        <Localized
          id="collection-credit-line"
          elems={{
            prev: <Balance balance={first.credit_totals.previous_due} inherit />,
            bill: <Num paisa={first.credit_totals.bill_paisa} />,
            due: <Balance balance={first.credit_totals.total_due} inherit />,
          }}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </ListPage>
  );
}
