// The four sections of the total dues page (PRD FR-9.7, FR-19.11): the buyers who owe, the sellers who owe the shop,
// the dead amount and the people who borrowed (ادھار رقم). Each one is a card with its count and its total from
// `dashboard_total_dues`, and its own list, one page at a time, read by the command of its own page, so
// a section is the page behind its tile. Each row opens that person's history. The figures are the
// backend's; nothing is added up here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { HandCoins, Handshake, UserRound, UserX, Users, type LucideIcon } from 'lucide-react';
import { useCallback, type ReactNode } from 'react';
import type { BuyerDuesRow } from '../../../bindings/BuyerDuesRow';
import type { DeadRow } from '../../../bindings/DeadRow';
import type { LentOpenRow } from '../../../bindings/LentOpenRow';
import type { SellerDuesRow } from '../../../bindings/SellerDuesRow';
import type { TotalDuesPart } from '../../../bindings/TotalDuesPart';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListColumn } from '../../../shared/grid';
import { Card, EmptyState, Num } from '../../../shared/ui';
import { lentSearchOf } from '../dues-links';
import { useBuyerDues, useDeadOwed, useLentOpen, useSellerDues } from '../dues-hooks';
import { BUYER_DUES_COLUMNS } from '../dues-columns';
import { DEAD_OWED_COLUMNS, LENT_OPEN_COLUMNS } from '../other-dues-columns';
import { SELLER_DUES_COLUMNS } from '../seller-dues-columns';

/** The columns of the sellers who owe the shop: what he owes beyond his advances, not what the shop owes him. */
const SELLER_OWES_FIELDS: ReadonlySet<string> = new Set([
  'code',
  'seller',
  'city',
  'owes',
  'last_change',
]);
const SELLER_OWES_COLUMNS: readonly ListColumn<SellerDuesRow>[] = SELLER_DUES_COLUMNS.filter(
  (column) => SELLER_OWES_FIELDS.has(column.field),
);

interface SectionProps {
  icon: LucideIcon;
  /** The Fluent message ID of the section's name. */
  titleId: string;
  /** What the backend counted for the section, or undefined while it is on its way. */
  part: TotalDuesPart | undefined;
  children: ReactNode;
}

/** One card: the name, how many people and what they owe together (the backend's), then the list. */
function Section({ icon, titleId, part, children }: SectionProps) {
  return (
    <Card compact icon={icon} title={<Localized id={titleId} />}>
      {part === undefined ? null : (
        <p className="type-body flex flex-wrap items-center justify-between gap-2 text-ink-muted">
          <Localized
            id="dashboard-tile-dead-note"
            vars={{ count: part.count }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
          <Localized
            id="total-dues-section-total"
            elems={{ money: <Num inherit paisa={part.total_paisa} /> }}
          >
            <span className="type-field text-ink" />
          </Localized>
        </p>
      )}
      {children}
    </Card>
  );
}

function emptySection(icon: LucideIcon, id: string) {
  return (
    <EmptyState
      icon={icon}
      title={<Localized id={`total-dues-${id}-empty`} />}
      help={<Localized id={`total-dues-${id}-empty-help`} />}
    />
  );
}

export function BuyersSection({ part }: { part: TotalDuesPart | undefined }) {
  const navigate = useNavigate();
  const list = useBuyerDues({ search: '', partyId: null });
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const open = useCallback(
    (row: BuyerDuesRow) => {
      void navigate({
        to: '/bikri/party-ledgers',
        search: { side: 'buyer', party: row.party.account_id },
      });
    },
    [navigate],
  );
  return (
    <Section icon={Users} titleId="total-dues-buyers" part={part}>
      <PagedList<BuyerDuesRow>
        columns={BUYER_DUES_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.party.account_id)}
        labelId="dues-label"
        failedId="dues-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        autoHeight
        empty={emptySection(Users, 'buyers')}
        onOpen={open}
      />
    </Section>
  );
}

export function SellersSection({ part }: { part: TotalDuesPart | undefined }) {
  const navigate = useNavigate();
  const list = useSellerDues(
    { side: 'seller_owes', partyId: null },
    'dashboard.total_dues.sellers',
  );
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const open = useCallback(
    (row: SellerDuesRow) => {
      void navigate({
        to: '/bikri/party-ledgers',
        search: { side: 'seller', party: row.party.account_id },
      });
    },
    [navigate],
  );
  return (
    <Section icon={UserRound} titleId="total-dues-sellers" part={part}>
      <PagedList<SellerDuesRow>
        columns={SELLER_OWES_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.party.account_id)}
        labelId="seller-dues-label"
        failedId="seller-dues-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        autoHeight
        empty={emptySection(UserRound, 'sellers')}
        onOpen={open}
      />
    </Section>
  );
}

export function DeadSection({ part }: { part: TotalDuesPart | undefined }) {
  const navigate = useNavigate();
  const list = useDeadOwed();
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  // A buyer or a seller opens his history on پارٹی کھاتے, one row for each بکری, as the buyers and sellers
  // sections do (PRD FR-19.8 v1.43, FR-9.7); a person who borrowed opens his loans.
  const open = useCallback(
    (row: DeadRow) => {
      if (row.party_type === 'money_lent') {
        void navigate({
          to: '/rokar/lent',
          search: {
            type: 'money_lent',
            party: {
              id: row.account_id,
              code: row.code,
              name_ur: row.name_ur,
              name_en: row.name_en,
              ...(row.tag === null || row.tag === '' ? {} : { tag: row.tag }),
            },
          },
        });
        return;
      }
      void navigate({
        to: '/bikri/party-ledgers',
        search: { side: row.party_type, party: row.account_id },
      });
    },
    [navigate],
  );
  return (
    <Section icon={UserX} titleId="total-dues-dead" part={part}>
      <PagedList<DeadRow>
        columns={DEAD_OWED_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.mark_id)}
        labelId="total-dues-dead-label"
        failedId="total-dues-dead-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        autoHeight
        empty={emptySection(UserX, 'dead')}
        onOpen={open}
      />
    </Section>
  );
}

export function OthersSection({ part }: { part: TotalDuesPart | undefined }) {
  const navigate = useNavigate();
  const list = useLentOpen('dashboard.total_dues.others');
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const open = useCallback(
    (row: LentOpenRow) => {
      void navigate({ to: '/rokar/lent', search: lentSearchOf(row.party) });
    },
    [navigate],
  );
  return (
    <Section icon={Handshake} titleId="total-dues-others" part={part}>
      <PagedList<LentOpenRow>
        columns={LENT_OPEN_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.party.account_id)}
        labelId="lent-open-label"
        failedId="lent-open-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        autoHeight
        empty={emptySection(HandCoins, 'others')}
        onOpen={open}
      />
    </Section>
  );
}
