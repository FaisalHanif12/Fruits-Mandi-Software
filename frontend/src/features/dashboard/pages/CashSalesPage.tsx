// The cash sales page (route /cash-sales; PRD FR-9.8, FR-12.44, FR-19.11, plan 79 item 2c): the tile آج کی نقدی opens it. One month
// at a time, a row for each day with money paid on the spot (موقع پر جمع): the date, how many buyers paid and
// the نقدی they paid, چکتا buyers included. A click, or Enter, on a day opens its buyers under the row: the
// buyer, his بکری, his bill and what he paid, with a total row. A وصولی collected later is not نقدی; it is
// on روکڑ بک. The month's totals under the columns are the backend's, from the posted records; nothing is
// added up here (CLAUDE.md rule 1).
//
// - The month is in the address (`?month=2026-10`), so a language switch and the way back land on the same
//   month. It opens on the current month and never goes past it. The buttons, the go-to-month box, Page Up
//   and Page Down move it.
// - ↑ ↓ Home End choose a day, Enter opens it. Several days can be open; moving the month shuts them.
// - Four states: loading, an error with a retry, a month with no نقدی (the buttons still work), and the days.

import { Localized } from '@fluent/react';
import { Banknote } from 'lucide-react';
import { useMemo, useState, type ReactNode } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { Language } from '../../../bindings/Language';
import type { Paper } from '../../../bindings/Paper';
import type { CashSalesDayRow } from '../../../bindings/CashSalesDayRow';
import { uiErrorOf } from '../../../shared/api';
import { ExcelExportNotices } from '../../../shared/export';
import { addMonths, isIsoMonth, isoToDisplayDate, windowMonth } from '../../../shared/format';
import { DayTable, type DayColumn } from '../../../shared/grid';
import { useWindowParam } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  EmptyState,
  ErrorState,
  Fact,
  GoToMonth,
  ListPage,
  LoadingState,
  MonthNav,
  MonthTitle,
  Num,
} from '../../../shared/ui';
import { TablePrintPreview, useTableExcel } from '../../reports';
import {
  dashboardCashSalesDayPrint,
  dashboardCashSalesDayPrintData,
  dashboardCashSalesMonthPrint,
  dashboardCashSalesMonthPrintData,
} from '../api';
import { CashSalesDay } from '../components/CashSalesDay';
import { PrintButtons, usePrintView } from '../print-view';
import { useCashSalesMonth } from '../dues-hooks';

