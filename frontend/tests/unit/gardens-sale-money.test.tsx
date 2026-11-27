// The garden page of plan 99 (PRD FR-12.53, FR-19.11), in the Urdu and the English app, against a fake
// backend: the goods not sold yet and the goods sold are two lists, each with its own columns and the
// backend's totals; "where the sale money is" shows one month at a time (or every month), with a
// search, each sale's bill, paid on the spot, collected since, still due and the garden's credit, the
// lot charges, the backend's totals, and how the credit was worked out; Enter on a sale of our shop
// opens its بکری; and the four states. The screen adds nothing up (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ArrivalRow } from '../../src/bindings/ArrivalRow';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import * as windowsApi from '../../src/shared/windows/api';
import { listArrivals, manyArrivals } from '../helpers/arrivals-many';
import { named } from '../helpers/bikri-screen';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { paisaForTest } from '../helpers/paisa';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { SALE_MONEY_ROWS, saleMoneyAnswer } from '../helpers/sale-money';
import { fakeItemPickers } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/windows/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeItemPickers();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
    first_date: '2026-08-01',
    last_date: '2026-10-10',
  });
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('the garden page of plan 99 (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeGardens();
    vi.mocked(gardensApi.gardensSaleMoney).mockImplementation((input) =>
      Promise.resolve(saleMoneyAnswer(SALE_MONEY_ROWS, input)),
    );
  });

  let router: Awaited<ReturnType<typeof renderApp>>['router'];
  const open = async () => {
    router = (await renderApp('/gardens/1')).router;
    await screen.findByText(said(message(language, 'gardens-detail-title', { code: '0080' })));
    return userEvent.setup();
  };
  const salesList = () =>
    screen.findByRole('group', { name: named(message(language, 'gardens-sales-label')) });
  const cell = (list: HTMLElement, rowId: string, field: string) =>
    visible(list.querySelector(`.ag-row[row-id="${rowId}"] [col-id="${field}"]`)?.textContent);
  const pinned = (list: HTMLElement, field: string) =>
    visible(list.querySelector(`.ag-row-pinned [col-id="${field}"]`)?.textContent);
  const lastSaleAsk = () =>
    vi
      .mocked(gardensApi.gardensSaleMoney)
      .mock.calls.map(([input]) => input)
      .filter((input) => input.arrival_id === null)
      .at(-1);

  it('shows where the money of this month’s sales is, with the backend’s totals and the lot charges', async () => {
    await open();
    const list = await salesList();
    await waitFor(() => {
      expect(cell(list, '1-0', 'credit')).toBe('18,800');
    });
    // This month, from its first day to its last, with nothing searched.
    expect(lastSaleAsk()).toMatchObject({
      garden_id: 1,
      from: '01-10-2026',
      to: '31-10-2026',
      search: null,
      offset: 0,
    });
    expect(windowsApi.windowsBounds).toHaveBeenCalledWith({
      source: { kind: 'garden_sales', garden_id: 1 },
    });
    expect(cell(list, '1-0', 'bill')).toBe('21,360');
    expect(cell(list, '1-0', 'paid')).toBe('0');
    expect(cell(list, '1-0', 'still_due')).toBe('21,360');
    expect(cell(list, '1-0', 'buyer')).toContain(language === 'ur' ? 'مانی' : 'Mani');
    // The goods as R5 writes them, and the value in its own column.
    expect(cell(list, '1-0', 'buyer')).toContain('(20 . 1,000)');
    expect(cell(list, '1-0', 'value')).toBe('20,000');
    expect(pinned(list, 'value')).toBe('30,000');
    // The lot's charges are a row of their own, taken off the garden's money; a field it has not is
    // empty.
    expect(cell(list, '1-1', 'credit')).toBe('-1,250');
    expect(cell(list, '1-1', 'bill')).toBe('');
    expect(cell(list, '1-2', 'buyer')).toContain('Haji Shop');
    // The totals are the backend's over the whole filter.
    expect(pinned(list, 'bill')).toBe('21,360');
    expect(pinned(list, 'credit')).toBe('27,550');
    expect(visible(document.body.textContent)).toContain(
      message(language, 'gardens-sales-charges-total'),
    );
  });

  it('the row with focus says how the garden was credited, with the working the backend sent', async () => {
    const user = await open();
    const list = await salesList();
    await waitFor(() => {
      expect(cell(list, '1-0', 'credit')).toBe('18,800');
    });
    const line = () => {
      const found = document.querySelector<HTMLElement>('[data-focused-sale]');
      if (found === null) throw new Error('no focused sale');
      return found;
    };
    expect(visible(line().textContent)).toContain('18,800');
    await user.click(within(line()).getByRole('button'));
    const working = await screen.findByRole('dialog');
    for (const figure of ['20,000', '1,200', '18,800']) {
      expect(visible(working.textContent)).toContain(figure);
    }
  });

  it('every month on the tick, and a search, ask the backend again', async () => {
    const user = await open();
    await salesList();
    await user.click(
      screen.getByRole('checkbox', {
        name: named(message(language, 'gardens-sales-every-month')),
      }),
    );
    await waitFor(() => {
      expect(lastSaleAsk()).toMatchObject({ from: null, to: null });
    });
    await user.type(
      screen.getByRole('textbox', {
        name: named(message(language, 'gardens-sales-search-label')),
      }),
      'mani',
    );
    await waitFor(() => {
      expect(lastSaleAsk()?.search).toBe('mani');
    });
  });

  it('Enter on a sale of our shop opens its بکری', async () => {
    const user = await open();
    const list = await salesList();
    await waitFor(() => {
      expect(cell(list, '1-0', 'credit')).toBe('18,800');
    });
    const first = list.querySelector<HTMLElement>('.ag-row[row-id="1-0"] [col-id="sale"]');
    if (first === null) throw new Error('no sale cell');
    await user.click(first);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41 });
  });

  it('a month with no sale says what to do; an error says so and the retry asks again', async () => {
    vi.mocked(gardensApi.gardensSaleMoney).mockImplementation((input) =>
      Promise.resolve(saleMoneyAnswer([], input)),
    );
    await open();
    expect(
      await screen.findByText(said(message(language, 'gardens-sales-month-empty'))),
    ).toBeInTheDocument();
    leaveAppWindow();
    vi.mocked(gardensApi.gardensSaleMoney).mockRejectedValue(
      fail({ code: 'ERR_INTERNAL', field: null, values: {} }),
    );
    resetClientState(language);
    signedIn(sessionUser({ language }));
    await open();
    expect(
      await screen.findByText(said(message(language, 'gardens-sales-failed'))),
    ).toBeInTheDocument();
  });

  it('the goods not sold yet and the goods sold are two lists, each with its own columns and totals', async () => {
    const base = manyArrivals(6);
    const [, , third] = base;
    if (third === undefined) throw new Error('no third entry');
    const sold: ArrivalRow = {
      ...third,
      id: 200,
      number: 200,
      qty: 50,
      qty_sold: 45,
      qty_our_sold: 30,
      qty_other_sold: 15,
      qty_written_off: 5,
      qty_left: 0,
      qty_our_left: 0,
      qty_other_left: 0,
      sale_value_paisa: paisaForTest(4_500_000),
      last_sale_date: '2026-10-09',
    };
    vi.mocked(gardensApi.gardensArrivalsList).mockImplementation((input) =>
      Promise.resolve(listArrivals([...base, sold], input)),
    );
    const user = await open();
    const soldList = await screen.findByRole('group', {
      name: named(message(language, 'gardens-sold-label')),
    });
    await waitFor(() => {
      expect(cell(soldList, '200', 'qty_our_sold')).toBe('30');
    });
    expect(cell(soldList, '200', 'qty_other_sold')).toBe('15');
    expect(cell(soldList, '200', 'qty_written_off')).toBe('5');
    expect(cell(soldList, '200', 'sale_value')).toBe('45,000');
    expect(cell(soldList, '200', 'last_sale_date')).toBe('09-10-2026');
    expect(pinned(soldList, 'qty_our_sold')).toBe('30');
    const unsold = screen.getByRole('group', {
      name: named(message(language, 'gardens-arrival-label')),
    });
    expect(unsold.querySelector('.ag-row[row-id="200"]')).toBeNull();
    // Nothing of the sold list is open until a row is chosen; then its sale lines open under it.
    expect(screen.getAllByRole('complementary')).toHaveLength(1);
    const number = soldList.querySelector<HTMLElement>('.ag-row[row-id="200"] [col-id="number"]');
    if (number === null) throw new Error('no sold row');
    await user.click(number);
    await waitFor(() => {
      expect(screen.getAllByRole('complementary')).toHaveLength(2);
    });
    await waitFor(() => {
      expect(gardensApi.gardensSaleMoney).toHaveBeenCalledWith(
        expect.objectContaining({ arrival_id: 200, from: null, to: null }),
      );
    });
  });
});
