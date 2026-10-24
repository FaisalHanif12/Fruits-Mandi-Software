// The paged lists and the backend agree (plan 17b, PRD FR-19.11, Code Standards B31). Each wrapper
// sends the command name the app registers and the one `{ input }` the backend's own test sent, with
// `offset` and `limit`; the real answer has the rows, the count of the whole filter and the totals;
// and the second page of a list carries the same count and the same totals as the first, because
// both are over the whole filter and never over the rows of one page.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as mastersApi from '../../src/features/masters/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as rulesApi from '../../src/features/rules/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as invoke from '../../src/shared/api/invoke';
import * as urduApi from '../../src/shared/urdu/api';
import * as accountsApi from '../../src/shared/accounts/api';
import * as bikriReal from '../helpers/bikri-real';
import { rule } from '../helpers/rules';
import { same } from '../helpers/shapes';
import * as paged from '../helpers/paged-real';
import * as vouchersReal from '../helpers/vouchers-real';
import stockRequest from '../fixtures/bikri_stock_list_request.json';
import stockOwnRequest from '../fixtures/bikri_stock_list_own_rows_request.json';
import stock from '../fixtures/bikri_stock_list.json';
import stockOwn from '../fixtures/bikri_stock_list_own_rows.json';
import oneLot from '../fixtures/bikri_stock_list_one_lot.json';
import oneLotRequest from '../fixtures/bikri_stock_list_one_lot_request.json';
import oneItem from '../fixtures/masters_list_one_item.json';
import oneItemRequest from '../fixtures/masters_list_one_item_request.json';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

beforeEach(() => {
  vi.resetAllMocks();
});

describe('each paged wrapper sends exactly the request the backend’s own test sent', () => {
  it('bikri_list, bikri_stock_list, pukhta_register', async () => {
    call.mockResolvedValue(bikriReal.LIST_DAY);
    await bikriApi.bikriList(bikriReal.LIST_DAY_REQUEST as never);
    expect(call).toHaveBeenLastCalledWith('bikri_list', { input: bikriReal.LIST_DAY_REQUEST });
    call.mockResolvedValue(stock);
    await bikriApi.bikriStockList(stockRequest as never);
    expect(call).toHaveBeenLastCalledWith('bikri_stock_list', { input: stockRequest });
    await bikriApi.bikriStockList(stockOwnRequest as never);
    expect(call).toHaveBeenLastCalledWith('bikri_stock_list', { input: stockOwnRequest });
    call.mockResolvedValue(bikriReal.REGISTER_DAY);
    await pukhtaApi.pukhtaRegister(bikriReal.REGISTER_DAY_REQUEST as never);
    expect(call).toHaveBeenLastCalledWith('pukhta_register', {
      input: bikriReal.REGISTER_DAY_REQUEST,
    });
  });

  it('vouchers_list, two pages', async () => {
    call.mockResolvedValue(vouchersReal.LIST_ALL);
    await vouchersApi.vouchersList(vouchersReal.REQUEST_LIST_ALL);
    expect(call).toHaveBeenLastCalledWith('vouchers_list', {
      input: vouchersReal.REQUEST_LIST_ALL,
    });
    await vouchersApi.vouchersList(paged.REQUEST_VOUCHERS_ALL_PAGE_2);
    expect(call).toHaveBeenLastCalledWith('vouchers_list', {
      input: paged.REQUEST_VOUCHERS_ALL_PAGE_2,
    });
  });

  it('masters_list, rules_list and translit_pairs_list, two pages each', async () => {
    call.mockResolvedValue(paged.MASTERS_ITEMS);
    await mastersApi.mastersList(
      paged.REQUEST_MASTERS_ITEMS_PAGE_2.kind,
      paged.REQUEST_MASTERS_ITEMS_PAGE_2.include_inactive,
      paged.REQUEST_MASTERS_ITEMS_PAGE_2.offset,
      paged.REQUEST_MASTERS_ITEMS_PAGE_2.limit,
    );
    expect(call).toHaveBeenLastCalledWith('masters_list', {
      input: paged.REQUEST_MASTERS_ITEMS_PAGE_2,
    });
    call.mockResolvedValue(paged.RULES_ALL);
    await rulesApi.rulesList(
      paged.REQUEST_RULES_ALL_PAGE_2.filter,
      paged.REQUEST_RULES_ALL_PAGE_2.offset,
      paged.REQUEST_RULES_ALL_PAGE_2.limit,
    );
    expect(call).toHaveBeenLastCalledWith('rules_list', { input: paged.REQUEST_RULES_ALL_PAGE_2 });
    call.mockResolvedValue(paged.PAIRS);
    await urduApi.translitPairsList(
      paged.REQUEST_PAIRS_PAGE_2.offset,
      paged.REQUEST_PAIRS_PAGE_2.limit,
    );
    expect(call).toHaveBeenLastCalledWith('translit_pairs_list', {
      input: paged.REQUEST_PAIRS_PAGE_2,
    });
  });

  it('accounts_list sends the page it is asked for, and no longer asks for every account', async () => {
    call.mockResolvedValue({ rows: [], count: 0, total_dr_paisa: 0, total_cr_paisa: 0 });
    const filter = { account_type: null, text: '', include_inactive: false };
    await accountsApi.accountsList(filter, 100, 100);
    expect(call).toHaveBeenLastCalledWith('accounts_list', {
      input: { filter, offset: 100, limit: 100 },
    });
  });
});

