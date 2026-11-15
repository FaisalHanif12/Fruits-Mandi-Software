// The commission page (route /commission; PRD FR-9.3, FR-19.11, plan 76 point 7, plan 77): the tile آج کا کمیشن
// opens it. The Owner picks a period (one date, a month, a year or a range of dates) and, if he wants, one
// buyer or one seller, and reads the commission and sale of that period or of that party in it, with the total.
// A row for each day (a month, a date or a range) or for each month (a year); a click or Enter on a day opens
// the postings that made it, in place; a click on a month of a year opens that month. The totals row, the
// rows, the partners' shares and the periods it can go to are the backend's, read from the posted records; the
// page adds nothing up (CLAUDE.md rule 1).
//
// - The period and the party are kept in the address (`?period=year&year=2026&side=seller&party=...`), so a
//   language switch, a reload and the way back from a بکری land on the same page. The month keeps its plan 76
//   buttons, its box and Page Up and Page Down; a year and a date have the same pair; a range is paged.
// - Under a party the partner columns are left out: shares belong to whole postings, not to one party.
// - The partner columns are behind the partner code (PRD FR-17.11): without it the box "enter the code"
//   shows in their place and the rest of the page works.
// - Four states: loading, an error with a retry, no commission in the period (the buttons still work; with a
//   side and no party, a note to pick one), and the rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowLeft, Percent, Printer, UserSearch } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { addDays, addMonths, monthOf, windowMonth } from '../../../shared/format';
import { ExcelExportButton, ExcelExportNotices } from '../../../shared/export';
import { Pager } from '../../../shared/grid';
import { describePager, usePagerControl } from '../../../shared/hooks';
import { useLanguage } from '../../../shared/i18n';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  ErrorState,
  GoToMonth,
  ListPage,
  LoadingState,
  MonthNav,
  PartnerLockedBox,
} from '../../../shared/ui';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { dashboardCommissionPrint, dashboardCommissionPrintData } from '../api';
import { CommissionDays, useOpenDays } from '../components/CommissionDays';
import { CommissionFilterBar } from '../components/CommissionFilterBar';
import { DayNav, GoToCommissionDate, GoToYear, YearNav } from '../components/CommissionNav';
import { commissionInput, commissionParty, useCommissionRead } from '../commission-hooks';
import { addYears, useCommissionFilters, yearOf } from '../commission-search';

