// Plan 99 against the backend's REAL answers (PRD FR-12.53, FR-17.3; tests/fixtures/gardens_sale_money_*,
// gardens_arrivals_list_sold_out and partners_open_ventures_*, copied, never edited): the test doubles
// have the real shape; the wrappers send the commands the way the backend's tests sent them; every
// working of a sale has words in both languages with its real figures; and the garden page shows the
// real month, figure by figure. The screen works out nothing (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES } from '../../src/i18n';
import * as invoke from '../../src/shared/api/invoke';
import * as urduApi from '../../src/shared/urdu/api';
import * as windowsApi from '../../src/shared/windows/api';
import { listArrivals, manyArrivals } from '../helpers/arrivals-many';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { openVenturesAnswer } from '../helpers/partner-ventures';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { CHARGES_ROW, MANI_ROW, OTHER_SHOP_ROW, saleMoneyAnswer } from '../helpers/sale-money';
import * as real from '../helpers/sale-money-real';
import { same } from '../helpers/shapes';
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

const rowOfKind = (kind: string) => {
  const row = real.SALE_MONEY_MONTH.rows.find((one) => one.kind === kind);
  if (row === undefined) throw new Error(`the real month has no ${kind} row`);
  return row;
};

describe('the test doubles of plan 99 have the real shapes', () => {
  it('the sale money: the answer, a row of each kind, and the lot charges', () => {
    const canned = saleMoneyAnswer([MANI_ROW], real.SALE_MONEY_MONTH_REQUEST);
    expect(same(real.SALE_MONEY_MONTH, canned)).toBeNull();
    for (const double of [MANI_ROW, CHARGES_ROW, OTHER_SHOP_ROW]) {
      expect(same(rowOfKind(double.kind), double)).toBeNull();
    }
    // A sale of the arrival and a second page have the same shape.
    for (const answer of [real.SALE_MONEY_ARRIVAL, real.SALE_MONEY_SECOND_PAGE]) {
      expect(same(answer, canned)).toBeNull();
    }
  });

  it('the goods sold and a partner’s open ventures', () => {
    // The backend's request leaves its empty defaults out; the double takes them in full.
    const asked = {
      ...real.ARRIVALS_SOLD_OUT_REQUEST,
      state: null,
      search: null,
      sort: null,
      offset: 0,
      limit: 0,
    };
    expect(same(real.ARRIVALS_SOLD_OUT, listArrivals(manyArrivals(1), asked))).toBeNull();
    const month = openVenturesAnswer(
      real.OPEN_VENTURES_MONTH.rows,
      real.OPEN_VENTURES_MONTH_REQUEST,
    );
    expect(same(real.OPEN_VENTURES_MONTH, month)).toBeNull();
    expect(real.OPEN_VENTURES_LATEST.rows).toHaveLength(1);
    expect(real.BOUNDS_VENTURES_REQUEST.source).toEqual({
      kind: 'partner_ventures',
      partner_id: 1,
    });
  });

  it.each(LANGUAGES)(
    '%s: every working of a sale has its words with the real figures',
    (language) => {
      const rows = [
        ...real.SALE_MONEY_MONTH.rows,
        ...real.SALE_MONEY_ARRIVAL.rows,
        ...real.SALE_MONEY_SECOND_PAGE.rows,
      ];
      for (const row of rows) {
        const vars = Object.fromEntries(row.working.args.map((one) => [one.name, '7']));
        const text = message(language, row.working.message_id, vars);
        expect(text, row.working.message_id).not.toMatch(/[{}$]/);
        expect(text, row.working.message_id).toContain('7');
      }
    },
  );
});

describe('the wrappers send the commands as the backend’s tests did', () => {
  it('gardens_sale_money and partners_open_ventures take one input', async () => {
    const gardens = await vi.importActual<typeof gardensApi>('../../src/features/gardens/api');
    const partners = await vi.importActual<typeof partnersApi>('../../src/features/partners/api');
    const call = vi.spyOn(invoke, 'callCommand').mockResolvedValue(undefined);
    await gardens.gardensSaleMoney(real.SALE_MONEY_MONTH_REQUEST);
    await partners.partnersOpenVentures(real.OPEN_VENTURES_MONTH_REQUEST);
    expect(call.mock.calls).toEqual([
      ['gardens_sale_money', { input: real.SALE_MONEY_MONTH_REQUEST }],
      ['partners_open_ventures', { input: real.OPEN_VENTURES_MONTH_REQUEST }],
    ]);
    call.mockRestore();
  });
});

describe.each(LANGUAGES)('the garden page with the real sale money (%s)', (language) => {
  beforeEach(() => {
    vi.resetAllMocks();
    healthyBackend();
    fakePartners(THREE_PARTNERS);
    fakeItemPickers();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeGardens();
    vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
    vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
    vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
      first_date: '2026-10-01',
      last_date: '2026-10-10',
    });
    vi.mocked(gardensApi.gardensSaleMoney).mockResolvedValue(real.SALE_MONEY_MONTH);
  });

  afterEach(() => {
    leaveAppWindow();
  });

  it('shows each real figure and the backend’s totals, and opens a working with the real numbers', async () => {
    const user = userEvent.setup();
    await renderApp('/gardens/1');
    const list = await screen.findByRole('group', {
      name: named(message(language, 'gardens-sales-label')),
    });
    // The month window asks its first day the way the backend's own test asked it.
    expect(windowsApi.windowsBounds).toHaveBeenCalledWith(real.BOUNDS_GARDEN_SALES_REQUEST);
    const cell = (rowId: string, field: string) =>
      visible(list.querySelector(`.ag-row[row-id="${rowId}"] [col-id="${field}"]`)?.textContent);
    await waitFor(() => {
      expect(cell('1-0', 'credit')).toBe('18,800');
    });
    expect(cell('1-0', 'value')).toBe('20,000');
    expect(cell('1-0', 'bill')).toBe('21,360');
    expect(cell('1-0', 'still_due')).toBe('21,360');
    expect(cell('1-1', 'paid')).toBe('5,000');
    expect(cell('1-1', 'still_due')).toBe('5,680');
    expect(cell('1-2', 'credit')).toBe('-220');
    expect(cell('1-4', 'buyer')).toContain('Haji Traders');
    const pinned = (field: string) =>
      visible(list.querySelector(`.ag-row-pinned [col-id="${field}"]`)?.textContent);
    expect(pinned('value')).toBe('63,000');
    expect(pinned('bill')).toBe('51,280');
    expect(pinned('paid')).toBe('5,000');
    expect(pinned('still_due')).toBe('46,280');
    expect(pinned('credit')).toBe('59,900');
    const line = document.querySelector<HTMLElement>('[data-focused-sale]');
    if (line === null) throw new Error('no focused sale');
    await user.click(within(line).getByRole('button'));
    const working = await screen.findByRole('dialog');
    for (const figure of ['20,000', '1,200', '18,800']) {
      expect(visible(working.textContent)).toContain(figure);
    }
  });
});
