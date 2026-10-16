// A saved بکری opened read-only (PRD FR-12.13, plan 13). It has the layout of the sale book: the
// toolbar above, the header, the lots with their lines, and the partners' share of its income. It
// shows the values stored when the بکری was saved and works nothing out again (Code Standards B20,
// CLAUDE.md rule 1). Editing and cancelling come in plan 16.
//
// A cancelled بکری says so in words and in red, with the reason (NFR-USE-12).

import { Localized } from '@fluent/react';
import { useQuery } from '@tanstack/react-query';
import { Ban } from 'lucide-react';
import type { BikriDetail } from '../../../../bindings/BikriDetail';
import { isoToDisplayDate, utcToLocalDisplay } from '../../../../shared/format';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { findMaster } from '../../../../shared/search';
import { Card, Code, Fact, Num, Table, type TableColumn } from '../../../../shared/ui';
import { useLocalization } from '@fluent/react';
import type { SavedPosting } from '../../../../bindings/SavedPosting';
import type { SavedSplit } from '../../../../bindings/SavedSplit';
import { SavedLotCard } from './SavedLotCard';
import { SavedSummary } from './SavedSummary';

const POSTING_LABEL: Record<string, string> = {
  bikri_first: 'bikri-posting-first',
  bikri_later: 'bikri-posting-later',
  reversal: 'bikri-posting-reversal',
};

/** The name of a master record the detail holds only by id (the truck stand), or null. */
function useMasterName(scope: 'adda' | 'store', id: number | null) {
  const language = useLanguage();
  const query = useQuery({
    queryKey: ['search', 'master-name', scope, id],
    queryFn: () => findMaster(scope, id ?? 0),
    enabled: id !== null,
    staleTime: Infinity,
  });
  return query.data === null || query.data === undefined ? null : displayName(query.data, language);
}

export function SavedBikriView({ detail }: { detail: BikriDetail }) {
  const language = useLanguage();
  const { l10n } = useLocalization();
  const adda = useMasterName('adda', detail.adda_id);
  const store = useMasterName('store', detail.store_id);
  const cancelled = detail.status === 'cancelled';

  const splitColumns: readonly TableColumn<SavedSplit>[] = [
    {
      id: 'partner',
      header: l10n.getString('glossary-shirakat-dar'),
      cell: (row) => displayName(row, language),
    },
    {
      id: 'income',
      header: l10n.getString('bikri-split-income'),
      kind: 'number',
      cell: (row) => <Num paisa={row.income_paisa} />,
    },
  ];
  const postingColumns: readonly TableColumn<SavedPosting>[] = [
    {
      id: 'kind',
      header: l10n.getString('bikri-posting-kind'),
      cell: (row) => l10n.getString(POSTING_LABEL[row.posting_type] ?? 'bikri-posting-other'),
    },
    {
      id: 'date',
      header: l10n.getString('bikri-header-date'),
      kind: 'number',
      cell: (row) => <Code>{isoToDisplayDate(row.entry_date)}</Code>,
    },
    {
      id: 'total',
      header: l10n.getString('bikri-posting-total'),
      kind: 'number',
      cell: (row) => <Num paisa={row.total} />,
    },
  ];

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-baseline gap-x-4">
        <Localized id="bikri-saved-title" vars={{ number: String(detail.number) }}>
          <h1 className="type-title" />
        </Localized>
        <Localized id="bikri-saved-readonly">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {cancelled ? (
        <div
          role="status"
          className="flex flex-wrap items-center gap-2 rounded-2 border border-due bg-due-soft p-3 text-due"
        >
          <Ban aria-hidden="true" className="size-icon-md shrink-0" />
          <Localized id="bikri-saved-cancelled">
            <span className="type-label" />
          </Localized>
          {detail.cancel_reason === null ? null : (
            <Localized id="bikri-saved-cancel-reason" vars={{ reason: detail.cancel_reason }}>
              <span className="type-body" />
            </Localized>
          )}
        </div>
      ) : null}
      <Card compact>
        <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-4">
          <Fact label={<Localized id="bikri-header-date" />}>
            <Code>{isoToDisplayDate(detail.entry_date)}</Code>
          </Fact>
          <Fact label={<Localized id="bikri-header-bill-date" />}>
            <Code>{isoToDisplayDate(detail.bill_date)}</Code>
          </Fact>
          {detail.truck_no === null ? null : (
            <Fact label={<Localized id="bikri-header-truck" />}>
              <Code>{detail.truck_no}</Code>
            </Fact>
          )}
          {detail.zamindar === null ? null : (
            <Fact label={<Localized id="glossary-zamindar" />}>
              {displayName(detail.zamindar, language)}
            </Fact>
          )}
          {adda === null ? null : <Fact label={<Localized id="glossary-adda" />}>{adda}</Fact>}
          {detail.total_freight === null ? null : (
            <Fact label={<Localized id="bikri-header-total-freight" />}>
              <Num paisa={detail.total_freight} />
            </Fact>
          )}
          {store === null ? null : (
            <Fact label={<Localized id="bikri-header-store" />}>{store}</Fact>
          )}
          {detail.book === null ? null : (
            <Fact label={<Localized id="bikri-header-book" />}>{detail.book}</Fact>
          )}
          {detail.driver_name === null ? null : (
            <Fact label={<Localized id="bikri-header-driver" />}>{detail.driver_name}</Fact>
          )}
          {detail.driver_phone === null ? null : (
            <Fact label={<Localized id="bikri-header-driver-phone" />}>
              <Code>{detail.driver_phone}</Code>
            </Fact>
          )}
          {detail.note === null ? null : (
            <Fact label={<Localized id="bikri-header-note" />}>{detail.note}</Fact>
          )}
          <Fact label={<Localized id="bikri-saved-at" />}>
            <Code>{utcToLocalDisplay(detail.created_at)}</Code>
          </Fact>
        </dl>
      </Card>
      {detail.lots.map((lot) => (
        <SavedLotCard key={lot.id} lot={lot} />
      ))}
      <SavedSummary detail={detail} />
      <div className="grid grid-cols-1 gap-3 normal:grid-cols-2">
        <Card compact title={<Localized id="bikri-split-title" />}>
          <Table
            label={l10n.getString('bikri-split-title')}
            columns={splitColumns}
            rows={detail.split}
            rowKey={(row) => String(row.partner_id)}
          />
        </Card>
        <Card compact title={<Localized id="bikri-postings-title" />}>
          <Table
            label={l10n.getString('bikri-postings-title')}
            columns={postingColumns}
            rows={detail.postings}
            rowKey={(row) => String(row.entry_id)}
          />
        </Card>
      </div>
    </div>
  );
}