describe('the real answers are pages: rows, the count and the totals of the whole filter', () => {
  it('the list of بکری, the register and the stock list have the keys of a page', () => {
    expect(Object.keys(bikriReal.LIST_DAY).sort()).toEqual(['rows', 'total_count', 'totals']);
    expect(Object.keys(bikriReal.REGISTER_DAY).sort()).toEqual(['rows', 'total_count', 'totals']);
    for (const answer of [stock, stockOwn]) {
      expect(Object.keys(answer)).toEqual(
        expect.arrayContaining(['rows', 'own_rows', 'total_count']),
      );
    }
    // The own rows come after the lots and are counted with them (backend, plan 17b T1).
    expect(stockOwn.total_count).toBe(stockOwn.rows.length + stockOwn.own_rows.length);
  });

  it('the register has sums only when a side is chosen; with none they are all null', () => {
    expect(bikriReal.REGISTER_DAY_REQUEST).not.toHaveProperty('side');
    const none = bikriReal.REGISTER_DAY.totals;
    expect([none.qty, none.amount, none.commission, none.open]).toEqual([null, null, null, null]);
    expect(bikriReal.REGISTER_DAY_BUYERS_REQUEST).toMatchObject({ side: 'buyer' });
    const given = bikriReal.REGISTER_DAY_BUYERS.totals;
    expect(given.qty).not.toBeNull();
    expect(given.amount).not.toBeNull();
    expect(given.commission).not.toBeNull();
    expect(given.open).not.toBeNull();
  });

  it('the second page has other rows and the same count and totals as the first', () => {
    for (const [first, second] of [
      [vouchersReal.LIST_ALL, paged.VOUCHERS_ALL_PAGE_2],
      [paged.MASTERS_ITEMS, paged.MASTERS_ITEMS_PAGE_2],
      [paged.RULES_ALL, paged.RULES_ALL_PAGE_2],
      [paged.PAIRS, paged.PAIRS_PAGE_2],
    ] as const) {
      expect(second.total_count).toBeGreaterThanOrEqual(first.rows.length);
      expect(second.rows.length).toBeGreaterThan(0);
    }
    expect(paged.VOUCHERS_ALL_PAGE_2.total_count).toBe(vouchersReal.LIST_ALL.total_count);
    expect(paged.VOUCHERS_ALL_PAGE_2.totals).toEqual(vouchersReal.LIST_ALL.totals);
    expect(paged.RULES_ALL_PAGE_2.total_count).toBe(paged.RULES_ALL.total_count);
    expect(paged.RULES_ALL_PAGE_2.totals).toEqual(paged.RULES_ALL.totals);
    expect(paged.PAIRS_PAGE_2.total_count).toBe(paged.PAIRS.total_count);
    expect(paged.PAIRS_PAGE_2.totals).toEqual(paged.PAIRS.totals);
  });

  it('every request carries an explicit offset and limit', () => {
    for (const request of [
      bikriReal.LIST_DAY_REQUEST,
      bikriReal.REGISTER_DAY_REQUEST,
      stockRequest,
      stockOwnRequest,
      vouchersReal.REQUEST_LIST_ALL,
      vouchersReal.REQUEST_LIST_PAYMENTS,
      paged.REQUEST_VOUCHERS_ALL_PAGE_2,
      paged.REQUEST_MASTERS_ITEMS,
      paged.REQUEST_MASTERS_ITEMS_PAGE_2,
      paged.REQUEST_RULES_ALL,
      paged.REQUEST_RULES_ALL_PAGE_2,
      paged.REQUEST_PAIRS,
      paged.REQUEST_PAIRS_PAGE_2,
    ]) {
      expect(request).toHaveProperty('offset');
      expect(request).toHaveProperty('limit');
    }
  });
});

