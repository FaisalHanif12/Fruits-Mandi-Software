// The rows of the commission page (PRD FR-9.3, plan 76 point 7, plan 77): one row for each day of a month, a
// date or a range, or for each month of a year, with its figures. For all parties a row has seller commission,
// buyer commission, لاگا, total commission, the shared income and each partner's share of it. For one seller
// it has his sale (خام بکری), the commission charged to him and the commission on his goods; for one buyer his
// purchase value, buyer commission, لاگا and their total. Partner shares belong to whole postings, so they
// never show under a party. A day opens in place under its row and lists the postings that made it, each with
// its بکری (a link to it) and its figures; a month row of a year opens that month. The partner columns exist
// only while the partner section is open; while it is locked the page shows the "enter the code" box and the
// rest works (PRD FR-17.11). Every figure is the backend's, read from the posted records; nothing is added
// up here (CLAUDE.md rule 1). The shared income is not the total commission: it also holds فون and ریٹ فرق,
// and the partners' shares add up to it.

import { Localized } from '@fluent/react';
import { SearchX } from 'lucide-react';
import { useMemo, useState, type ReactNode } from 'react';
import type { CommissionDayRow } from '../../../bindings/CommissionDayRow';
import type { CommissionEntry } from '../../../bindings/CommissionEntry';
import type { CommissionMonth } from '../../../bindings/CommissionMonth';
import type { CommissionParty } from '../../../bindings/CommissionParty';
import type { CommissionPartner } from '../../../bindings/CommissionPartner';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { DayTable, type DayColumn } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  EmptyState,
  ErrorState,
  LinkButton,
  LoadingState,
  MonthTitle,
  Num,
  RUNNING_NUMBER,
} from '../../../shared/ui';
import { nameWithTag } from '../../reports';
import type { AnalyticsGoods } from '../../../bindings/AnalyticsGoods';
import { useCommissionDay } from '../commission-hooks';
import { useOpenDestination } from '../destinations';

/** The side the rows are of: null is every party. */
export type RowsSide = 'buyer' | 'seller' | null;

/** The share of a partner in a row, or null when the row has no partner figures (the section is locked). */
function shareOf(row: CommissionDayRow, partnerId: number) {
  return row.partner_shares?.find((share) => share.partner_id === partnerId)?.share_paisa ?? null;
}

/** One figure of the card of a posting: its words and its amount. */
function Figure({ labelId, children }: { labelId: string; children: ReactNode }) {
  return (
    <div className="flex items-baseline gap-2">
      <Localized id={labelId}>
        <dt className="type-body text-ink-muted" />
      </Localized>
      <dd className="type-body">{children}</dd>
    </div>
  );
}

/** The figures of a row, with the words of each, for the side the rows are of. */
function figuresOf(side: RowsSide, row: CommissionDayRow): readonly [string, ReactNode][] {
  const paisa = (value: CommissionDayRow['seller_comm_paisa'] | null) =>
    value === null ? null : <Num paisa={value} />;
  switch (side) {
    case 'seller':
      return [
        ['commission-col-sale', paisa(row.sale_paisa)],
        ['commission-col-charged', paisa(row.seller_comm_paisa)],
        ['commission-col-goods', paisa(row.goods_comm_paisa)],
      ];
    case 'buyer':
      return [
        ['commission-col-purchase', paisa(row.sale_paisa)],
        ['commission-col-buyer', paisa(row.buyer_comm_paisa)],
        ['commission-col-laga', paisa(row.laga_paisa)],
        ['commission-col-total', paisa(row.total_comm_paisa)],
      ];
    case null:
      return [
        ['commission-col-seller', paisa(row.seller_comm_paisa)],
        ['commission-col-buyer', paisa(row.buyer_comm_paisa)],
        ['commission-col-laga', paisa(row.laga_paisa)],
        ['commission-col-total', paisa(row.total_comm_paisa)],
        ['commission-col-shared', paisa(row.shared_income_paisa)],
      ];
  }
}

