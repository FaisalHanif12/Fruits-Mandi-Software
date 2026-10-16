// One lot of a saved بکری, read-only (plan 13): what the lot was and its sale lines. Every number
// is the value stored when the بکری was saved (PRD FR-11.6, Code Standards B20): nothing is worked
// out again with today's rules, and nothing is added up here (CLAUDE.md rule 1). The sums of the
// lot are in the summary under the lots (SavedSummary). Lines replaced by an edit (plan 16) are
// not shown.

import { Localized } from '@fluent/react';
import type { SavedLot } from '../../../../bindings/SavedLot';
import type { SavedSale } from '../../../../bindings/SavedSale';
import { isFromStep, useLayoutStep } from '../../../../shared/hooks';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { Card, Fact, Num, Table, type TableColumn } from '../../../../shared/ui';
import { useLocalization } from '@fluent/react';

/** The buyer of a line: the one-time person by his own name, else the account (PRD FR-13.2). */
function buyerName(sale: SavedSale, language: Parameters<typeof displayName>[1]): string {
  return displayName(sale.pukhta ?? sale.buyer, language);
}

function lineColumns(
  language: Parameters<typeof displayName>[1],
  say: (id: string) => string,
  step: ReturnType<typeof useLayoutStep>,
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
  const seller = displayName(lot.pukhta ?? lot.seller, language);
  const lines = lot.sales.filter((sale) => sale.is_active);
  return (
    <Card
      compact
      title={
        <Localized id="bikri-saved-lot-title" vars={{ lot: String(lot.lot_no), seller }}>
          <span />
        </Localized>
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
        {lot.marka_text === null ? null : (
          <Fact label={<Localized id="glossary-marka" />}>{lot.marka_text}</Fact>
        )}
      </dl>
      <Table
        label={l10n.getString('bikri-sales-label')}
        columns={lineColumns(language, say, step)}
        rows={lines}
        rowKey={(sale) => String(sale.id)}
      />
    </Card>
  );
}
