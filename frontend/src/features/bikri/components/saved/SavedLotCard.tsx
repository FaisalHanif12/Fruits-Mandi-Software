// One lot of a saved بکری, read-only (plan 13): what the lot was and its sale lines. Every number
// is the value stored when the بکری was saved (PRD FR-11.6, Code Standards B20): nothing is worked
// out again with today's rules, and nothing is added up here (CLAUDE.md rule 1). The sums of the
// lot are in the summary under the lots (SavedSummary). Lines replaced by an edit (plan 16) are
// not shown. A lot of a trip's goods is marked, names its purchase and trip (a link to the trip),
// shows the cost of the goods sold on each line in a wide window, and gives the seller credit,
// the cost and the result with each trip partner's part (plan 16b, PRD FR-12.26, C25). The saved
// numbers are shown as stored; nothing is worked out again.

import { Localized } from '@fluent/react';
import type { SavedLot } from '../../../../bindings/SavedLot';
import type { SavedSale } from '../../../../bindings/SavedSale';
import { isFromStep, useLayoutStep } from '../../../../shared/hooks';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { useNavigate } from '@tanstack/react-router';
import { Card, Fact, LinkButton, Num, Table, type TableColumn } from '../../../../shared/ui';
import { isNegativePaisa } from '../../../../shared/format';
import { purchaseNumberText } from '../../../purchases';
import { tripNumberText } from '../../../trips';
import { useLocalization } from '@fluent/react';

/** The buyer of a line: the one-time person by his own name, else the account (PRD FR-13.2). */
function buyerName(sale: SavedSale, language: Parameters<typeof displayName>[1]): string {
  return displayName(sale.pukhta ?? sale.buyer, language);
}

function lineColumns(
  language: Parameters<typeof displayName>[1],
  say: (id: string) => string,
  step: ReturnType<typeof useLayoutStep>,
  own: boolean,
): readonly TableColumn<SavedSale>[] {
  return [
    {
      id: 'buyer',
      header: say('glossary-gahak'),
      cell: (sale) => buyerName(sale, language),
    },
    {
      id: 'qty',
      header: say('bikri-col-qty'),
      kind: 'number',
      cell: (sale) => <Num>{String(sale.qty)}</Num>,
    },
    {
      id: 'rate',
      header: say('bikri-col-rate'),
      kind: 'number',
      cell: (sale) => <Num paisa={sale.seller_rate} />,
    },
    {
      id: 'amount',
      header: say('bikri-col-seller-amount'),
      kind: 'number',
      cell: (sale) => <Num paisa={sale.seller_amount} />,
    },
    ...(isFromStep(step, 'normal')
      ? ([
          {
            id: 'seller-comm',
            header: say('bikri-col-seller-comm'),
            kind: 'number',
            cell: (sale) => <Num paisa={sale.seller_comm.amount} />,
          },
          {
            id: 'buyer-comm',
            header: say('bikri-col-buyer-comm'),
            kind: 'number',
            cell: (sale) => <Num paisa={sale.buyer_comm.amount} />,
          },
        ] satisfies readonly TableColumn<SavedSale>[])
      : []),
    ...(isFromStep(step, 'wide')
      ? ([
          {
            id: 'laga',
            header: say('glossary-laga'),
            kind: 'number',
            cell: (sale) => <Num paisa={sale.laga.amount} />,
          },
        ] satisfies readonly TableColumn<SavedSale>[])
      : []),
    {
      id: 'buyer-total',
      header: say('bikri-col-buyer-total'),
      kind: 'number',
      cell: (sale) => <Num paisa={sale.buyer_total} />,
    },
    ...(own && isFromStep(step, 'wide')
      ? ([
          {
            id: 'cost',
            header: say('bikri-saved-cost'),
            kind: 'number',
            cell: (sale) => <Num paisa={sale.cost_paisa} />,
          },
        ] satisfies readonly TableColumn<SavedSale>[])
      : []),
    {
      id: 'payment',
      header: say('bikri-col-payment'),
      cell: (sale) => say(sale.payment_type === 'cash' ? 'glossary-naqdi' : 'glossary-udhari'),
    },
  ];
}

interface SavedLotCardProps {
  lot: SavedLot;
}

