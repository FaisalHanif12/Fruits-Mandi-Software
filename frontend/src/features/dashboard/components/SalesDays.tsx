// The rows of the sales page (PRD FR-9.9, plan 83): one row for each day of a month, a date or a range, or for each
// month of a year, with its figures: the number of بکری, the نگ sold, the خام بکری, the buyers' bill, what was paid on
// the spot (نقدی) and what was left on account (ادھاری). Today's row is marked with the word. A day opens in place
// under its row and lists the بکری posted that day, each with its sellers and lots, its figures and a button that
// opens it; a month row of a year opens that month. The totals row and every figure are the backend's, read from the
// posted records, so the day equals the tile and the month equals R2; nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { CalendarCheck, SearchX } from 'lucide-react';
import { useMemo, type ReactNode } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { DayTable, type DayColumn } from '../../../shared/grid';
import { useLanguage, useLotNames } from '../../../shared/i18n';
import {
  EmptyState,
  ErrorState,
  LinkButton,
  LoadingState,
  MonthTitle,
  Num,
  RUNNING_NUMBER,
  StatusChip,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { useOpenDestination } from '../destinations';
import type { SalesDayRow } from '../../../bindings/SalesDayRow';
import type { SalesPeriod } from '../../../bindings/SalesPeriod';
import type { SalesPeriodRow } from '../../../bindings/SalesPeriodRow';
import { useSalesDay, type SalesFilterInput } from '../sales-hooks';

interface SalesDaysProps {
  data: SalesPeriod;
  /** The backend's today, `YYYY-MM-DD`: its row is marked. */
  today: string | null;
  /** A new period is on its way: the rows on screen are the last ones. */
  busy: boolean;
  /** The days that are open (`YYYY-MM-DD`), kept by the page so a language switch loses nothing. */
  open: ReadonlySet<string>;
  onToggle: (date: string, open: boolean) => void;
  active: string | null;
  onActive: (date: string) => void;
  /** What the table says when it has no rows. */
  empty: ReactNode;
  /** The party, item and goods the rows are of, for the بکری of an open day. */
  filter: SalesFilterInput;
}

export function SalesDays({
  data,
  today,
  busy,
  open,
  onToggle,
  active,
  onActive,
  empty,
  filter,
}: SalesDaysProps) {
  // A year is one row for each month, its key `YYYY-MM` (PRD FR-9.9).
  const byMonth = data.period.kind === 'year';
  const columns = useMemo<readonly DayColumn<SalesPeriodRow>[]>(() => {
    const figure = (
      key: string,
      headerId: string,
      pick: (row: SalesPeriodRow) => ReactNode,
      fromNormal = false,
    ): DayColumn<SalesPeriodRow> => ({
      key,
      headerId,
      ...(fromNormal ? { fromStep: 'normal' as const } : {}),
      cell: pick,
      footer: pick(data.totals),
    });
    return [
      {
        key: 'date',
        headerId: byMonth ? 'sales-col-month' : 'sales-col-date',
        cell: (row) =>
          byMonth ? (
            <MonthTitle month={row.date} />
          ) : (
            <span className="inline-flex flex-wrap items-center justify-center gap-x-2">
              <Num>{isoToDisplayDate(row.date)}</Num>
              {row.date === today ? (
                <StatusChip tone="primary" icon={CalendarCheck}>
                  <Localized id="sales-today" />
                </StatusChip>
              ) : null}
            </span>
          ),
        footer: (
          <Localized id="glossary-meezan">
            <span />
          </Localized>
        ),
      },
      figure('count', 'sales-col-count', (row) => <Num>{String(row.bikri_count)}</Num>),
      figure('qty', 'sales-col-qty', (row) => <Num>{String(row.qty)}</Num>, true),
      figure('gross', 'sales-col-gross', (row) => <Num paisa={row.gross_paisa} />),
      figure(
        'buyer-total',
        'sales-col-buyer-total',
        (row) => <Num paisa={row.buyer_total_paisa} />,
        true,
      ),
      figure('cash', 'sales-col-cash', (row) => <Num paisa={row.cash_paisa} />),
      figure('credit', 'sales-col-credit', (row) => <Num paisa={row.credit_paisa} />),
    ];
  }, [data.totals, byMonth, today]);
  return (
    <DayTable
      columns={columns}
      rows={data.rows}
      open={open}
      onToggle={onToggle}
      active={active}
      onActive={onActive}
      renderDay={(date) => <SalesDayDetail date={date} filter={filter} />}
      busy={busy}
      labelId="sales-label"
      empty={empty}
    />
  );
}

/** What one open day shows: the بکری posted that day, in the filters of the page, with the day's totals. */
function SalesDayDetail({ date, filter }: { date: string; filter: SalesFilterInput }) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const { lotName } = useLotNames();
  const open = useOpenDestination();
  const day = useSalesDay(date, filter, true);

  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="sales-day-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  if (day.data === undefined) return <LoadingState rows={2} />;
  const data = day.data;
  if (data.rows.length === 0) {
    return (
      <EmptyState
        icon={SearchX}
        title={<Localized id="sales-day-empty" />}
        help={<Localized id="sales-day-empty-help" />}
      />
    );
  }
  const columns: readonly TableColumn<SalesDayRow>[] = [
    {
      id: 'bikri',
      header: <Localized id="sales-day-col-bikri" />,
      kind: 'centre',
      size: 'qty',
      cell: (row) => (
        <LinkButton
          onClick={() => {
            open({ kind: 'bikri-book', id: row.bikri_id });
          }}
        >
          <Localized
            id="sales-day-open"
            vars={{ number: String(row.bikri_number) }}
            elems={RUNNING_NUMBER}
          >
            <span />
          </Localized>
        </LinkButton>
      ),
    },
    {
      id: 'sellers',
      header: <Localized id="sales-day-col-sellers" />,
      // The day's total row says how many بکری the day holds (the backend's figure, the tile's and the row's); it sits in
      // the wide column so the words do not wrap.
      total: (
        <Localized
          id="sales-day-total"
          vars={{ count: data.totals.bikri_count }}
          elems={RUNNING_NUMBER}
        >
          <span />
        </Localized>
      ),
      cell: (row) => (
        <span className="flex flex-col" lang={language}>
          {row.lots.map((lot) => (
            <span key={lot.lot_id}>{lotName(lot.seller, lot.item, lot.counter)}</span>
          ))}
        </span>
      ),
    },
    {
      id: 'qty',
      header: <Localized id="sales-day-col-qty" />,
      kind: 'number',
      size: 'qty',
      cell: (row) => <Num>{String(row.qty)}</Num>,
      total: <Num>{String(data.totals.qty)}</Num>,
    },
    {
      id: 'gross',
      header: <Localized id="sales-day-col-gross" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.gross_paisa} />,
      total: <Num paisa={data.totals.gross_paisa} />,
    },
    {
      id: 'buyer-total',
      header: <Localized id="sales-day-col-buyer-total" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.buyer_total_paisa} />,
      total: <Num paisa={data.totals.buyer_total_paisa} />,
    },
    {
      id: 'cash',
      header: <Localized id="sales-day-col-cash" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.cash_paisa} />,
      total: <Num paisa={data.totals.cash_paisa} />,
    },
    {
      id: 'credit',
      header: <Localized id="sales-day-col-credit" />,
      kind: 'number',
      size: 'money',
      cell: (row) => <Num paisa={row.credit_paisa} />,
      total: <Num paisa={data.totals.credit_paisa} />,
    },
  ];
  return (
    <div data-sales-day={date} className="flex flex-col gap-2">
      <Table
        label={l10n.getString('sales-day-label')}
        columns={columns}
        rows={data.rows}
        rowKey={(row) => String(row.bikri_id)}
      />
      {data.total_count > data.rows.length ? (
        <Localized
          id="sales-day-more"
          vars={{ shown: String(data.rows.length), total: String(data.total_count) }}
          elems={RUNNING_NUMBER}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </div>
  );
}
