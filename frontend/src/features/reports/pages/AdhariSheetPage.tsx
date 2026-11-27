// ادھاری تفصیل (route /reports/adhari-sheet, found by R5; PRD R5 v1.43, FR-19.11, FR-19.15, plan 80 Q-R5-2): how
// much the shop has to take from its buyers, on one compact page. One row for each buyer who bought on the
// day, also one who paid in full, sorted by name: what he bought in short ("(Banana 20 . 600)"), today's
// bill, today's credit, what he owed at the start of the day and what he owes now (no cash column or cash total,
// PRD R5 v1.57). A وصولی of the day is not on the sheet: it shows in the next day's previous balance. Under the table
// are the buyers who owed but bought nothing today, one row each by name (plan 97 item 6) with their total, and the total credit of all buyers; a
// person in the dead amount and a buyer whose credit ends in جمع are listed apart. The rows, the counts and
// every total are the backend's; the rows on screen are never added up (CLAUDE.md rule 1).
//
// - The day is in the address (`?date=2026-10-10`), so a language switch, a reload and the way back from a
//   ledger land on the same day.
// - Excel writes every buyer of the day (the backend's `adhari_sheet` kind), the same table as the print.
// - ↑ ↓ choose a buyer; Enter opens his ledger (R13), except a چکتا buyer, who has none (FR-13.6). Page Up
//   and Page Down turn the pages. F9 or the button shows the print preview (A4 or A5, Urdu or English, PDF),
//   which is the print of ادھاری تفصیل: the same day, the whole day.
// - Four states: loading, an error with a retry, nothing for the day, and the rows.

import { Localized, useLocalization } from '@fluent/react';
import { FileText, Printer } from 'lucide-react';
import { useCallback, useState } from 'react';
import type { AdhariOtherRow } from '../../../bindings/AdhariOtherRow';
import type { AdhariSheetRow } from '../../../bindings/AdhariSheetRow';
import type { AdhariSheetTotals } from '../../../bindings/AdhariSheetTotals';
import { uiErrorOf } from '../../../shared/api';
import { ExcelExportButton, ExcelExportNotices } from '../../../shared/export';
import { isIsoDay, isoToDisplayDate } from '../../../shared/format';
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
import { reportsAdhariSheetPrint, reportsAdhariSheetPrintData } from '../api';
import { AdhariOthers } from '../components/AdhariOthers';
import { TablePrintPreview } from '../components/TablePrintPreview';
import { ADHARI_COLUMNS } from '../adhari-columns';
import { useAdhariSheet } from '../adhari-hooks';
import { useOpenReport } from '../open-report';
import { useTableExcel } from '../table-excel';

/**
 * The id of a row. The grid also asks for the id of the pinned totals row, which has no buyer: it gets a
 * name of its own, so the totals row is never taken for a buyer.
 */
function rowId(row: AdhariSheetRow): string {
  const { party } = row as { party?: AdhariSheetRow['party'] };
  if (party === undefined) return 'totals';
  return party.person_key ?? String(party.account_id);
}

/** One apart-line under the table: the figures of a group of buyers, from the backend's totals. */
function TotalsLine({ id, sums, count }: { id: string; sums: AdhariSheetTotals; count?: number }) {
  return (
    <Localized
      id={id}
      vars={count === undefined ? {} : { count: String(count) }}
      elems={{
        count: <Num inherit />,
        prev: <Balance balance={sums.previous_due} inherit />,
        bill: <Num paisa={sums.bill_paisa} />,
        credit: <Num paisa={sums.today_credit_paisa} />,
        due: <Balance balance={sums.total_due} inherit />,
      }}
    >
      <p className="type-body text-ink-muted" />
    </Localized>
  );
}