export function CommissionPage() {
  const navigate = useNavigate();
  const day = useDayStatus();
  const today = day.data?.today ?? null;
  const [filters, change] = useCommissionFilters();
  const days = useOpenDays();
  const [active, setActive] = useState<string | null>(null);
  const [view, setView] = useState<'table' | 'preview'>('table');
  const language = useLanguage();
  useEscape('leave', () => {
    if (view === 'preview') setView('table');
    else void navigate({ to: '/' });
  });
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });

  // A side with nobody picked has nothing to read: the page asks for the party instead of guessing.
  const needsParty = filters.side !== null && filters.party === null;
  // The page of a range: it goes back to the first page whenever the period or the party changes.
  const filterKey = JSON.stringify([
    filters.kind,
    filters.from,
    filters.to,
    filters.side,
    filters.party?.id,
  ]);
  const pagerControl = usePagerControl('dashboard.commission', filterKey);
  const paged = filters.kind === 'range';
  const asked = useMemo(() => {
    if (needsParty || (paged && !pagerControl.ready)) return null;
    // Only a range is paged; the other kinds hold few rows and are read whole.
    return commissionInput(
      filters,
      today,
      paged
        ? { offset: (pagerControl.page - 1) * pagerControl.pageSize, limit: pagerControl.pageSize }
        : null,
    );
  }, [
    filters,
    today,
    needsParty,
    paged,
    pagerControl.ready,
    pagerControl.page,
    pagerControl.pageSize,
  ]);
  const read = useCommissionRead(asked);

  const data = read.data;
  const first = data?.first_month ?? null;
  const firstYear = first === null ? null : yearOf(first);
  const refusal = read.isError ? uiErrorOf(read.error) : null;

  // The month on screen is the one the backend answered; the buttons stop at its limits.
  const shownMonth = data?.month ?? null;
  const shownDay = filters.day ?? today;
  const shownYear = filters.year ?? (today === null ? null : yearOf(today));
  const resetRows = () => {
    days.closeAll();
    setActive(null);
  };
  const moveMonth = (next: string) => {
    resetRows();
    // The current month is the address with no month, so a reload on it is always the latest one.
    change({ month: today !== null && next === monthOf(today) ? null : next });
  };
  const moveYear = (next: string) => {
    resetRows();
    change({ year: today !== null && next === yearOf(today) ? null : next });
  };
  const moveDay = (next: string) => {
    resetRows();
    change({ day: next === today ? null : next });
  };

  const stepKeys = today !== null;
  useShortcut(
    SHORTCUTS.pastEarlier,
    () => {
      if (today === null) return;
      if (filters.kind === 'month' && shownMonth !== null) {
        moveMonth(windowMonth(addMonths(shownMonth, -1), today, first));
      } else if (filters.kind === 'year' && shownYear !== null) {
        if (firstYear === null || shownYear > firstYear) moveYear(addYears(shownYear, -1));
      } else if (filters.kind === 'day' && shownDay !== null) {
        moveDay(addDays(shownDay, -1));
      } else if (filters.kind === 'range') {
        pagerControl.setPage(Math.max(1, pagerControl.page - 1));
      }
    },
    { enabled: stepKeys },
  );
  useShortcut(
    SHORTCUTS.pastLater,
    () => {
      if (today === null) return;
      if (filters.kind === 'month' && shownMonth !== null) {
        moveMonth(windowMonth(addMonths(shownMonth, 1), today, first));
      } else if (filters.kind === 'year' && shownYear !== null) {
        if (shownYear < yearOf(today)) moveYear(addYears(shownYear, 1));
      } else if (filters.kind === 'day' && shownDay !== null) {
        if (shownDay < today) moveDay(addDays(shownDay, 1));
      } else if (filters.kind === 'range') {
        pagerControl.setPage(pagerControl.page + 1);
      }
    },
    { enabled: stepKeys },
  );

  // A month the address asked for that has nothing before it opens the first month with data instead.
  useEffect(() => {
    if (
      filters.kind === 'month' &&
      filters.month !== null &&
      first !== null &&
      filters.month < first
    ) {
      change({ month: first });
    }
  }, [filters.kind, filters.month, first, change]);

  const total = data?.total_count ?? 0;
  const pager = describePager(pagerControl, total, read.isFetching);

  let nav = null;
  if (today !== null) {
    switch (filters.kind) {
      case 'month':
        nav =
          shownMonth === null ? null : (
            <div className="flex flex-col gap-3">
              <GoToMonth today={today} first={first} onGo={moveMonth} />
              <MonthNav month={shownMonth} today={today} first={first} onMove={moveMonth} />
            </div>
          );
        break;
      case 'year':
        nav =
          shownYear === null ? null : (
            <div className="flex flex-col gap-3">
              <GoToYear today={today} first={firstYear} onGo={moveYear} />
              <YearNav year={shownYear} today={today} first={firstYear} onMove={moveYear} />
            </div>
          );
        break;
      case 'day':
        nav =
          shownDay === null ? null : (
            <div className="flex flex-col gap-3">
              <GoToCommissionDate today={today} onGo={moveDay} />
              <DayNav day={shownDay} today={today} onMove={moveDay} />
            </div>
          );
        break;
      case 'range':
        nav = data === undefined ? null : <Pager pager={pager} />;
        break;
    }
  }

  // What the print and the Excel hold: every row of the period on screen, of the party on screen.
  // A month is the month the backend answered (so the print is the page on screen even if the month turned).
  const printBase = (printLanguage: 'ur' | 'en', paper: 'a4' | 'a5' | 'mm80') => ({
    month: filters.kind === 'month' ? (shownMonth ?? filters.month) : null,
    ...(asked?.period === undefined ? {} : { period: asked.period }),
    ...(asked?.party === undefined ? {} : { party: asked.party }),
    language: printLanguage,
    paper,
  });
  const exporter = useTableExcel('commission', (sheet) => ({
    kind: 'commission_month',
    ...printBase(sheet, 'a4'),
    paper: null,
  }));

  let body;
  if (needsParty) {
    body = (
      <EmptyState
        icon={UserSearch}
        title={<Localized id={`commission-pick-${filters.side ?? 'buyer'}`} />}
        help={<Localized id="commission-pick-help" />}
      />
    );
  } else if (read.isError && data === undefined) {
    body = (
      <ErrorState
        title={<Localized id="commission-failed" />}
        error={uiErrorOf(read.error)}
        onRetry={() => {
          void read.refetch();
        }}
      />
    );
  } else if (data === undefined) {
    body = <LoadingState rows={6} />;
  } else {
    const empty =
      filters.side !== null ? (
        <EmptyState
          icon={Percent}
          title={<Localized id="commission-empty-party" />}
          help={<Localized id="commission-empty-party-help" />}
        />
      ) : (
        <EmptyState
          icon={Percent}
          title={
            <Localized
              id={`commission-empty${filters.kind === 'month' ? '' : `-${filters.kind}`}`}
            />
          }
          help={
            <Localized
              id={
                filters.kind === 'month' ? 'commission-empty-help' : 'commission-empty-other-help'
              }
            />
          }
        />
      );
    // One date is one row, and its postings are open at once.
    const open = filters.kind === 'day' ? new Set(data.rows.map((row) => row.date)) : days.open;
    body = (
      <div className="flex flex-col gap-3">
        {data.partners_locked && filters.side === null ? <PartnerLockedBox /> : null}
        <CommissionDays
          data={data}
          busy={read.isFetching}
          open={open}
          party={commissionParty(filters)}
          onToggle={(date, isOpen) => {
            if (data.period.kind === 'year') {
              // A month of a year opens as that month (its date is `YYYY-MM`).
              if (isOpen) {
                resetRows();
                change({ kind: 'month', month: date, year: null });
              }
            } else if (filters.kind !== 'day') {
              days.toggle(date, isOpen);
            }
          }}
          active={active}
          onActive={setActive}
          empty={empty}
        />
        {nav}
      </div>
    );
  }
  if (view === 'preview') {
    return (
      <ListPage icon={Percent} titleId="commission-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['commission', shownMonth, asked?.period, asked?.party]}
          read={(printLanguage, paper) =>
            dashboardCommissionPrintData(printBase(printLanguage, paper))
          }
          send={(to, printLanguage, paper) =>
            dashboardCommissionPrint({
              commission: printBase(printLanguage, paper),
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
      icon={Percent}
      titleId="commission-title"
      help={
        <Localized id="commission-help">
          <p />
        </Localized>
      }
      filters={
        <CommissionFilterBar
          filters={filters}
          onChange={(part) => {
            resetRows();
            change(part);
          }}
          today={today}
          error={refusal}
        />
      }
      actions={
        <>
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              void navigate({ to: '/' });
            }}
          >
            <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
            <Localized id="commission-back" />
          </Button>
          <Button
            shortcut={SHORTCUTS.print}
            disabled={data === undefined}
            onClick={() => {
              setView('preview');
            }}
          >
            <Printer aria-hidden="true" className="size-icon-md" />
            <Localized id="report-preview" />
          </Button>
          <ExcelExportButton
            exporter={exporter}
            language={language}
            disabled={data === undefined}
          />
        </>
      }
    >
      <ExcelExportNotices exporter={exporter} />
      {body}
    </ListPage>
  );
}