/** The words of a posting type. A type this page does not know is told by a general word, never by its id. */
function postingId(type: string): string {
  switch (type) {
    case 'bikri_first':
    case 'bikri_later':
    case 'reversal':
    case 'general':
      return `commission-posting-${type.replaceAll('_', '-')}`;
    default:
      return 'commission-posting-other';
  }
}

function EntryCard({
  entry,
  partners,
  side,
}: {
  entry: CommissionEntry;
  partners: readonly CommissionPartner[];
  side: RowsSide;
}) {
  const language = useLanguage();
  const open = useOpenDestination();
  const people = [...entry.sellers, ...entry.buyers]
    .map((party) => nameWithTag(party, party.tag, language))
    .join('، ');
  return (
    <li className="flex flex-col gap-1 rounded-3 border border-border bg-surface px-3 py-2">
      <div className="flex flex-wrap items-baseline gap-x-3">
        {entry.bikri_id === null || entry.bikri_number === null ? null : (
          <LinkButton
            onClick={() => {
              if (entry.bikri_id !== null) open({ kind: 'bikri-book', id: entry.bikri_id });
            }}
          >
            <Localized id="commission-bikri" vars={{ number: String(entry.bikri_number) }}>
              <span />
            </Localized>
          </LinkButton>
        )}
        <Localized id={postingId(entry.posting_type)}>
          <span className="type-chip text-ink-muted" />
        </Localized>
        {people === '' ? null : <span className="type-body">{people}</span>}
      </div>
      <dl className="flex flex-wrap gap-x-5 gap-y-1">
        {figuresOf(side, entry.figures).map(([labelId, cell]) =>
          cell === null ? null : (
            <Figure key={labelId} labelId={labelId}>
              {cell}
            </Figure>
          ),
        )}
      </dl>
      {side !== null || entry.figures.partner_shares === null ? null : (
        <ul className="flex flex-wrap gap-x-5 type-body text-ink-muted">
          {partners.map((partner) => {
            const share = shareOf(entry.figures, partner.id);
            return share === null ? null : (
              <li key={partner.id} className="flex items-baseline gap-2">
                <span lang={language}>{displayName(partner, language)}</span>
                <Num paisa={share} />
              </li>
            );
          })}
        </ul>
      )}
    </li>
  );
}

