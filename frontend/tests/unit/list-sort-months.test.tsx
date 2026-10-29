// The title of a column sorts a list by the backend, and a list in date order has a heading row above
// each month (plan 38 R4). The sort is the backend's over the whole filter (the grid never reorders
// rows); the month rows carry the backend's count and subtotals, never a sum of the page. Urdu and
// English.

import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import type { MonthSubtotal } from '../../src/bindings/MonthSubtotal';
import type { SortInput } from '../../src/bindings/SortInput';
import { LANGUAGES, type Language } from '../../src/i18n';
import { PagedList, type ListColumn } from '../../src/shared/grid';
import { usePagedQuery } from '../../src/shared/hooks';
import { useLanguageStore } from '../../src/shared/i18n';
import { message, visible } from '../helpers/render';

interface Row {
  id: number;
  month: string;
  name: string;
}

const ROWS: Row[] = [
  { id: 1, month: '2026-10', name: 'a' },
  { id: 2, month: '2026-10', name: 'b' },
  { id: 3, month: '2026-09', name: 'c' },
];

/** The backend's subtotals over the whole filter: more rows than the page shows. */
const MONTHS: MonthSubtotal[] = [
  { month: '2026-10', count: 12, figures: [{ key: 'name', is_money: false, value: 480 }] },
  { month: '2026-09', count: 7, figures: [{ key: 'name', is_money: false, value: 35 }] },
];

const COLUMNS: ListColumn<Row>[] = [
  {
    field: 'name',
    type: 'text',
    headerId: 'common-back',
    value: (row) => row.name,
    sortKey: 'name',
  },
];

const calls: (SortInput | null)[] = [];
const opened = vi.fn<(row: Row) => void>();

function List({ withMonths }: { withMonths: boolean }) {
  const query = usePagedQuery({
    screen: 'test.sorted',
    queryKey: ['test-sorted'],
    fetchPage: (_offset, _limit, sort) => {
      calls.push(sort);
      return Promise.resolve({ rows: ROWS, total: ROWS.length });
    },
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total,
  });
  const page = query.data?.pages[0];
  return (
    <PagedList<Row>
      columns={COLUMNS}
      rows={page?.rows}
      getRowId={(row) => String(row.id)}
      labelId="common-back"
      failedId="common-back"
      error={null}
      onRetry={() => undefined}
      pager={query.pager}
      sort={{ value: query.sort, onChange: query.setSort }}
      months={withMonths ? { months: MONTHS, monthOf: (row) => row.month } : undefined}
      onOpen={opened}
      empty={<span />}
    />
  );
}

function mount(withMonths: boolean) {
  return render(
    <AppProviders queryClient={createQueryClient()}>
      <List withMonths={withMonths} />
    </AppProviders>,
  );
}

describe.each(LANGUAGES)('a list with sorting and month rows (%s)', (language: Language) => {
  beforeEach(() => {
    calls.length = 0;
    opened.mockClear();
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('asks the backend for the new order when a title is clicked: up, then down, then its own order', async () => {
    const user = userEvent.setup();
    mount(false);
    // The header is drawn again after each new order, so the title is found again each time.
    const title = () => screen.findByText(message(language, 'common-back'));
    await title();
    await waitFor(() => {
      expect(calls.at(-1)).toBeNull();
    });
    await user.click(await title());
    await waitFor(() => {
      expect(calls.at(-1)).toEqual({ column: 'name', direction: 'asc' });
    });
    await user.click(await title());
    await waitFor(() => {
      expect(calls.at(-1)).toEqual({ column: 'name', direction: 'desc' });
    });
    await user.click(await title());
    await waitFor(() => {
      expect(calls.at(-1)).toBeNull();
    });
  });

  it('puts a heading row above each month with the backend’s count and subtotal, not the page’s', async () => {
    mount(true);
    const october = `${message(language, 'report-month-10')} 2026`;
    const september = `${message(language, 'report-month-9')} 2026`;
    await waitFor(() => {
      expect(document.querySelectorAll('.month-heading')).toHaveLength(2);
    });
    const [first, second] = Array.from(document.querySelectorAll<HTMLElement>('.month-heading'));
    expect(visible(first?.textContent)).toContain(october);
    // 12 entries and 480 are the whole filter's October, though the page has two October rows.
    expect(visible(first?.textContent)).toContain(
      message(language, 'common-month-count', { count: 12 }),
    );
    expect(visible(first?.textContent)).toContain('480');
    expect(visible(second?.textContent)).toContain(september);
    expect(visible(second?.textContent)).toContain('35');
  });

  it('Enter on a row opens it and Enter on a month row opens nothing', async () => {
    const user = userEvent.setup();
    mount(true);
    await waitFor(() => {
      expect(document.querySelectorAll('.month-heading')).toHaveLength(2);
    });
    const cell = document.querySelector<HTMLElement>('.ag-row[row-id="2"] [col-id="name"]');
    if (cell === null) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(opened).toHaveBeenCalledTimes(1);
    });
    expect(opened.mock.calls[0]?.[0].id).toBe(2);
    const heading = document.querySelector<HTMLElement>('.month-heading');
    if (heading === null) throw new Error('no heading');
    await user.click(heading);
    await user.keyboard('{Enter}');
    expect(opened).toHaveBeenCalledTimes(1);
  });

  it('shows no month rows for a list that did not ask for them', async () => {
    mount(false);
    await screen.findByText(message(language, 'common-back'));
    expect(document.querySelector('.month-heading')).toBeNull();
  });
});
