// The kharch page (route /kharch; PRD FR-9.5, plan 77): the tile آج کا خرچہ opens it. Two parts for one period (one
// date, a month, a year or a range of dates): the shop's kharch (one row for each دکان خرچہ head with the whole
// shop's total) and what each partner spent himself (ذاتی خرچہ, with a total), with a partner filter. Both come from
// the ONE read of the analytics view M, drawn by the same board (`KharchBoard`), so the page and the view can never
// differ. A figure opens its entries (date, amount, reason); the partner part waits for the partner code (the shop
// part shows without it); each part prints and goes to Excel. The totals and every figure are the backend's; the
// page adds nothing up (CLAUDE.md rule 1).
//
// - The period and the partner are kept in the address (`?period=year&year=2026&partner=2`), so a language switch, a
//   reload and the way back land on the same page. The month (the default) has previous and next buttons, a box to
//   go to a month, Page Up and Page Down; a year and a date have the same pair; a range is typed.
// - Four states: loading while today is not known, then the parts' own (loading, an error with a retry, nothing
//   found, the rows).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowLeft, WalletCards } from 'lucide-react';
import { useMemo } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { addDays, addMonths, monthOf, windowMonth } from '../../../shared/format';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import { Button, GoToMonth, ListPage, LoadingState, MonthNav } from '../../../shared/ui';
import { KharchBoard } from '../../analytics';
import { addYears, yearOf } from '../commission-search';
import { DayNav, GoToCommissionDate, GoToYear, YearNav } from '../components/CommissionNav';
import { KharchFilterBar } from '../components/KharchFilterBar';
import { useKharchFilters } from '../kharch-search';
import { kharchWire } from '../kharch-wire';

export function KharchPage() {
  const navigate = useNavigate();
  const day = useDayStatus();
  const today = day.data?.today ?? null;
  const [filters, change] = useKharchFilters();
  const wire = useMemo(() => kharchWire(filters, today), [filters, today]);

  // The first day of the books: the buttons stop at its month and its year (the backend's `first_entry_date`).
  const first = day.data?.first_entry_date ?? null;
  const firstMonth = first === null ? null : monthOf(first);
  const firstYear = first === null ? null : yearOf(first);
  const shownMonth = filters.month ?? (today === null ? null : monthOf(today));
  const shownYear = filters.year ?? (today === null ? null : yearOf(today));
  const shownDay = filters.day ?? today;

  // The current month, year and day are the address with none, so a reload on them is always the latest.
  const moveMonth = (next: string) => {
    change({ month: today !== null && next === monthOf(today) ? null : next });
  };
  const moveYear = (next: string) => {
    change({ year: today !== null && next === yearOf(today) ? null : next });
  };
  const moveDay = (next: string) => {
    change({ day: next === today ? null : next });
  };

  useShortcut(
    SHORTCUTS.pastEarlier,
    () => {
      if (today === null) return;
      if (filters.kind === 'month' && shownMonth !== null) {
        moveMonth(windowMonth(addMonths(shownMonth, -1), today, firstMonth));
      } else if (filters.kind === 'year' && shownYear !== null) {
        if (firstYear === null || shownYear > firstYear) moveYear(addYears(shownYear, -1));
      } else if (filters.kind === 'day' && shownDay !== null) {
        moveDay(addDays(shownDay, -1));
      }
    },
    { enabled: today !== null },
  );
  useShortcut(
    SHORTCUTS.pastLater,
    () => {
      if (today === null) return;
      if (filters.kind === 'month' && shownMonth !== null) {
        moveMonth(windowMonth(addMonths(shownMonth, 1), today, firstMonth));
      } else if (filters.kind === 'year' && shownYear !== null) {
        if (shownYear < yearOf(today)) moveYear(addYears(shownYear, 1));
      } else if (filters.kind === 'day' && shownDay !== null) {
        if (shownDay < today) moveDay(addDays(shownDay, 1));
      }
    },
    { enabled: today !== null },
  );

  let nav = null;
  if (today !== null) {
    if (filters.kind === 'month' && shownMonth !== null) {
      nav = (
        <div className="flex flex-col gap-3">
          <GoToMonth today={today} first={firstMonth} onGo={moveMonth} />
          <MonthNav month={shownMonth} today={today} first={firstMonth} onMove={moveMonth} />
        </div>
      );
    } else if (filters.kind === 'year' && shownYear !== null) {
      nav = (
        <div className="flex flex-col gap-3">
          <GoToYear today={today} first={firstYear} onGo={moveYear} />
          <YearNav year={shownYear} today={today} first={firstYear} onMove={moveYear} />
        </div>
      );
    } else if (filters.kind === 'day' && shownDay !== null) {
      nav = (
        <div className="flex flex-col gap-3">
          <GoToCommissionDate today={today} onGo={moveDay} />
          <DayNav day={shownDay} today={today} onMove={moveDay} />
        </div>
      );
    }
  }

  return (
    <ListPage
      icon={WalletCards}
      titleId="kharch-title"
      help={
        <Localized id="kharch-help">
          <p />
        </Localized>
      }
      actions={
        <Button
          shortcut={SHORTCUTS.back}
          onClick={() => {
            void navigate({ to: '/' });
          }}
        >
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="kharch-back" />
        </Button>
      }
      filters={<KharchFilterBar filters={filters} onChange={change} today={today} />}
    >
      {wire === null ? (
        <LoadingState rows={6} />
      ) : (
        <div className="flex flex-col gap-3">
          <KharchBoard
            filters={wire}
            onLeave={() => {
              void navigate({ to: '/' });
            }}
          />
          {nav}
        </div>
      )}
    </ListPage>
  );
}