/** What one open day shows: the postings that made its commission (of one party, when the page has one). */
function DayDetail({
  date,
  party,
  goods,
  side,
}: {
  date: string;
  party: CommissionParty | null;
  goods: AnalyticsGoods;
  side: RowsSide;
}) {
  const day = useCommissionDay(date, party, goods, true);
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="commission-day-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  if (day.data === undefined) return <LoadingState rows={2} />;
  if (day.data.entries.length === 0) {
    return (
      <EmptyState
        icon={SearchX}
        title={<Localized id="commission-day-empty" />}
        help={<Localized id="commission-day-empty-help" />}
      />
    );
  }
  return (
    <div data-commission-day={date} className="flex flex-col gap-2">
      <ul className="flex flex-col gap-2">
        {day.data.entries.map((entry) => (
          <EntryCard key={entry.entry_id} entry={entry} partners={day.data.partners} side={side} />
        ))}
      </ul>
      {day.data.total_count > day.data.entries.length ? (
        <Localized
          id="commission-day-more"
          vars={{ shown: String(day.data.entries.length), total: String(day.data.total_count) }}
          elems={RUNNING_NUMBER}
        >
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </div>
  );
}

interface CommissionDaysProps {
  data: CommissionMonth;
  /** A new period is on its way: the rows on screen are the last ones. */
  busy: boolean;
  /** The days that are open (`YYYY-MM-DD`), kept by the page so a language switch loses nothing. */
  open: ReadonlySet<string>;
  onToggle: (date: string, open: boolean) => void;
  active: string | null;
  onActive: (date: string) => void;
  /** What the table says when it has no rows. */
  empty: ReactNode;
  /** The party the rows are of, as the backend reads it (for the postings of an open day); null is every party. */
  party: CommissionParty | null;
  /** Which goods the rows are of (plan 78 D4); the postings of an open day are of the same goods. */
  goods: AnalyticsGoods;
}

export function CommissionDays({
  data,
  busy,
  open,
  onToggle,
  active,
  onActive,
  empty,
  party,
  goods,
}: CommissionDaysProps) {
  const language = useLanguage();
  const side: RowsSide = data.party?.side ?? null;
  // Under a goods filter the shares and the shared income are not shown: they belong to whole postings (plan 78 D4).
  const showPartners = side === null && goods === 'all' && !data.partners_locked;
  // A year is one row for each month, its date `YYYY-MM` (PRD FR-9.3).
  const byMonth = data.period.kind === 'year';
  const columns = useMemo<readonly DayColumn<CommissionDayRow>[]>(() => {
    const first: DayColumn<CommissionDayRow> = {
      key: 'date',
      headerId: byMonth ? 'commission-col-month' : 'commission-col-date',
      cell: (row) =>
        byMonth ? <MonthTitle month={row.date} /> : <Num>{isoToDisplayDate(row.date)}</Num>,
      footer: (
        <Localized id="glossary-meezan">
          <span />
        </Localized>
      ),
    };
    const figure = (
      key: string,
      headerId: string,
      pick: (row: CommissionDayRow) => CommissionDayRow['laga_paisa'] | null,
      fromNormal = false,
    ): DayColumn<CommissionDayRow> => ({
      key,
      headerId,
      ...(fromNormal ? { fromStep: 'normal' as const } : {}),
      cell: (row) => {
        const value = pick(row);
        return value === null ? null : <Num paisa={value} />;
      },
      footer: (() => {
        const value = pick(data.totals);
        return value === null ? null : <Num paisa={value} />;
      })(),
    });
    if (side === 'seller') {
      return [
        first,
        figure('sale', 'commission-col-sale', (row) => row.sale_paisa),
        figure('charged', 'commission-col-charged', (row) => row.seller_comm_paisa),
        figure('goods', 'commission-col-goods', (row) => row.goods_comm_paisa),
      ];
    }
    if (side === 'buyer') {
      return [
        first,
        figure('purchase', 'commission-col-purchase', (row) => row.sale_paisa),
        figure('buyer', 'commission-col-buyer', (row) => row.buyer_comm_paisa, true),
        figure('laga', 'commission-col-laga', (row) => row.laga_paisa, true),
        figure('total', 'commission-col-total', (row) => row.total_comm_paisa),
      ];
    }
    return [
      first,
      figure('seller', 'commission-col-seller', (row) => row.seller_comm_paisa, true),
      figure('buyer', 'commission-col-buyer', (row) => row.buyer_comm_paisa, true),
      figure('laga', 'commission-col-laga', (row) => row.laga_paisa, true),
      figure('total', 'commission-col-total', (row) => row.total_comm_paisa),
      figure('shared', 'commission-col-shared', (row) => row.shared_income_paisa),
      ...(showPartners
        ? data.partners.map((partner): DayColumn<CommissionDayRow> => ({
            key: `partner-${String(partner.id)}`,
            header: <span lang={language}>{displayName(partner, language)}</span>,
            cell: (row) => {
              const share = shareOf(row, partner.id);
              return share === null ? null : <Num paisa={share} />;
            },
            footer: (() => {
              const share = shareOf(data.totals, partner.id);
              return share === null ? null : <Num paisa={share} />;
            })(),
          }))
        : []),
    ];
  }, [data, side, showPartners, byMonth, language]);
  return (
    <DayTable
      columns={columns}
      rows={data.rows}
      open={open}
      onToggle={onToggle}
      active={active}
      onActive={onActive}
      renderDay={(date) => <DayDetail date={date} party={party} goods={goods} side={side} />}
      busy={busy}
      labelId="commission-label"
      empty={empty}
    />
  );
}

/** Which days are open: kept above the table so a language switch loses nothing (F39). */
export function useOpenDays() {
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const toggle = (date: string, isOpen: boolean) => {
    setOpen((current) => {
      const next = new Set(current);
      if (isOpen) next.add(date);
      else next.delete(date);
      return next;
    });
  };
  const closeAll = () => {
    setOpen(new Set());
  };
  return { open, toggle, closeAll };
}
