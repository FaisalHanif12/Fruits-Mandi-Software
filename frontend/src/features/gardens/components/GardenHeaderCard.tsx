// The facts of one garden (PRD 12.11, FR-12.34, FR-12.37, FR-12.38), in two parts so the entry form
// comes right after the short one: the header (number, name, state, the four key figures that are
// always in view, the running balance, the rent still due, the commission and the money in, with the
// working behind the ⓘ of the two that have one, then the day, the fruits, the end of the season and
// a note) and, under the history, the figures that fold, open when the page opens (each partner with
// his % and where his money comes from by default, the rent, what was paid from the shop and from
// pockets, the money in, the garden's account, and each partner's money; plan 96, PRD FR-12.53). The
// running balance starts in minus and is shown with its sign: a minus is not an alarm here, it is the
// money paid and not yet back (FR-12.37).
// Everything is the backend's; nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { Calculator, Pencil } from 'lucide-react';
import type { ReactNode } from 'react';
import { formatBps, isoToDisplayDate, isZeroPaisa } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  Button,
  CalcInfo,
  Card,
  Code,
  Fact,
  FoldSection,
  IconChip,
  Num,
  PartnerLockedBox,
  STATUS_CLOSED,
  STATUS_OPEN,
  StatusChip,
} from '../../../shared/ui';
import type { GardenPartnerResult, GardenRow } from '../types';
import { GardenPartnersCard } from './GardenPartnersCard';

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
      <dd className="value-col">{children}</dd>
    </div>
  );
}

/**
 * The four figures of the garden that are always in view, as small cards at the head of the header (plan 75,
 * plan 96): the running balance and the commission with their working, the rent still due and the money in.
 */
function KeyFigures({ garden }: GardenCardProps) {
  return (
    <dl className="grid grid-cols-2 gap-3 @form4:grid-cols-4">
      <Fact stat label={<Localized id="gardens-balance" />}>
        <span className="inline-flex items-center gap-1">
          <span>
            <Num inherit paisa={garden.balance_paisa} />
          </span>
          <CalcInfo working={garden.balance_working} inCell />
        </span>
      </Fact>
      <Fact stat label={<Localized id="gardens-rent-due" />}>
        <Num inherit paisa={garden.rent_due_paisa} />
      </Fact>
      <Fact stat label={<Localized id="glossary-commission" />}>
        <span className="inline-flex items-center gap-1">
          <span>
            <Num inherit paisa={garden.commission_paisa} />
          </span>
          <CalcInfo working={garden.commission_working} inCell />
        </span>
      </Fact>
      <Fact stat label={<Localized id="gardens-fact-money-in" />}>
        <Num inherit paisa={garden.money_in_paisa} />
      </Fact>
    </dl>
  );
}

interface GardenHeaderCardProps extends GardenCardProps {
  /** Edit was pressed: the page opens the header for edit (plan 44). Without it there is no button. */
  onEdit?: (() => void) | undefined;
}

export function GardenHeaderCard({ garden, onEdit }: GardenHeaderCardProps) {
  const language = useLanguage();
  const open = garden.status === 'open';
  return (
    <Card
      compact
      title={
        <span className="inline-flex flex-wrap items-center gap-2">
          <Localized id="gardens-detail-title" vars={{ code: garden.code }}>
            <span />
          </Localized>
          <span lang={language}>{displayName(garden, language)}</span>
          <StatusChip {...(open ? STATUS_OPEN : STATUS_CLOSED)}>
            <Localized id={open ? 'gardens-status-open' : 'gardens-status-closed'} />
          </StatusChip>
        </span>
      }
    >
      <div className="@container min-w-0 flex flex-col gap-3">
        <KeyFigures garden={garden} />
        <dl className="grid grid-cols-2 gap-x-4 gap-y-3 @form3:grid-cols-4">
          <Fact label={<Localized id="gardens-field-date" />}>
            <Code>{isoToDisplayDate(garden.entry_date)}</Code>
          </Fact>
          <Fact label={<Localized id="glossary-qism-maal" />}>
            {garden.items.length === 0 ? (
              <Localized id="gardens-none" />
            ) : (
              // Every item of the garden, the main fruit first (plan 78 D2).
              <span lang={language}>
                {garden.items
                  .map((item) => displayName(item, language))
                  .join(language === 'ur' ? '، ' : ', ')}
              </span>
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
      </div>
      {onEdit === undefined ? null : (
        <div>
          <Button onClick={onEdit}>
            <Pencil aria-hidden="true" className="size-icon-md" />
            <Localized id="gardens-header-edit" />
          </Button>
        </div>
      )}
    </Card>
  );
}

interface GardenFiguresFoldProps extends GardenCardProps {
  /** Each partner's money in the garden; not shown while the partner section is locked. */
  results: readonly GardenPartnerResult[];
  /** The partner code is not given: the partners' money is behind it (PRD FR-17.11). */
  partnersLocked: boolean;
  /** A closed garden shows the result of the closing; an open one shows what is paid so far. */
  closed: boolean;
}

export function GardenFiguresFold({
  garden,
  results,
  partnersLocked,
  closed,
}: GardenFiguresFoldProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  // After a relief (plan 78 D3, PRD FR-12.51) the rent is the one now; the rent the garden was made with, the
  // relief, what the owner paid back and what he still owes show under it. A line that is 0 or not there is left out.
  const relief: readonly { id: string; message: string; paisa: GardenRow['rent_paisa'] }[] = [
    ...(garden.rent_original_paisa === null
      ? []
      : [
          {
            id: 'rent-original',
            message: 'gardens-fact-rent-original',
            paisa: garden.rent_original_paisa,
          },
        ]),
    ...(isZeroPaisa(garden.relief_paisa)
      ? []
      : [{ id: 'relief', message: 'gardens-fact-relief', paisa: garden.relief_paisa }]),
    ...(isZeroPaisa(garden.refunds_paisa)
      ? []
      : [{ id: 'refunds', message: 'gardens-fact-refunds', paisa: garden.refunds_paisa }]),
    ...(isZeroPaisa(garden.owner_owes_paisa)
      ? []
      : [{ id: 'owner-owes', message: 'gardens-fact-owner-owes', paisa: garden.owner_owes_paisa }]),
  ];
  const rows: readonly { id: string; message: string; paisa: GardenRow['rent_paisa'] }[] = [
    { id: 'rent', message: 'gardens-fact-rent', paisa: garden.rent_paisa },
    ...relief,
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
    <FoldSection
      defaultOpen
      title={
        <span className="inline-flex items-center gap-2">
          <IconChip icon={Calculator} />
          <Localized id="gardens-figures-title" />
        </span>
      }
    >
      <div className="@container flex min-w-0 flex-col gap-4">
        <div className="grid grid-cols-1 gap-x-6 gap-y-4 @form3:grid-cols-2">
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
                    id={
                      partner.source === 'shop' ? 'ventures-source-shop' : 'ventures-source-pocket'
                    }
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
          </div>
        </div>
        {partnersLocked ? (
          <PartnerLockedBox />
        ) : (
          <GardenPartnersCard results={results} closed={closed} bare />
        )}
      </div>
    </FoldSection>
  );
}
