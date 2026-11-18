// The commission page with a goods filter (plan 78 D4, PRD FR-9.3, Umar 17-11-2026): all goods, consigned goods,
// trip goods (اپنا مال of a market trip) or garden goods. It combines with the period and the party, travels in the
// address, is sent to the month, the day and the print, and hides the partner columns, because partner shares are
// kept per posting and not per lot. Both languages, against a fake backend. The screen adds nothing up: every
// figure is the backend's (the garden goods of E2E-40: 3,600 + 3,600 + 800 = 8,000).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AnalyticsGoods } from '../../src/bindings/AnalyticsGoods';
import type { CommissionDay } from '../../src/bindings/CommissionDay';
import type { CommissionDayRow } from '../../src/bindings/CommissionDayRow';
import type { CommissionMonth } from '../../src/bindings/CommissionMonth';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { printData } from '../helpers/collection-sheet';
import { fakeDashboard } from '../helpers/dashboard';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const rupees = (value: number) => paisaForTest(value * 100);

const PARTNERS = [
  { id: 1, name_ur: 'الف', name_en: 'Partner A' },
  { id: 2, name_ur: 'ب', name_en: 'Partner B' },
  { id: 3, name_ur: 'ج', name_en: 'Partner C' },
];

/** The whole shop on 10-10-26 (plan 76): every kind of goods, with the partners' shares. */
const SHOP_ROW: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(2_484),
  buyer_comm_paisa: rupees(2_484),
  laga_paisa: rupees(480),
  total_comm_paisa: rupees(5_448),
  shared_income_paisa: rupees(5_488),
  partner_shares: [
    { partner_id: 1, share_paisa: rupees(2_196) },
    { partner_id: 2, share_paisa: rupees(1_646) },
    { partner_id: 3, share_paisa: rupees(1_646) },
  ],
  sale_paisa: null,
  goods_comm_paisa: null,
};

/** The commission earned on garden lots only (E2E-40): 3,600 + 3,600 + 800 = 8,000, and no partner shares. */
const GARDEN_ROW: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(3_600),
  buyer_comm_paisa: rupees(3_600),
  laga_paisa: rupees(800),
  total_comm_paisa: rupees(8_000),
  shared_income_paisa: rupees(8_000),
  partner_shares: null,
  sale_paisa: null,
  goods_comm_paisa: null,
};

/** The month the backend answers for the goods asked for: the shop for all goods, the garden's row for the others. */
function month(goods: AnalyticsGoods): CommissionMonth {
  const row = goods === 'all' ? SHOP_ROW : GARDEN_ROW;
  return {
    month: '2026-10',
    year: 2026,
    period: { kind: 'month', from: '2026-10-01', to: '2026-10-31' },
    party: null,
    goods,
    first_month: '2026-09',
    current_month: '2026-10',
    // Under a goods filter the backend sends no partner columns (shares are kept per posting, not per lot).
    partners: goods === 'all' ? PARTNERS : [],
    rows: [row],
    total_count: 1,
    offset: 0,
    limit: 100,
    totals: { ...row, date: '' },
    partners_locked: false,
  };
}

function dayOf(goods: AnalyticsGoods): CommissionDay {
  const row = goods === 'all' ? SHOP_ROW : GARDEN_ROW;
  return {
    date: '2026-10-10',
    party: null,
    goods,
    partners: goods === 'all' ? PARTNERS : [],
    entries: [
      {
        entry_id: 5,
        posting_type: 'bikri_first',
        source_type: 'bikri',
        source_id: 41,
        bikri_id: 41,
        bikri_number: 1,
        sellers: [],
        buyers: [],
        figures: row,
      },
    ],
    total_count: 1,
    totals: row,
    partners_locked: false,
  };
}

const goodsOf = (value: unknown): AnalyticsGoods =>
  value === 'consigned' || value === 'trip' || value === 'garden' ? value : 'all';

