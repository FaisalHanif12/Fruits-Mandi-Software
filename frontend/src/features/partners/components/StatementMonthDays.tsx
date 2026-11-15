// The days of one month of a partner's statement (PRD FR-17.12, plan 76 point 11): the opening کل رقم of the
// month on top, then one row for each day that has a movement: his commission share, his expense share, his
// gardens and trips, the fruit he took, the money taken and returned, capital in and out, the day's net, and
// his کل رقم at the end of the day; under them the month's totals and the closing کل رقم. A day opens in place
// (`PartnerDayCard`) with every movement of that day. Every figure is the backend's, read from posted
// records; nothing is added up here (CLAUDE.md rule 1). A figure of 0 is left empty, so the row shows only what
// happened. A negative figure shows its minus sign and the due colour, so colour is never the only signal.

import { Localized } from '@fluent/react';
import { ScrollText } from 'lucide-react';
import { useMemo } from 'react';
import type { PartnerStatementMonth } from '../../../bindings/PartnerStatementMonth';
import type { StatementColumns } from '../../../bindings/StatementColumns';
import type { StatementMonthDay } from '../../../bindings/StatementMonthDay';
import { isoToDisplayDate, isZeroPaisa, type Paisa } from '../../../shared/format';
import { DayTable, type DayColumn } from '../../../shared/grid';
import { EmptyState, Num } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import { PartnerDayCard } from './PartnerDayCard';

/** A figure of the table: empty for 0, else the signed amount. */
function figure(paisa: Paisa) {
  return isZeroPaisa(paisa) ? null : <SignedNum paisa={paisa} />;
}

/** The figure columns of a row, in the order of the table, each with its heading message. */
const FIGURES: readonly {
  key: keyof StatementColumns;
  headerId: string;
  /** The narrowest step the column is drawn at; the day card holds what a narrow table leaves out. */
  from: 'normal' | undefined;
}[] = [
  { key: 'commission_paisa', headerId: 'partners-month-col-commission', from: undefined },
  { key: 'expense_paisa', headerId: 'partners-month-col-expense', from: undefined },
  { key: 'gardens_trips_paisa', headerId: 'partners-month-col-ventures', from: 'normal' },
  { key: 'fruit_paisa', headerId: 'partners-month-col-fruit', from: 'normal' },
  { key: 'taken_returned_paisa', headerId: 'partners-month-col-taken', from: 'normal' },
  { key: 'capital_paisa', headerId: 'partners-month-col-capital', from: 'normal' },
];

interface StatementMonthDaysProps {
  data: PartnerStatementMonth;
  /** A new month is on its way: the rows on screen are the last ones. */
  busy: boolean;
  open: ReadonlySet<string>;
  onToggle: (date: string, open: boolean) => void;
  active: string | null;
  onActive: (date: string) => void;
}

export function StatementMonthDays({
  data,
  busy,
  open,
  onToggle,
  active,
  onActive,
}: StatementMonthDaysProps) {
  const columns = useMemo<readonly DayColumn<StatementMonthDay>[]>(
    () => [
      {
        key: 'date',
        headerId: 'partners-month-col-date',
        cell: (row) => <Num>{isoToDisplayDate(row.date)}</Num>,
        lead: (
          <Localized id="partners-month-opening">
            <span />
          </Localized>
        ),
        footer: (
          <Localized id="glossary-meezan">
            <span />
          </Localized>
        ),
      },
      ...FIGURES.map(({ key, headerId, from }): DayColumn<StatementMonthDay> => ({
        key,
        headerId,
        ...(from === undefined ? {} : { fromStep: from }),
        cell: (row) => figure(row.columns[key]),
        footer: figure(data.totals[key]),
      })),
      {
        key: 'net_paisa',
        headerId: 'partners-month-col-net',
        cell: (row) => <SignedNum paisa={row.columns.net_paisa} />,
        footer: <SignedNum paisa={data.totals.net_paisa} />,
      },
      {
        key: 'running',
        headerId: 'partners-month-col-total',
        cell: (row) => <SignedNum paisa={row.running_paisa} />,
        lead: <SignedNum paisa={data.opening_total_paisa} />,
        footer: <SignedNum paisa={data.closing_total_paisa} />,
      },
    ],
    [data],
  );
  return (
    <DayTable
      columns={columns}
      rows={data.days}
      open={open}
      onToggle={onToggle}
      active={active}
      onActive={onActive}
      withLead
      renderDay={(date) => <PartnerDayCard partnerId={data.partner.id} date={date} />}
      busy={busy}
      labelId="partners-month-label"
      empty={
        <EmptyState
          icon={ScrollText}
          title={<Localized id="partners-month-empty" />}
          help={<Localized id="partners-month-empty-help" />}
        />
      }
    />
  );
}
