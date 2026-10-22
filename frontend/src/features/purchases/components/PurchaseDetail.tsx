// The purchase that has focus in the trip's list (PRD FR-12.25, FR-12.27, FR-12.28): every figure
// of it, the parts that were paid and where they came from, the sales that took goods from it,
// its write-offs, and what can be done with it: write off spoiled goods, undo a write-off,
// cancel the purchase. The list shows the columns that fit the window; this card holds the rest,
// so nothing needs a sideways scroll.
//
// The result is the backend's number with its ⓘ (sales money + other shops − cost sold − cost
// written off). A loss is red, and it is also written: the label says "Trading Loss" and a line
// under it says it is a loss, so colour is never the only signal (NFR-USE-12).
//
// Four states: loading, an error with a retry, and the card. A cancelled purchase has a red
// banner with the reason and no actions.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import type { UseQueryResult } from '@tanstack/react-query';
import { Ban, RotateCcw, TriangleAlert } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { isNegativePaisa, isoToDisplayDate } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  Button,
  CalcInfo,
  Card,
  Code,
  EntryPartsList,
  ErrorState,
  Fact,
  LinkButton,
  LoadingState,
  Num,
  Table,
  type TableColumn,
} from '../../../shared/ui';
import { purchaseNumberText, supplierText } from '../list-columns';
import type { PurchaseLotRef, PurchaseRow, WriteOffRow } from '../types';

interface PurchaseDetailProps {
  query: UseQueryResult<PurchaseRow>;
  onWriteOff: (row: PurchaseRow) => void;
  onUndoWriteOff: (row: PurchaseRow) => void;
  onCancel: (row: PurchaseRow) => void;
}

