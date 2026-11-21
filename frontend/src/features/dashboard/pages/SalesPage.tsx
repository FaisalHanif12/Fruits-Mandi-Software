// The sales page (route /sales; PRD FR-9.9, FR-19.11, plan 83): the tile آج کی بکری opens it, built like the commission
// page. The Owner picks a period (one date, a month, a year or a range of dates) and, if he wants, one buyer or one
// seller, one item and one kind of goods, and reads the sales of that period: for each day (a month, a date or a
// range) or each month (a year) the number of بکری, the نگ sold, the خام بکری, the buyers' bill, what was paid on the
// spot and what was left on account, with the totals. Today's row is marked. A click or Enter on a day opens the بکری
// posted that day, in place, each with a button that opens it; a click on a month of a year opens that month. The
// totals row, the rows and the periods it can go to are the backend's, read from the posted records, so today's row
// equals the tile and the month equals R2; the page adds nothing up (CLAUDE.md rule 1).
//
// - The period and the filters are kept in the address (`?period=year&year=2026&side=seller&party=...`), so a
//   language switch, a reload and the way back from a بکری land on the same page. The month has its buttons, its
//   box and Page Up and Page Down; a year and a date have the same pair; a range is paged.
// - Every party box starts at All (PRD FR-19.1). A side with nobody picked has nothing to read: the page asks for
//   him instead of guessing.
// - Four states: loading, an error with a retry, no sales in the period (the buttons still work), and the rows.

import { Localized } from '@fluent/react';
import { Printer, TrendingUp, UserSearch } from 'lucide-react';
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
} from '../../../shared/ui';
import { TablePrintPreview, useTableExcel } from '../../reports';
import { dashboardSalesPrint, dashboardSalesPrintData } from '../api';
import { addYears, yearOf } from '../commission-search';
import { commissionPeriod } from '../commission-hooks';
import { useOpenDays } from '../components/CommissionDays';
import { DayNav, GoToCommissionDate, GoToYear, YearNav } from '../components/CommissionNav';
import { SalesDays } from '../components/SalesDays';
import { SalesFilterBar } from '../components/SalesFilterBar';
import type { SalesPrintInput } from '../../../bindings/SalesPrintInput';
import { salesFilterInput, salesInput, salesNeedsParty, useSalesRead } from '../sales-hooks';
import { useSalesFilters } from '../sales-search';

export function SalesPage() {
  const day = useDayStatus();
  const today = day.data?.today ?? null;
  const [filters, change] = useSalesFilters();
  const days = useOpenDays();
  const [active, setActive] = useState<string | null>(null);
  const [view, setView] = useState<'table' | 'preview'>('table');
  const language = useLanguage();
  // Esc closes the preview first; with the table showing, Esc is the shell's: back to the page the user came
  // from (plan 78 F1).
  useEscape(
    'leave',
    () => {
      setView('table');
    },
    view === 'preview',
  );
  useShortcut(SHORTCUTS.print, () => {
    if (view === 'table') setView('preview');
  });

  // A side with nobody picked has nothing to read: the page asks for the party instead of guessing.
  const needsParty = salesNeedsParty(filters);
  // The page of a range: it goes back to the first page whenever the period or a filter changes.
  const filterKey = JSON.stringify([
    filters.kind,
    filters.from,
    filters.to,
    filters.side,
    filters.party?.id,
    filters.item?.id,
    filters.goods,
  ]);
  const pagerControl = usePagerControl('dashboard.sales', filterKey);
  const paged = filters.kind === 'range';
  const asked = useMemo(() => {
    if (needsParty || (paged && !pagerControl.ready)) return null;
    return salesInput(
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
  const read = useSalesRead(asked);

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

  // What the print and the Excel hold: every row of the period on screen, in the filters on screen. A month is the
  // month the backend answered (so the print is the page on screen even if the month turned).
  const printBase = (
    printLanguage: SalesPrintInput['language'],
    paper: SalesPrintInput['paper'],
  ): SalesPrintInput | null => {
    const period =
      filters.kind === 'month' && shownMonth !== null
        ? ({ kind: 'month', month: shownMonth } as const)
        : commissionPeriod(filters, today);
    if (period === null) return null;
    return { period, ...salesFilterInput(filters), language: printLanguage, paper };
  };
  const exporter = useTableExcel('sales', (sheet) => {
    const input = printBase(sheet, null);
    if (input === null) throw new Error('sales export without a period');
    return { kind: 'sales_period', ...input };
  });

  const hasFilter = filters.side !== null || filters.item !== null || filters.goods !== 'all';
  let body;
  if (needsParty) {
    body = (
      <EmptyState
        icon={UserSearch}
        title={<Localized id={`sales-pick-${filters.side ?? 'buyer'}`} />}
        help={<Localized id="sales-pick-help" />}
      />
    );
  } else if (read.isError && data === undefined) {
    body = (
      <ErrorState
        title={<Localized id="sales-failed" />}
        error={uiErrorOf(read.error)}
        onRetry={() => {
          void read.refetch();
        }}
      />
    );
  } else if (data === undefined) {
    body = <LoadingState rows={6} />;
  } else {
    const empty = hasFilter ? (
      <EmptyState
        icon={TrendingUp}
        title={<Localized id="sales-empty-filtered" />}
        help={<Localized id="sales-empty-filtered-help" />}
      />
    ) : (
      <EmptyState
        icon={TrendingUp}
        title={
          <Localized id={`sales-empty${filters.kind === 'month' ? '' : `-${filters.kind}`}`} />
        }
        help={
          <Localized
            id={filters.kind === 'month' ? 'sales-empty-help' : 'sales-empty-other-help'}
          />
        }
      />
    );
    // One date is one row, and its بکری are open at once.
    const open = filters.kind === 'day' ? new Set(data.rows.map((row) => row.date)) : days.open;
    body = (
      <div className="flex flex-col gap-3">
        <SalesDays
          data={data}
          today={today}
          busy={read.isFetching}
          open={open}
          filter={salesFilterInput(filters)}
          onToggle={(date, isOpen) => {
            if (data.period.kind === 'year') {
              // A month of a year opens as that month (its key is `YYYY-MM`).
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
      <ListPage icon={TrendingUp} titleId="sales-title">
        <TablePrintPreview
          exporter={exporter}
          scope={['sales', shownMonth, asked?.period, asked?.party, asked?.item_id, asked?.goods]}
          read={(printLanguage, paper) => {
            const input = printBase(printLanguage, paper);
            if (input === null) return Promise.reject(new Error('sales print without a period'));
            return dashboardSalesPrintData(input);
          }}
          send={(to, printLanguage, paper) => {
            const input = printBase(printLanguage, paper);
            if (input === null) return Promise.reject(new Error('sales print without a period'));
            return dashboardSalesPrint({ sheet: input, to, copies: null });
          }}
          onBack={() => {
            setView('table');
          }}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={TrendingUp}
      titleId="sales-title"
      help={
        <Localized id="sales-help">
          <p />
        </Localized>
      }
      filters={
        <SalesFilterBar
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
