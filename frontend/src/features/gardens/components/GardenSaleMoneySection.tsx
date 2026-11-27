// "Where the sale money is" (فروخت کی رقم کہاں ہے, PRD FR-12.53): every sale of the garden's goods,
// in our shop and in another shop, with where its money is: what the buyer was billed, what he paid
// on the spot (into روکڑ), what came in later by his receipts, what is still on his account, and
// what the garden was credited, with the lot charges taken off. One month at a time with the shared
// month bar (PRD FR-19.11), or every month on a tick: then the totals are the garden's money in from
// our shop and from other shops, as R36 has it. A search by buyer, shop, item or sale number. All
// figures and totals are the backend's; nothing is worked out here.

import { Localized } from '@fluent/react';
import { Coins, SearchX } from 'lucide-react';
import { useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { useListWindow } from '../../../shared/hooks';
import { formNavigationProps } from '../../../shared/keyboard';
import { Card, Checkbox, EmptyState, FilterBar, ListWindowBar } from '../../../shared/ui';
import type { GardenRow } from '../types';
import { SaleMoneyList } from './SaleMoneyList';

export function GardenSaleMoneySection({ garden }: { garden: GardenRow }) {
  const dayStatus = useDayStatus();
  const [search, setSearch] = useState('');
  const [everyMonth, setEveryMonth] = useState(false);
  const listWindow = useListWindow({
    kind: 'month',
    source: { kind: 'garden_sales', garden_id: garden.id },
    today: dayStatus.data?.today,
  });
  // Without the day (it could not be read) the list asks for every month rather than waiting.
  const windowed = !everyMonth && !dayStatus.isError;
  const range = windowed ? listWindow.range : { from: null, to: null };

  return (
    <Card compact icon={Coins} title={<Localized id="gardens-sales-title" />}>
      <Localized id="gardens-sales-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      <div {...formNavigationProps} className="max-w-form">
        <FilterBar
          screen="gardens.sales"
          search={{
            value: search,
            onChange: setSearch,
            labelId: 'gardens-sales-search-label',
            placeholderId: 'gardens-sales-search-hint',
          }}
          extra={
            <Checkbox
              checked={everyMonth}
              onChange={setEveryMonth}
              label={<Localized id="gardens-sales-every-month" />}
            />
          }
        />
      </div>
      <SaleMoneyList
        gardenId={garden.id}
        filter={{ from: range.from, to: range.to, arrivalId: null, search }}
        screen="gardens.sales"
        labelId="gardens-sales-label"
        enabled={listWindow.ready || !windowed}
        onPageKey={windowed ? listWindow.onPageKey : undefined}
        empty={
          search.trim() !== '' ? (
            <EmptyState
              icon={SearchX}
              title={<Localized id="gardens-sales-no-match" />}
              help={<Localized id="gardens-sales-no-match-help" />}
            />
          ) : windowed ? (
            <EmptyState
              icon={Coins}
              title={<Localized id="gardens-sales-month-empty" />}
              help={<Localized id="gardens-sales-month-empty-help" />}
            />
          ) : (
            <EmptyState
              icon={Coins}
              title={<Localized id="gardens-sales-empty" />}
              help={<Localized id="gardens-sales-empty-help" />}
            />
          )
        }
      />
      {windowed ? <ListWindowBar window={listWindow} /> : null}
    </Card>
  );
}
