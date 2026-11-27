// The list of a garden's sales and where their money is (PRD FR-12.53, FR-19.11): one page of rows at
// a time with the backend's totals over the whole filter, never a sum of the rows on screen. Enter on
// a sale of our shop opens its بکری (FR-19.4). Under the list stands the row that has focus: what the
// garden was credited for it, with how it was worked out (FR-7.43), and the total of the lot charges
// taken off the garden's money. Used by the sale money section (a month or every month) and by the
// details of a goods arrived entry (its own sales). Four states: loading, an error with a retry,
// nothing yet, and the rows. Every figure is the backend's.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useMemo, useState, type ReactNode } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { NumberRuns, PagedList, useListContext, type ListContext } from '../../../shared/grid';
import { isZeroPaisa } from '../../../shared/format';
import { CalcInfo, Num } from '../../../shared/ui';
import { useSaleMoney, type SaleMoneyFilter } from '../hooks';
import { SALE_MONEY_COLUMNS, saleText, type SaleMoneyListRow } from '../sale-money-columns';
import type { SaleMoneyTotals } from '../types';

interface SaleMoneyListProps {
  gardenId: number;
  filter: SaleMoneyFilter;
  /** The name under which this list remembers its page. */
  screen: string;
  /** The Fluent ID of the list's name, read by screen readers, and its values. */
  labelId: string;
  labelVars?: Record<string, string> | undefined;
  /** Shown when the filter has no rows. */
  empty: ReactNode;
  /** Page Up and Page Down move the month window instead of the page. */
  onPageKey?: ((direction: 'previous' | 'next') => void) | undefined;
  /** False while the filter is not known yet (the day is being read). */
  enabled?: boolean;
}

export function SaleMoneyList({
  gardenId,
  filter,
  screen,
  labelId,
  labelVars,
  empty,
  onPageKey,
  enabled = true,
}: SaleMoneyListProps) {
  const navigate = useNavigate();
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const list = useSaleMoney(gardenId, filter, screen, enabled);
  const first = list.data?.pages[0];
  const { page } = list.pager;
  const rows = useMemo<SaleMoneyListRow[] | undefined>(
    () =>
      list.data?.pages
        .flatMap((one) => one.rows)
        // The backend's rows have no id of their own (a lot can have several lines): their place in
        // the page is their identity while the page is on screen.
        .map((row, index) => ({ ...row, key: `${String(page)}-${String(index)}` })),
    [list.data, page],
  );
  const focused = rows?.find((row) => row.key === focusKey) ?? rows?.[0] ?? null;
  const totals: SaleMoneyTotals | undefined = first?.totals;

  const open = (row: SaleMoneyListRow) => {
    if (row.bikri_id === null) {
      setFocusKey(row.key);
      return;
    }
    void navigate({ to: '/bikri/book', search: { id: row.bikri_id } });
  };

  return (
    <div className="flex min-w-0 flex-col gap-3">
      <PagedList<SaleMoneyListRow>
        columns={SALE_MONEY_COLUMNS}
        rows={rows}
        getRowId={(row) => row.key}
        labelId={labelId}
        labelVars={labelVars}
        failedId="gardens-sales-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        onPageKey={onPageKey}
        empty={empty}
        onOpen={open}
        onFocusRow={setFocusKey}
        focusRowId={focusKey}
        totals={
          totals === undefined
            ? undefined
            : (context: ListContext) => ({
                labelField: 'sale',
                values: {
                  sale: context.text('glossary-meezan'),
                  buyer: context.text('gardens-sales-goods-total', { qty: String(totals.qty) }),
                  value: context.money(totals.value_paisa),
                  bill: context.money(totals.buyer_total_paisa),
                  paid: context.money(totals.paid_paisa),
                  collected: context.money(totals.collected_paisa),
                  still_due: context.money(totals.still_due_paisa),
                  credit: context.money(totals.garden_credit_paisa),
                },
              })
        }
      />
      {focused === null ? null : <FocusedSale row={focused} />}
      {totals === undefined || isZeroPaisa(totals.lot_charges_paisa) ? null : (
        <p className="flex flex-wrap items-baseline gap-2 type-body text-ink-muted">
          <Localized id="gardens-sales-charges-total" />
          <span className="text-ink">
            <Num paisa={totals.lot_charges_paisa} />
          </span>
        </p>
      )}
    </div>
  );
}

/** The row that has focus, in full: what the garden was credited for it, and how (FR-7.43). */
function FocusedSale({ row }: { row: SaleMoneyListRow }) {
  return (
    <div
      className="flex flex-wrap items-baseline gap-x-3 gap-y-1 rounded-2 bg-surface-sunken px-3 py-2 type-body text-ink"
      data-focused-sale=""
    >
      <FocusedSaleName row={row} />
      <span className="inline-flex items-center gap-1">
        <span className="text-ink-muted">
          <Localized id="gardens-sales-col-credit" />
        </span>
        <Num paisa={row.garden_credit_paisa} />
        <CalcInfo working={row.working} />
      </span>
      {row.value_paisa === null ? null : (
        <span className="inline-flex items-center gap-1">
          <span className="text-ink-muted">
            <Localized id="gardens-sales-col-value" />
          </span>
          <Num paisa={row.value_paisa} />
        </span>
      )}
      {row.buyer_total_paisa === null ? null : (
        <span className="inline-flex items-center gap-1">
          <span className="text-ink-muted">
            <Localized id="gardens-sales-col-bill" />
          </span>
          <Num paisa={row.buyer_total_paisa} />
        </span>
      )}
    </div>
  );
}

/** The words of the row's first cell, in the app language, each number kept in order (F25). */
function FocusedSaleName({ row }: { row: SaleMoneyListRow }) {
  const context = useListContext();
  return (
    <span className="type-label">
      <NumberRuns text={saleText(row, context)} />
    </span>
  );
}
