// The facts of one garden (PRD 12.11, FR-12.34, FR-12.37, FR-12.38), in two cards so the entry form
// comes right after the short one: the header (number, name, state, day, main fruit, end of the
// season, a note, and the running balance and the rent still due in two lines, with the working
// behind the balance's ⓘ) and, under the entry form, the figures (each partner with his % and where
// his money comes from by default, the rent, what was paid from the shop and from pockets, the
// money in, and the garden's account). The running balance starts in minus and is shown with its
// sign: a minus is not an alarm here, it is the money paid and not yet back (FR-12.37).
// Everything is the backend's; nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { CircleCheck, CircleDot } from 'lucide-react';
import type { ReactNode } from 'react';
import { formatBps, isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { CalcInfo, Card, Code, Fact, Num } from '../../../shared/ui';
import type { GardenRow } from '../types';

interface GardenCardProps {
  garden: GardenRow;
}

/** A line of the figures: a word on the start side, a number on the end side. */
function Line({
  label,
  children,
  strong = false,
}: {
  label: string;
  children: ReactNode;
  strong?: boolean;
}) {
  return (
    <div
      className={`flex items-center justify-between gap-3 border-b border-border py-1 text-ink ${strong ? 'type-label' : 'type-body'}`}
    >
      <dt>{label}</dt>
      <dd className="flex items-center gap-1">{children}</dd>
    </div>
  );
}

/** The running balance and the rent still due: two lines, the balance with its working. */
function BalanceLines({ garden }: GardenCardProps) {
  const { l10n } = useLocalization();
  return (
    <dl className="flex flex-col">
      <Line label={l10n.getString('gardens-balance')} strong>
        <Num paisa={garden.balance_paisa} />
        <CalcInfo working={garden.balance_working} inCell />
      </Line>
      <Line label={l10n.getString('gardens-rent-due')}>
        <Num paisa={garden.rent_due_paisa} />
      </Line>
    </dl>
  );
}

export function GardenHeaderCard({ garden }: GardenCardProps) {
  const language = useLanguage();
  const open = garden.status === 'open';
  const StatusIcon = open ? CircleDot : CircleCheck;
  return (
    <Card
      compact
      title={
        <span className="inline-flex flex-wrap items-center gap-2">
          <Localized id="gardens-detail-title" vars={{ code: garden.code }}>
            <span />
          </Localized>
          <span lang={language}>{displayName(garden, language)}</span>
          <span
            className={`inline-flex items-center gap-1 type-chip rounded-1 px-2 ${open ? 'bg-primary-soft text-primary' : 'bg-surface-sunken text-ink-muted'}`}
          >
            <StatusIcon aria-hidden="true" className="size-icon-sm shrink-0" />
            <Localized id={open ? 'gardens-status-open' : 'gardens-status-closed'} />
          </span>
        </span>
      }
    >
      <dl className="grid grid-cols-2 gap-x-4 gap-y-3 normal:grid-cols-4">
        <Fact label={<Localized id="gardens-field-date" />}>
          <Code>{isoToDisplayDate(garden.entry_date)}</Code>
        </Fact>
        <Fact label={<Localized id="glossary-qism-maal" />}>
          {garden.item === null ? (
            <Localized id="gardens-none" />
          ) : (
            <span lang={language}>{displayName(garden.item, language)}</span>
          )}
        </Fact>
        <Fact label={<Localized id="gardens-field-season-end" />}>
          {garden.season_end_date === null ? (
            <Localized id="gardens-none" />
          ) : (
            <Code>{isoToDisplayDate(garden.season_end_date)}</Code>
          )}
        </Fact>
        {garden.closed_date === null ? null : (
          <Fact label={<Localized id="gardens-closed-on" />}>
            <Code>{isoToDisplayDate(garden.closed_date)}</Code>
          </Fact>
        )}
        {garden.note === null ? null : (
          <Fact label={<Localized id="gardens-field-note" />}>{garden.note}</Fact>
        )}
      </dl>
      <BalanceLines garden={garden} />
    </Card>
  );
}

export function GardenFiguresCard({ garden }: GardenCardProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const rows: readonly { id: string; message: string; paisa: GardenRow['rent_paisa'] }[] = [
    { id: 'rent', message: 'gardens-fact-rent', paisa: garden.rent_paisa },
    { id: 'installments', message: 'gardens-fact-installments', paisa: garden.installments_paisa },
    { id: 'costs', message: 'gardens-fact-costs', paisa: garden.costs_paisa },
    { id: 'paid', message: 'gardens-fact-paid', paisa: garden.paid_paisa },
    { id: 'shop', message: 'gardens-fact-paid-shop', paisa: garden.paid_shop_paisa },
    { id: 'pocket', message: 'gardens-fact-paid-pocket', paisa: garden.paid_pocket_paisa },
    { id: 'in', message: 'gardens-fact-money-in', paisa: garden.money_in_paisa },
    { id: 'other', message: 'gardens-fact-other-shop', paisa: garden.other_shop_paisa },
    { id: 'account', message: 'gardens-fact-account', paisa: garden.account_balance_paisa },
  ];
  return (
    <Card compact title={<Localized id="gardens-figures-title" />}>
      <div className="grid grid-cols-1 gap-x-6 gap-y-4 normal:grid-cols-2">
        <div className="flex min-w-0 flex-col gap-1">
          <Localized id="gardens-partners-title">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <ul aria-label={l10n.getString('gardens-partners-title')} className="flex flex-col">
            {garden.partners.map((partner) => (
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
          {garden.shares_locked ? (
            <Localized id="gardens-shares-locked">
              <p className="type-body text-ink-muted" />
            </Localized>
          ) : null}
        </div>
        <div className="flex min-w-0 flex-col gap-3">
          <dl className="flex flex-col">
            {rows.map((row) => (
              <Line key={row.id} label={l10n.getString(row.message)}>
                <Num paisa={row.paisa} />
              </Line>
            ))}
          </dl>
          <dl className="flex flex-col">
            <Line label={l10n.getString('glossary-commission')}>
              <span className="flex items-center gap-1">
                <Num paisa={garden.commission_paisa} />
                <CalcInfo working={garden.commission_working} inCell />
              </span>
            </Line>
          </dl>
          <BalanceLines garden={garden} />
        </div>
      </div>
    </Card>
  );
}
