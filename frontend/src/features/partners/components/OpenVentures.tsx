// The open gardens and trips a partner is in, under his line on the overview (PRD 17.3, FR-12.29,
// plan 17c, plan 99): newest first, one month at a time by the day they were made (PRD FR-17.3,
// FR-19.11), opening on the latest month that has one, a page at a time; the line above keeps the
// count of all the open ones, so none is hidden. Each shows his %, what he paid so far from the
// shop's cash and from his own pocket, the money he got back, and the running balance of a garden or
// the result of a trip. The totals under the columns are the backend's over the month; the gardens'
// balance and the trips' result are kept apart, never added together. A garden or a trip opens from
// its name. Four states: loading, an error with a retry, nothing in this month, and the rows. Every
// number is the backend's and only shown here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Sprout } from 'lucide-react';
import type { ReactNode } from 'react';
import type { PartnerVentureCard } from '../../../bindings/PartnerVentureCard';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { formatBps, isoToDisplayDate, type Paisa } from '../../../shared/format';
import { Pager } from '../../../shared/grid';
import { useListWindow } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  EmptyState,
  ErrorState,
  LinkButton,
  ListWindowBar,
  LoadingState,
  Num,
  PartnerLockedBox,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { usePartnerLatestVentureMonth, usePartnerOpenVentures } from '../hooks';

interface OpenVenturesProps {
  partnerId: number;
  /** His name, for the name of the list read by a screen reader. */
  partnerName: string;
}

/** A total of the balance column with its words: the gardens' balance, then the trips' result. */
function Apart({ labelId, paisa }: { labelId: string; paisa: Paisa }) {
  return (
    <span className="flex flex-col items-center">
      <span className="type-label text-ink-muted">
        <Localized id={labelId} />
      </span>
      <Num paisa={paisa} />
    </span>
  );
}

export function OpenVentures({ partnerId, partnerName }: OpenVenturesProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const language = useLanguage();
  const dayStatus = useDayStatus();
  const latest = usePartnerLatestVentureMonth(partnerId);
  const listWindow = useListWindow({
    kind: 'month',
    source: { kind: 'partner_ventures', partner_id: partnerId },
    today: dayStatus.data?.today,
    opensOn: latest.data,
  });
  // Without the day (it could not be read) the list asks for every open one rather than waiting.
  const windowed = !dayStatus.isError;
  const range = windowed ? listWindow.range : { from: null, to: null };
  // The month to open on is read first, so an older month never flashes as empty. If it cannot be
  // read, the list asks anyway and says what is wrong.
  const placed = !latest.isPending && (listWindow.ready || !windowed);
  const list = usePartnerOpenVentures(partnerId, range, placed);

  const open = (venture: PartnerVentureCard) => {
    void navigate(
      venture.kind === 'garden'
        ? { to: '/gardens/$id', params: { id: String(venture.venture_id) } }
        : { to: '/trips/$id', params: { id: String(venture.venture_id) } },
    );
  };

  const first = list.data?.pages[0];
  const totals = first?.totals;
  const total = (value: ReactNode): { total?: ReactNode } =>
    totals === undefined ? {} : { total: value };
  const columns: readonly TableColumn<PartnerVentureCard>[] = [
    {
      id: 'venture',
      header: l10n.getString('partners-ventures-col-venture'),
      cell: (venture) => {
        const label =
          venture.kind === 'garden'
            ? `${venture.code} ${displayName(venture, language)}`
            : displayName(venture, language);
        return (
          <LinkButton
            onClick={() => {
              open(venture);
            }}
          >
            <span lang={language}>{label}</span>
          </LinkButton>
        );
      },
      ...total(<Localized id="glossary-meezan" />),
    },
    {
      id: 'kind',
      header: l10n.getString('partners-ventures-col-kind'),
      cell: (venture) => (
        <Localized id={venture.kind === 'garden' ? 'glossary-bagh' : 'glossary-safar'}>
          <span />
        </Localized>
      ),
    },
    {
      id: 'made',
      header: l10n.getString('partners-ventures-col-made'),
      kind: 'number',
      size: 'date',
      // A date is never broken at its hyphens.
      cell: (venture) => (
        <span className="whitespace-nowrap">
          <Num>{isoToDisplayDate(venture.entry_date)}</Num>
        </span>
      ),
    },
    {
      id: 'share',
      header: l10n.getString('glossary-hissa'),
      kind: 'number',
      size: 'qty',
      cell: (venture) => <Num>{`${formatBps(venture.share_bps)}%`}</Num>,
    },
    {
      id: 'paid-shop',
      header: l10n.getString('partners-ventures-col-shop'),
      kind: 'number',
      size: 'money',
      cell: (venture) => <Num paisa={venture.paid_shop_paisa} />,
      ...total(totals === undefined ? null : <Num paisa={totals.paid_shop_paisa} />),
    },
    {
      id: 'paid-pocket',
      header: l10n.getString('partners-ventures-col-pocket'),
      kind: 'number',
      size: 'money',
      cell: (venture) => <Num paisa={venture.paid_pocket_paisa} />,
      ...total(totals === undefined ? null : <Num paisa={totals.paid_pocket_paisa} />),
    },
    {
      id: 'back',
      header: l10n.getString('partners-ventures-col-back'),
      kind: 'number',
      size: 'money',
      cell: (venture) => <Num paisa={venture.money_back_paisa} />,
      ...total(totals === undefined ? null : <Num paisa={totals.money_back_paisa} />),
    },
    {
      id: 'balance',
      header: l10n.getString('partners-ventures-col-balance'),
      kind: 'number',
      size: 'money',
      cell: (venture) => <Num paisa={venture.balance_paisa} />,
      ...total(
        totals === undefined ? null : (
          <span className="flex flex-col gap-1">
            <Apart labelId="glossary-baghat" paisa={totals.gardens_balance_paisa} />
            <Apart labelId="partners-ventures-total-trips" paisa={totals.trips_result_paisa} />
          </span>
        ),
      ),
    },
  ];

  const locked =
    list.isError && uiErrorOf(list.error).errors.some((item) => item.code === 'ERR_PARTNER_LOCKED');
  if (locked) return <PartnerLockedBox />;
  const rows = list.data?.pages.flatMap((page) => page.rows);
  let body;
  if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="partners-ventures-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows === undefined) {
    body = <LoadingState rows={2} />;
  } else if (rows.length === 0) {
    body = (
      <EmptyState
        icon={Sprout}
        title={
          <Localized id={windowed ? 'partners-ventures-month-empty' : 'partners-ventures-empty'} />
        }
        help={
          <Localized
            id={windowed ? 'partners-ventures-month-empty-help' : 'partners-ventures-empty-help'}
          />
        }
      />
    );
  } else {
    body = (
      <div className="flex min-w-0 flex-col gap-2">
        <Table
          label={l10n.getString('partners-ventures-label', { name: partnerName })}
          columns={columns}
          rows={rows}
          rowKey={(venture) => `${venture.kind}-${String(venture.venture_id)}`}
        />
        <Pager pager={list.pager} />
      </div>
    );
  }
  return (
    <div className="flex flex-col gap-3 border-t border-border pt-3">
      {body}
      {windowed ? <ListWindowBar window={listWindow} /> : null}
    </div>
  );
}