export function AdhariSheetPage() {
  const open = useOpenReport();
  const { l10n } = useLocalization();
  const today = useDayStatus().data?.today ?? null;
  const [asked, setAsked] = useWindowParam('date', isIsoDay);
  const [focusId, setFocusId] = useState<string | null>(null);
  const [view, setView] = useState<'table' | 'preview'>('table');
  // Nothing asked is today, which the backend knows: an empty day is read as today.
  const day = asked ?? '';
  const list = useAdhariSheet({ date: day });
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const language = useLanguage();
  // Excel: every buyer of the day, the same table as the print (the backend's `adhari_sheet` kind).
  const exporter = useTableExcel('adhari-sheet', (sheet) => ({
    kind: 'adhari_sheet',
    date: day,
    language: sheet,
    paper: null,
  }));
  useEscape(
    'leave',
    () => {
      setView('table');
    },
    view !== 'table',
  );
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the day before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const shown = first?.date ?? asked ?? today;
  const isToday = asked === null || asked === today;

  const openLedger = useCallback(
    (row: AdhariSheetRow) => {
      // A چکتا buyer has no ledger (PRD FR-13.6).
      if (row.party.person_key !== null) return;
      open?.('R13', { party: row.party.account_id });
    },
    [open],
  );

  // The ledger of an other buyer: he has an account, so the ledger of the party always opens (R13).
  const openOtherLedger = useCallback(
    (row: AdhariOtherRow) => {
      open?.('R13', { party: row.party.account_id });
    },
    [open],
  );

  // The totals row is the backend's: today's buyers, not the dead amount and not those whose credit ends in جمع.
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
                bill: context.money(sums.bill_paisa),
                today_credit: context.money(sums.today_credit_paisa),
                previous: context.money(sums.previous_due.amount_paisa),
                total_due: context.money(sums.total_due.amount_paisa),
              },
      };
    },
    [first],
  );

  if (view === 'preview') {
    return (
      <ListPage icon={FileText} titleId="report-r5-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['adhari-sheet', day]}
          report="R5"
          read={(printLanguage, paper) =>
            reportsAdhariSheetPrintData({ date: day, language: printLanguage, paper })
          }
          send={(to, printLanguage, paper) =>
            reportsAdhariSheetPrint({
              sheet: { date: day, language: printLanguage, paper },
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
      titleId="report-r5-title"
      help={
        <Localized id="adhari-help">
          <p />
        </Localized>
      }
      actions={
        <>
          <Button
            shortcut={SHORTCUTS.print}
            onClick={() => {
              setView('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
          <ExcelExportButton exporter={exporter} language={language} />
        </>
      }
      filters={
        <div ref={containerRef} {...formNavigationProps} className="flex flex-col gap-3">
          <FilterBar screen="reports.adhari_sheet">
            <GoToDate
              today={today}
              onGo={(next) => {
                // Today is the address with no day, so a reload on it is always the latest one.
                setAsked(next === today ? null : next);
              }}
              label={<Localized id="adhari-goto" />}
              info={<Localized id="adhari-goto-hint" />}
              placeholder={l10n.getString('adhari-goto-placeholder')}
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
                <Localized id="adhari-today" />
              </Button>
            )}
          </p>
        </div>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      <PagedList<AdhariSheetRow>
        columns={ADHARI_COLUMNS}
        rows={rows}
        getRowId={rowId}
        labelId="adhari-label"
        failedId="adhari-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        empty={
          <EmptyState
            icon={FileText}
            title={<Localized id="adhari-empty" />}
            help={<Localized id="adhari-empty-help" />}
          />
        }
        onOpen={openLedger}
        onFocusRow={setFocusId}
        focusRowId={focusId}
        totals={totals}
      />
      {first === undefined ? null : (
        <AdhariOthers rows={first.other_rows} onOpen={openOtherLedger} />
      )}
      {first?.others === null || first?.others === undefined ? null : (
        <TotalsLine id="adhari-others-line" sums={first.others} count={first.others_count} />
      )}
      {first === undefined ? null : <TotalsLine id="adhari-all-line" sums={first.all_totals} />}
      {first?.dead_totals === null || first?.dead_totals === undefined ? null : (
        <TotalsLine id="adhari-dead-line" sums={first.dead_totals} />
      )}
      {first?.credit_totals === null || first?.credit_totals === undefined ? null : (
        // The buyers whose total credit ends in جمع are in the rows with their side word and are not in the main
        // total, which stays what the buyers owe (the R20 total). Their own total is the backend's.
        <TotalsLine id="adhari-credit-line" sums={first.credit_totals} />
      )}
    </ListPage>
  );
}