const lastRead = () => vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.at(-1)?.[0];

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardCommissionMonth).mockImplementation((input) =>
    Promise.resolve(month(goodsOf(input.goods))),
  );
  vi.mocked(dashboardApi.dashboardCommissionDay).mockImplementation((input) =>
    Promise.resolve(dayOf(goodsOf(input.goods))),
  );
  vi.mocked(dashboardApi.dashboardCommissionPrintData).mockImplementation((input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4')),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the commission page with a goods filter (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('commission-title'),
    });
  const goodsBox = () => screen.getByRole('combobox', { name: m('analytics-filter-goods') });
  const treeText = async () => visible((await screen.findByRole('treegrid')).textContent);
  const headerTexts = async () =>
    within(await screen.findByRole('treegrid'))
      .getAllByRole('columnheader')
      .map((header) => visible(header.textContent));
  const partnerName = (index: number) => {
    const partner = PARTNERS[index];
    if (partner === undefined) throw new Error('no partner');
    return language === 'ur' ? partner.name_ur : partner.name_en;
  };

  it('R1: it opens on all goods, asks for no goods at all and keeps the partner columns, as plan 77 had it', async () => {
    await renderApp('/commission');
    await heading();
    const text = await treeText();
    expect(text).toContain('5,448');
    expect(text).toContain('2,196');
    expect(lastRead()).toEqual({ month: null });
    expect(visible(goodsBox().textContent)).toBe(m('analytics-goods-all'));
  });

  it('R2: a choice in the box asks the backend for those goods, shows its figures, and hides the partner columns', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/commission');
    await heading();
    await screen.findByRole('treegrid');
    await user.click(goodsBox());
    await user.click(await screen.findByRole('option', { name: m('analytics-goods-garden') }));
    await waitFor(() => {
      expect(lastRead()).toEqual({ month: null, goods: 'garden' });
    });
    await waitFor(async () => {
      expect(await treeText()).toContain('8,000');
    });
    const text = await treeText();
    expect(text).toContain('3,600');
    expect(text).toContain('800');
    // Partner shares are per posting, not per lot: no partner column and no share.
    expect(await headerTexts()).not.toEqual(expect.arrayContaining([partnerName(0)]));
    expect(text).not.toContain('2,196');
    expect(router.state.location.search).toMatchObject({ goods: 'garden' });
    expect(visible(goodsBox().textContent)).toBe(m('analytics-goods-garden'));
  });

  it.each(['consigned', 'trip', 'garden'] as const)(
    'R3: the address %s opens the page on those goods, and the box says so',
    async (goods) => {
      await renderApp(`/commission?goods=${goods}`);
      await heading();
      await screen.findByRole('treegrid');
      expect(lastRead()).toEqual({ month: null, goods });
      expect(visible(goodsBox().textContent)).toBe(m(`analytics-goods-${goods}`));
    },
  );

  it('R4: a word the page does not know is left out: it is all goods', async () => {
    await renderApp('/commission?goods=everything');
    await heading();
    await screen.findByRole('treegrid');
    expect(lastRead()).toEqual({ month: null });
    expect(visible(goodsBox().textContent)).toBe(m('analytics-goods-all'));
  });

  it('R5: back to all goods leaves the address, asks the plain month and brings the partner columns back', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp('/commission?goods=trip');
    await heading();
    await screen.findByRole('treegrid');
    await user.click(goodsBox());
    await user.click(await screen.findByRole('option', { name: m('analytics-goods-all') }));
    await waitFor(() => {
      expect(lastRead()).toEqual({ month: null });
    });
    await waitFor(async () => {
      expect(await treeText()).toContain('2,196');
    });
    expect(router.state.location.search).not.toHaveProperty('goods');
  });

  it('R6: it combines with the period: a year of garden goods asks for both', async () => {
    await renderApp('/commission?period=year&year=2026&goods=garden');
    await heading();
    await screen.findByRole('treegrid');
    expect(lastRead()).toMatchObject({
      period: { kind: 'year', year: 2026 },
      goods: 'garden',
    });
  });

  it('R7: an open day asks for the postings of the same goods', async () => {
    const user = userEvent.setup();
    await renderApp('/commission?goods=garden');
    await heading();
    const tree = await screen.findByRole('treegrid');
    const day = within(tree).getAllByRole('row')[1];
    if (day === undefined) throw new Error('no day row');
    await user.click(day);
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionDay).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-10',
        goods: 'garden',
      });
    });
    await waitFor(() => {
      expect(document.querySelector('[data-commission-day="2026-10-10"]')).not.toBeNull();
    });
    const entry = document.querySelector('[data-commission-day="2026-10-10"]');
    expect(visible(entry?.textContent)).toContain('8,000');
    // No partner share in a posting of garden goods.
    expect(visible(entry?.textContent)).not.toContain('2,196');
  });

  it('R8: the print carries the goods, and all goods carries none', async () => {
    const user = userEvent.setup();
    await renderApp('/commission?goods=trip');
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionPrintData).mock.calls[0]?.[0]).toEqual({
        month: '2026-10',
        goods: 'trip',
        language,
        paper: 'a4',
      });
    });
  });
});
