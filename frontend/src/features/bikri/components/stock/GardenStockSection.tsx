// The gardens' goods that have come and are not sold yet, under the lots in باقی مال (PRD FR-12.40).
// They are listed apart, like the trips' goods: they belong to a garden, not to a seller. Each row
// is marked with the number of its goods arrived entry (GA-3), its garden and the day to sell.
// The card of the row that has focus holds every figure and the way to the garden. Selling them
// is done in a بکری: a new one already holds a lot for each entry that is ready (PRD FR-12.39).
//
// The list is the backend's (`bikri_stock_list`); nothing is worked out here.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Sprout } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { GardenStockRow } from '../../../../bindings/GardenStockRow';
import type { StockList } from '../../../../bindings/StockList';
import { isoToDisplayDate } from '../../../../shared/format';
import { ListGrid, type ListContext } from '../../../../shared/grid';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { Button, Card, Code, Fact, Num } from '../../../../shared/ui';
import { arrivalNumberText } from '../../../gardens';
import { GARDEN_STOCK_COLUMNS } from '../../garden-stock-columns';

/** A list with more rows than this fills the room and scrolls inside; shorter ones are as tall as their rows. */
const SHORT_LIST_ROWS = 30;

interface GardenStockSectionProps {
  rows: readonly GardenStockRow[];
  /** The backend's total under the list. */
  list: Pick<StockList, 'garden_balance_total'>;
}

export function GardenStockSection({ rows, list }: GardenStockSectionProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const navigate = useNavigate();
  const [selectedId, setSelectedId] = useState<number | null>(null);
  // The row that has focus; if it left the list (sold out), the first row is used.
  const selected = rows.find((row) => row.arrival_id === selectedId) ?? rows[0] ?? null;
  const columns = useMemo(() => GARDEN_STOCK_COLUMNS, []);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'garden',
      values: {
        garden: context.text('glossary-meezan'),
        balance: String(list.garden_balance_total),
      },
    }),
    [list],
  );

  const openGarden = (row: GardenStockRow) => {
    void navigate({ to: '/gardens/$id', params: { id: String(row.garden.id) } });
  };

  return (
    <section aria-labelledby="bikri-stock-garden-heading" className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <Localized id="bikri-stock-garden-title">
          <h2 id="bikri-stock-garden-heading" className="type-section text-ink" />
        </Localized>
        <Localized id="bikri-stock-garden-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <ListGrid<GardenStockRow>
        columns={columns}
        rows={rows}
        getRowId={(row) => String(row.arrival_id)}
        labelId="bikri-stock-garden-label"
        onOpen={openGarden}
        onFocusRow={(rowId) => {
          setSelectedId(Number(rowId));
        }}
        focusRowId={selectedId === null ? null : String(selectedId)}
        heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
        totals={totals}
      />
      {selected === null ? null : (
        <Card
          compact
          title={
            <Localized
              id="bikri-stock-garden-card-title"
              vars={{
                number: arrivalNumberText(selected.number, say),
                item: displayName(selected.item, language),
              }}
            >
              <span />
            </Localized>
          }
        >
          <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-5">
            <Fact label={<Localized id="glossary-bagh" />}>
              <span className="inline-flex flex-wrap items-center gap-2">
                <Code>{selected.garden.code}</Code>
                <span>{displayName(selected.garden, language)}</span>
              </span>
            </Fact>
            <Fact label={<Localized id="glossary-qism-maal" />}>
              {displayName(selected.item, language)}
            </Fact>
            <Fact label={<Localized id="glossary-packing" />}>
              {displayName(selected.packing, language)}
            </Fact>
            <Fact label={<Localized id="bikri-stock-col-received-date" />}>
              <Code>{isoToDisplayDate(selected.received_date)}</Code>
            </Fact>
            <Fact label={<Localized id="glossary-sell-date" />}>
              <Code>{isoToDisplayDate(selected.sell_date)}</Code>
            </Fact>
            <Fact label={<Localized id="bikri-stock-col-days" />}>
              <Num>{selected.days_waiting}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-stock-garden-col-arrived" />}>
              <Num>{selected.qty_arrived}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-saved-qty-sold" />}>
              <Num>{selected.qty_sold}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-stock-col-written-off" />}>
              <Num>{selected.qty_written_off}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-col-balance" />}>
              <Num>{selected.balance}</Num>
            </Fact>
          </dl>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              onClick={() => {
                openGarden(selected);
              }}
            >
              <Sprout aria-hidden="true" className="size-icon-md" />
              <Localized id="bikri-stock-garden-open" />
            </Button>
          </div>
        </Card>
      )}
    </section>
  );
}
