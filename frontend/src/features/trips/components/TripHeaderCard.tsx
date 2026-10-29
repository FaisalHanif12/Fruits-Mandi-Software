// The facts of one trip (PRD 12.10, FR-12.25, FR-12.27, FR-12.29), in two cards so the entry form
// comes right after the short one: the header (number, state, day, market, who was sent, and the
// result in one line with the working behind its ⓘ) and, under the entry form, the figures (each
// partner with his % and where his money comes from by default, the goods bought, sold, written
// off and left, and how the result is made up). A loss is red and also written (NFR-USE-12).
// Everything is the backend's; nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { CircleCheck, CircleDot, TriangleAlert } from 'lucide-react';
import { formatBps, isNegativePaisa, isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, Code, Fact, Num } from '../../../shared/ui';
import type { TripRow } from '../types';

interface TripCardProps {
  trip: TripRow;
}

/** The result of the trip in one line: Trading Profit or Trading Loss, its number and its ⓘ. */
function ResultLine({ trip }: TripCardProps) {
  const loss = isNegativePaisa(trip.result_paisa);
  return (
    <div
      className={`flex items-center justify-between gap-3 border-b border-border py-1 type-label ${loss ? 'text-due' : 'text-ink'}`}
    >
      <Localized id={loss ? 'glossary-tijarati-nuqsan' : 'glossary-tijarati-munafa'}>
        <span />
      </Localized>
      <span className="flex items-center gap-1">
        <Num paisa={trip.result_paisa} />
        <CalcInfo working={trip.result_working} inCell />
      </span>
    </div>
  );
}

function LossNote({ trip }: TripCardProps) {
  if (!isNegativePaisa(trip.result_paisa)) return null;
  return (
    <p className="flex items-center gap-2 type-body text-due">
      <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
      <Localized id="trips-loss-note">
        <span />
      </Localized>
    </p>
  );
}

export function TripHeaderCard({ trip }: TripCardProps) {
  const open = trip.status === 'open';
  const StatusIcon = open ? CircleDot : CircleCheck;
  return (
    <Card
      compact
      title={
        <span className="inline-flex flex-wrap items-center gap-2">
          <Localized id="trips-detail-title" vars={{ code: trip.code }}>
            <span />
          </Localized>
          <span
            className={`inline-flex items-center gap-1 type-chip rounded-1 px-2 ${open ? 'bg-primary-soft text-primary' : 'bg-surface-sunken text-ink-muted'}`}
          >
            <StatusIcon aria-hidden="true" className="size-icon-sm shrink-0" />
            <Localized id={open ? 'trips-status-open' : 'trips-status-closed'} />
          </span>
        </span>
      }
    >
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3 normal:grid-cols-4">
        <Fact label={<Localized id="trips-field-date" />}>
          <Code>{isoToDisplayDate(trip.entry_date)}</Code>
        </Fact>
        <Fact label={<Localized id="trips-field-market" />}>
          {trip.market ?? <Localized id="trips-none" />}
        </Fact>
        <Fact label={<Localized id="trips-field-sent" />}>
          {trip.sent_person ?? <Localized id="trips-none" />}
        </Fact>
        {trip.closed_date === null ? null : (
          <Fact label={<Localized id="trips-closed-on" />}>
            <Code>{isoToDisplayDate(trip.closed_date)}</Code>
          </Fact>
        )}
        {trip.note === null ? null : (
          <Fact label={<Localized id="trips-field-note" />}>{trip.note}</Fact>
        )}
      </dl>
      <ResultLine trip={trip} />
      <LossNote trip={trip} />
    </Card>
  );
}

export function TripFiguresCard({ trip }: TripCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const rows: readonly { id: string; message: string; paisa: TripRow['sales_paisa'] }[] = [
    { id: 'sales', message: 'purchases-detail-sales', paisa: trip.sales_paisa },
    { id: 'other', message: 'purchases-detail-other-shop', paisa: trip.other_shop_paisa },
    { id: 'sold', message: 'purchases-detail-cost-sold', paisa: trip.cost_sold_paisa },
    { id: 'off', message: 'purchases-detail-cost-written-off', paisa: trip.cost_written_off_paisa },
  ];
  return (
    <Card compact title={<Localized id="trips-figures-title" />}>
      <div className="grid grid-cols-1 gap-x-6 gap-y-4 normal:grid-cols-2">
        <div className="flex min-w-0 flex-col gap-1">
          <Localized id="trips-partners-title">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <ul aria-label={l10n.getString('trips-partners-title')} className="flex flex-col">
            {trip.partners.map((partner) => (
              <li
                key={partner.partner.id}
                className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 border-b border-border py-1 type-body text-ink"
              >
                <span className="flex items-baseline gap-2">
                  <span lang={language}>{displayName(partner.partner, language)}</span>
                  <span className="text-ink-muted">
                    <Num>
                      {l10n.getString('ventures-share-percent', {
                        value: formatBps(partner.share_bps),
                      })}
                    </Num>
                  </span>
                </span>
                <Localized
                  id={partner.source === 'shop' ? 'ventures-source-shop' : 'ventures-source-pocket'}
                >
                  <span className="text-ink-muted" />
                </Localized>
              </li>
            ))}
          </ul>
          {trip.shares_locked ? (
            <Localized id="trips-shares-locked">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null}
        </div>
        <div className="flex min-w-0 flex-col gap-3">
          <dl className="grid grid-cols-3 gap-x-4 gap-y-3">
            <Fact label={<Localized id="trips-fact-bought" />}>
              <Num>{String(trip.qty_bought)}</Num>
            </Fact>
            <Fact label={<Localized id="purchases-lot-qty-sold" />}>
              <Num>{String(trip.qty_sold)}</Num>
            </Fact>
            <Fact label={<Localized id="purchases-detail-written-off" />}>
              <Num>{String(trip.qty_written_off)}</Num>
            </Fact>
            <Fact label={<Localized id="purchases-col-left" />}>
              <Num>{String(trip.qty_left)}</Num>
            </Fact>
            <Fact label={<Localized id="trips-fact-cost" />}>
              <Num paisa={trip.cost_paisa} />
            </Fact>
            <Fact label={<Localized id="purchases-col-cost-left" />}>
              <Num paisa={trip.cost_left_paisa} />
            </Fact>
          </dl>
          <dl className="flex flex-col">
            {rows.map((row) => (
              <div
                key={row.id}
                className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink"
              >
                <dt>{l10n.getString(row.message)}</dt>
                <dd>
                  <Num paisa={row.paisa} />
                </dd>
              </div>
            ))}
          </dl>
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-label text-ink">
              <Localized id="glossary-commission">
                <dt />
              </Localized>
              <dd className="flex items-center gap-1">
                <Num paisa={trip.commission_paisa} />
                <CalcInfo working={trip.commission_working} inCell />
              </dd>
            </div>
          </dl>
          <ResultLine trip={trip} />
        </div>
      </div>
    </Card>
  );
}