describe('the rows of the real pages have the shape the screens use', () => {
  it('a rule row, and the masters the wrapper turns into records', async () => {
    const [first] = paged.RULES_ALL.rows;
    expect(same(first, rule(1))).toBeNull();
    call.mockResolvedValue(paged.MASTERS_ITEMS);
    const page = await mastersApi.mastersList('item', false, 0, 0);
    expect(page.total_count).toBe(paged.MASTERS_ITEMS.total_count);
    expect(page.totals).toEqual(paged.MASTERS_ITEMS.totals);
    expect(page.rows).toHaveLength(paged.MASTERS_ITEMS.rows.length);
    expect(page.rows.every((row) => row.kind === 'item' && row.names !== null)).toBe(true);
  });
});

describe('one record by its id: the lot of a later sale and the master of a form', () => {
  it('the stock list asks for the one lot with lot_id, and the answer is that lot alone', async () => {
    call.mockResolvedValue(oneLot);
    await bikriApi.bikriStockList({
      date: null,
      from: null,
      to: null,
      seller_id: null,
      item_id: null,
      include_written_off: false,
      lot_id: oneLotRequest.lot_id,
      offset: oneLotRequest.offset,
      limit: oneLotRequest.limit,
    });
    // The screen sends every key; the backend's own request leaves the defaults out.
    expect(call.mock.calls.at(-1)?.[0]).toBe('bikri_stock_list');
    expect(call.mock.calls.at(-1)?.[1]).toMatchObject({ input: oneLotRequest });
    expect(oneLot.rows.map((row) => row.lot_id)).toEqual([oneLotRequest.lot_id]);
    expect(oneLot.total_count).toBe(1);
    expect(oneLot.own_rows).toEqual([]);
  });

  it('the masters list asks for the one master with id, also an inactive one, and nothing else', async () => {
    call.mockResolvedValue(oneItem);
    const record = await mastersApi.mastersGetOne(oneItemRequest.kind as 'item', oneItemRequest.id);
    expect(call).toHaveBeenLastCalledWith('masters_list', { input: oneItemRequest });
    expect(record).toMatchObject({ kind: 'item', id: oneItemRequest.id, is_active: false });
    // A master that is not there comes back as null, not as an error.
    call.mockResolvedValue({ rows: [], total_count: 0, totals: { active_count: 0 } });
    expect(await mastersApi.mastersGetOne('item', 999)).toBeNull();
  });
});