export function CashSalesPage() {
  const day = useDayStatus();
  const today = day.data?.today ?? null;
  const [asked, setMonth] = useWindowParam('month', isIsoMonth);
  const read = useCashSalesMonth(asked);
  const data = read.data;
  // The days that are open are kept here, so a language switch loses nothing (F39).
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const [active, setActive] = useState<string | null>(null);

  const moveMonth = (month: string) => {
    setOpen(new Set());
    setActive(null);
    setMonth(month);
  };
  const shown = data?.month ?? null;
  // Print, PDF and Excel: the month on screen, or one day of it opened from its row (PRD FR-19.10).
  const print = usePrintView();
  const [printDay, setPrintDay] = useState<string | null>(null);
  useEscape(
    'leave',
    () => {
      setPrintDay(null);
    },
    printDay !== null,
  );
  const monthInput = (language: Language, paper: Paper | null) => ({
    month: shown,
    language,
    paper,
  });
  const dayInput = (language: Language, paper: Paper | null) => ({
    date: printDay ?? '',
    language,
    paper,
  });
  const monthExcel = useTableExcel('cash-sales-month', (sheet) => ({
    kind: 'cash_sales_month',
    ...monthInput(sheet, null),
  }));
  const dayExcel = useTableExcel('cash-sales-day', (sheet) => ({
    kind: 'cash_sales_day',
    ...dayInput(sheet, null),
  }));
  useShortcut(SHORTCUTS.pastEarlier, () => {
    if (today !== null && shown !== null) moveMonth(windowMonth(addMonths(shown, -1), today, null));
  });
  useShortcut(SHORTCUTS.pastLater, () => {
    if (today !== null && shown !== null) moveMonth(windowMonth(addMonths(shown, 1), today, null));
  });

  const columns = useMemo<readonly DayColumn<CashSalesDayRow>[]>(
    () => [
      {
        key: 'date',
        headerId: 'cash-sales-col-date',
        cell: (row) => <Num>{isoToDisplayDate(row.date)}</Num>,
        footer: (
          <Localized id="glossary-meezan">
            <span />
          </Localized>
        ),
      },
      {
        key: 'buyers',
        headerId: 'cash-sales-col-buyers',
        cell: (row) => <Num>{String(row.buyer_count)}</Num>,
        footer: data === undefined ? null : <Num>{String(data.buyer_days_total)}</Num>,
      },
      {
        key: 'cash',
        headerId: 'cash-sales-col-cash',
        cell: (row) => <Num paisa={row.cash_paisa} />,
        footer: data === undefined ? null : <Num paisa={data.total_paisa} />,
      },
    ],
    [data],
  );

  let body: ReactNode;
  if (read.isError && data === undefined) {
    body = (
      <ErrorState
        title={<Localized id="cash-sales-failed" />}
        error={uiErrorOf(read.error)}
        onRetry={() => {
          void read.refetch();
        }}
      />
    );
  } else if (data === undefined) {
    body = <LoadingState rows={6} />;
  } else {
    body = (
      <div className="flex flex-col gap-3">
        <dl className="grid grid-cols-2 gap-3">
          <Fact stat label={<Localized id="cash-sales-month-total" />}>
            <Num inherit paisa={data.total_paisa} />
          </Fact>
          <Fact stat label={<Localized id="cash-sales-month-buyers" />}>
            <Num inherit>{String(data.buyer_days_total)}</Num>
          </Fact>
        </dl>
        <DayTable
          columns={columns}
          rows={data.rows}
          open={open}
          onToggle={(date, isOpen) => {
            setOpen((current) => {
              const next = new Set(current);
              if (isOpen) next.add(date);
              else next.delete(date);
              return next;
            });
          }}
          active={active}
          onActive={setActive}
          renderDay={(date) => <CashSalesDay date={date} onPrint={setPrintDay} />}
          busy={read.isFetching}
          labelId="cash-sales-label"
          empty={
            <EmptyState
              icon={Banknote}
              title={<Localized id="cash-sales-empty" />}
              help={<Localized id="cash-sales-empty-help" />}
            />
          }
        />
        {today === null ? null : (
          <div className="flex flex-col gap-3">
            <GoToMonth today={today} first={null} onGo={moveMonth} />
            <MonthNav month={data.month} today={today} first={null} onMove={moveMonth} />
          </div>
        )}
      </div>
    );
  }

  if (printDay !== null) {
    return (
      <ListPage icon={Banknote} titleId="cash-sales-title">
        <TablePrintPreview
          exporter={dayExcel}
          scope={['cash_sales_day', printDay]}
          read={(language, paper) => dashboardCashSalesDayPrintData(dayInput(language, paper))}
          send={(to, language, paper) =>
            dashboardCashSalesDayPrint({ sheet: dayInput(language, paper), to, copies: null })
          }
          onBack={() => {
            setPrintDay(null);
          }}
        />
      </ListPage>
    );
  }
  if (print.preview) {
    return (
      <ListPage icon={Banknote} titleId="cash-sales-title">
        <TablePrintPreview
          exporter={monthExcel}
          scope={['cash_sales_month', shown]}
          read={(language, paper) => dashboardCashSalesMonthPrintData(monthInput(language, paper))}
          send={(to, language, paper) =>
            dashboardCashSalesMonthPrint({ sheet: monthInput(language, paper), to, copies: null })
          }
          onBack={print.back}
        />
      </ListPage>
    );
  }

  return (
    <ListPage
      icon={Banknote}
      titleId="cash-sales-title"
      actions={<PrintButtons onPreview={print.show} exporter={monthExcel} />}
      help={
        <Localized id="cash-sales-help">
          <p />
        </Localized>
      }
    >
      <ExcelExportNotices exporter={monthExcel} />
      {shown === null ? null : (
        <p className="type-section text-ink">
          <MonthTitle month={shown} />
        </p>
      )}
      {body}
    </ListPage>
  );
}