export function PurchaseDetail({
  query,
  onWriteOff,
  onUndoWriteOff,
  onCancel,
}: PurchaseDetailProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const navigate = useNavigate();
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  if (query.isPending) return <LoadingState rows={4} />;
  if (query.isError) {
    return (
      <ErrorState
        title={<Localized id="purchases-detail-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  }
  const row = query.data;
  const cancelled = row.status === 'cancelled';
  const loss = isNegativePaisa(row.result_paisa);

  const lotColumns: readonly TableColumn<PurchaseLotRef>[] = [
    {
      id: 'bikri',
      header: l10n.getString('purchases-lot-bikri'),
      cell: (lot) => (
        <LinkButton
          onClick={() => {
            void navigate({ to: '/bikri/book', search: { id: lot.bikri_id } });
          }}
        >
          <Localized
            id="purchases-lot-open"
            vars={{ number: String(lot.bikri_number), lot: String(lot.lot_no) }}
          />
        </LinkButton>
      ),
    },
    {
      id: 'date',
      header: l10n.getString('purchases-col-date'),
      kind: 'number',
      cell: (lot) => <Code>{isoToDisplayDate(lot.entry_date)}</Code>,
    },
    {
      id: 'qty',
      header: l10n.getString('purchases-lot-qty-sold'),
      kind: 'number',
      cell: (lot) => <Num>{String(lot.qty_sold)}</Num>,
    },
    {
      id: 'sales',
      header: l10n.getString('purchases-lot-sales'),
      kind: 'number',
      cell: (lot) => <Num paisa={lot.sales_paisa} />,
    },
    {
      id: 'cost',
      header: l10n.getString('purchases-detail-cost-sold'),
      kind: 'number',
      cell: (lot) => <Num paisa={lot.cost_paisa} />,
    },
  ];
  const writeOffColumns: readonly TableColumn<WriteOffRow>[] = [
    {
      id: 'date',
      header: l10n.getString('purchases-col-date'),
      kind: 'number',
      cell: (item) => <Code>{isoToDisplayDate(item.move_date)}</Code>,
    },
    {
      id: 'qty',
      header: l10n.getString('purchases-write-off-qty'),
      kind: 'number',
      cell: (item) => <Num>{String(item.qty)}</Num>,
    },
    {
      id: 'reason',
      header: l10n.getString('purchases-write-off-reason'),
      cell: (item) => item.reason ?? '',
    },
  ];

  const resultRows: readonly { id: string; label: string; paisa: PurchaseRow['sales_paisa'] }[] = [
    { id: 'sales', label: 'purchases-detail-sales', paisa: row.sales_paisa },
    { id: 'other', label: 'purchases-detail-other-shop', paisa: row.other_shop_paisa },
    { id: 'sold', label: 'purchases-detail-cost-sold', paisa: row.cost_sold_paisa },
    { id: 'off', label: 'purchases-detail-cost-written-off', paisa: row.cost_written_off_paisa },
  ];

  return (
    <Card
      compact
      title={
        <Localized
          id="purchases-detail-title"
          vars={{
            number: purchaseNumberText(row.number, say),
            item: displayName(row.item, language),
          }}
        >
          <span />
        </Localized>
      }
    >
      {cancelled ? (
        <div
          role="status"
          className="flex flex-wrap items-center gap-2 rounded-2 border border-due bg-due-soft p-3 text-due"
        >
          <Ban aria-hidden="true" className="size-icon-md shrink-0" />
          <Localized id="purchases-detail-cancelled">
            <span className="type-label" />
          </Localized>
          {row.cancel_reason === null ? null : (
            <Localized id="purchases-detail-cancel-reason" vars={{ reason: row.cancel_reason }}>
              <span className="type-body" />
            </Localized>
          )}
        </div>
      ) : null}
      <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-5">
        <Fact label={<Localized id="purchases-field-supplier" />}>
          {supplierText(row.supplier, say)}
        </Fact>
        <Fact label={<Localized id="purchases-field-date" />}>
          <Code>{isoToDisplayDate(row.entry_date)}</Code>
        </Fact>
        <Fact label={<Localized id="glossary-qism-maal" />}>{displayName(row.item, language)}</Fact>
        <Fact label={<Localized id="glossary-packing" />}>
          {displayName(row.packing, language)}
        </Fact>
        <Fact label={<Localized id="purchases-col-qty" />}>
          <Num>{String(row.qty)}</Num>
        </Fact>
        <Fact label={<Localized id="purchases-lot-qty-sold" />}>
          <Num>{String(row.qty_sold)}</Num>
        </Fact>
        <Fact label={<Localized id="purchases-detail-written-off" />}>
          <Num>{String(row.qty_written_off)}</Num>
        </Fact>
        <Fact label={<Localized id="purchases-col-left" />}>
          <Num>{String(row.qty_left)}</Num>
        </Fact>
        <Fact label={<Localized id="purchases-detail-days" />}>
          <Num>{String(row.days_waiting)}</Num>
        </Fact>
        <Fact label={<Localized id="purchases-line-paid-now" />}>
          <Num paisa={row.paid_now_paisa} />
        </Fact>
        <Fact label={<Localized id="purchases-detail-trip-cost" />}>
          <Num paisa={row.trip_cost_paisa} />
        </Fact>
        <Fact label={<Localized id="purchases-line-total-cost" />}>
          <Num paisa={row.total_cost_paisa} />
        </Fact>
        <Fact label={<Localized id="purchases-line-avg-cost" />}>
          <Num paisa={row.avg_cost_paisa} />
        </Fact>
        <Fact label={<Localized id="purchases-col-cost-left" />}>
          <Num paisa={row.cost_left_paisa} />
        </Fact>
        {row.note === null ? null : (
          <Fact label={<Localized id="purchases-field-note" />}>{row.note}</Fact>
        )}
      </dl>
      {row.parts.length === 0 ? null : (
        <div className="flex flex-col gap-1">
          <Localized id="purchases-detail-parts-title">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <EntryPartsList
            parts={row.parts}
            label={l10n.getString('purchases-detail-parts-title')}
          />
        </div>
      )}
      <div className="flex flex-col gap-1">
        <Localized id="purchases-result-title">
          <h3 className="type-label text-ink-muted" />
        </Localized>
        <dl className="flex flex-col">
          {resultRows.map((item) => (
            <div
              key={item.id}
              className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink"
            >
              <dt>{l10n.getString(item.label)}</dt>
              <dd>
                <Num paisa={item.paisa} />
              </dd>
            </div>
          ))}
          <div
            className={`flex items-center justify-between gap-3 border-b border-border py-1 type-label ${loss ? 'text-due' : 'text-ink'}`}
          >
            <Localized id={loss ? 'glossary-tijarati-nuqsan' : 'glossary-tijarati-munafa'}>
              <dt />
            </Localized>
            <dd className="flex items-center gap-1">
              <Num paisa={row.result_paisa} />
              <CalcInfo working={row.result_working} inCell />
            </dd>
          </div>
        </dl>
        {loss ? (
          <p className="flex items-center gap-2 type-body text-due">
            <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
            <Localized id="purchases-loss-note">
              <span />
            </Localized>
          </p>
        ) : null}
      </div>
      {row.lots.length === 0 ? null : (
        <Table
          label={l10n.getString('purchases-lots-title')}
          columns={lotColumns}
          rows={row.lots}
          rowKey={(lot) => String(lot.lot_id)}
        />
      )}
      {row.write_offs.length === 0 ? null : (
        <Table
          label={l10n.getString('purchases-write-offs-title')}
          columns={writeOffColumns}
          rows={row.write_offs}
          rowKey={(item) => String(item.move_id)}
        />
      )}
      {cancelled ? null : (
        <div className="flex flex-wrap items-center gap-2">
          <Button
            disabled={row.qty_left <= 0}
            onClick={() => {
              onWriteOff(row);
            }}
          >
            <Ban aria-hidden="true" className="size-icon-md" />
            <Localized id="purchases-write-off" />
          </Button>
          <Button
            disabled={row.write_offs.length === 0}
            onClick={() => {
              onUndoWriteOff(row);
            }}
          >
            <RotateCcw aria-hidden="true" className="size-icon-md" />
            <Localized id="purchases-undo-write-off" />
          </Button>
          <Button
            variant="danger"
            onClick={() => {
              onCancel(row);
            }}
          >
            <Ban aria-hidden="true" className="size-icon-md" />
            <Localized id="purchases-cancel" />
          </Button>
        </div>
      )}
    </Card>
  );
}