export function SavedLotCard({ lot }: SavedLotCardProps) {
  const language = useLanguage();
  const step = useLayoutStep();
  const { l10n } = useLocalization();
  const say = (id: string) => l10n.getString(id);
  const navigate = useNavigate();
  const purchase = lot.purchase;
  const seller = displayName(lot.pukhta ?? lot.seller, language);
  const lines = lot.sales.filter((sale) => sale.is_active);
  return (
    <Card
      compact
      title={
        <span className="inline-flex flex-wrap items-center gap-2">
          <Localized id="bikri-saved-lot-title" vars={{ lot: String(lot.lot_no), seller }}>
            <span />
          </Localized>
          {purchase === null ? null : (
            <Localized id="glossary-apna-maal">
              <span className="type-chip rounded-1 bg-surface-sunken px-2 text-ink-muted" />
            </Localized>
          )}
        </span>
      }
    >
      <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-4">
        <Fact label={<Localized id="glossary-qism-maal" />}>{displayName(lot.item, language)}</Fact>
        <Fact label={<Localized id="glossary-packing" />}>
          {displayName(lot.packing, language)}
        </Fact>
        <Fact label={<Localized id="bikri-col-qty-received" />}>
          <Num>{String(lot.qty_received)}</Num>
        </Fact>
        <Fact label={<Localized id="bikri-saved-qty-sold" />}>
          <Num>{String(lot.qty_sold)}</Num>
        </Fact>
        {lot.qty_written_off === 0 ? null : (
          <Fact label={<Localized id="bikri-saved-qty-written-off" />}>
            <Num>{String(lot.qty_written_off)}</Num>
          </Fact>
        )}
        {lot.qty_taken === 0 ? null : (
          <Fact label={<Localized id="bikri-saved-qty-taken" />}>
            <Num>{String(lot.qty_taken)}</Num>
          </Fact>
        )}
        {purchase === null ? null : (
          <Fact label={<Localized id="bikri-saved-purchase" />}>
            <LinkButton
              onClick={() => {
                void navigate({
                  to: '/trips/$id',
                  params: { id: String(purchase.trip_id) },
                  search: { purchase: purchase.id },
                });
              }}
            >
              <Localized
                id="bikri-saved-purchase-open"
                vars={{
                  number: purchaseNumberText(purchase.number, (id, vars) =>
                    l10n.getString(id, vars),
                  ),
                  trip: tripNumberText(purchase.trip_number, (id, vars) =>
                    l10n.getString(id, vars),
                  ),
                }}
              />
            </LinkButton>
          </Fact>
        )}
        {lot.marka_text === null ? null : (
          <Fact label={<Localized id="glossary-marka" />}>{lot.marka_text}</Fact>
        )}
      </dl>
      {lot.own === null ? null : (
        <div className="flex flex-col gap-1">
          <Localized id="bikri-saved-own-title">
            <h3 className="type-label text-ink-muted" />
          </Localized>
          <dl className="flex flex-col">
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="bikri-own-credit">
                <dt />
              </Localized>
              <dd>
                <Num paisa={lot.own.credit_paisa} />
              </dd>
            </div>
            {lot.own.credits.map((credit) => (
              <div
                key={credit.partner.id}
                className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink"
              >
                <dt lang={language}>{displayName(credit.partner, language)}</dt>
                <dd>
                  <Num paisa={credit.amount_paisa} />
                </dd>
              </div>
            ))}
            <div className="flex items-center justify-between gap-3 border-b border-border py-1 type-body text-ink">
              <Localized id="purchases-detail-cost-sold">
                <dt />
              </Localized>
              <dd>
                <Num paisa={lot.own.cost_paisa} />
              </dd>
            </div>
            <div
              className={`flex items-center justify-between gap-3 border-b border-border py-1 type-label ${isNegativePaisa(lot.own.result_paisa) ? 'text-due' : 'text-ink'}`}
            >
              <Localized
                id={
                  isNegativePaisa(lot.own.result_paisa)
                    ? 'glossary-tijarati-nuqsan'
                    : 'glossary-tijarati-munafa'
                }
              >
                <dt />
              </Localized>
              <dd>
                <Num paisa={lot.own.result_paisa} />
              </dd>
            </div>
          </dl>
        </div>
      )}
      <Table
        label={l10n.getString('bikri-sales-label')}
        columns={lineColumns(language, say, step, purchase !== null)}
        rows={lines}
        rowKey={(sale) => String(sale.id)}
      />
    </Card>
  );
}
