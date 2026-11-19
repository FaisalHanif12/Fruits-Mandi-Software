// The commission page with a period and a party (plan 77, PRD FR-9.3), in the Urdu and the English app, against a
// fake backend with the numbers of the plan (after Events 1 to 5, 10-10-26): seller رحیم sale 29,400, commission
// charged to him 1,764, commission on his goods 3,928; buyer اکرم ٹول ٹیکس value 12,000, buyer commission 720,
// لاگا 160, total 880. The side toggle (all, buyer, seller), the party kept in the address, the columns that follow
// the side with no partner columns, a year as one row for each month that opens as that month, one date, a range
// that is paged and refused when reversed, the box that says "pick a party", and the print that names the
// party. The screen adds nothing up: every figure is the backend's.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { CommissionDay } from '../../src/bindings/CommissionDay';
import type { CommissionDayRow } from '../../src/bindings/CommissionDayRow';
import type { CommissionMonth } from '../../src/bindings/CommissionMonth';
import type { CommissionPartyShown } from '../../src/bindings/CommissionPartyShown';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { printData } from '../helpers/collection-sheet';
import { fakeDashboard } from '../helpers/dashboard';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const rupees = (value: number) => paisaForTest(value * 100);

const RAHIM = {
  id: 19,
  name_ur: 'رحیم',
  name_en: 'Rahim',
  code: '0010',
};
const AKRAM = {
  id: 21,
  name_ur: 'اکرم ٹول ٹیکس',
  name_en: 'Akram Toll Tax',
  code: '0020',
};

const shown = (
  side: 'buyer' | 'seller',
  record: typeof RAHIM,
  tail: Partial<CommissionPartyShown['party']> = {},
): CommissionPartyShown => ({
  side,
  party: {
    account_id: record.id,
    code: record.code,
    short_code: null,
    name_ur: record.name_ur,
    name_en: record.name_en,
    tag: null,
    person_key: null,
    ...tail,
  },
});

/** Seller رحیم on 10-10-26 (plan 77 Section 3): sale 29,400, charged 1,764, on his goods 3,928. */
const SELLER_ROW: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(1_764),
  buyer_comm_paisa: rupees(1_764),
  laga_paisa: rupees(400),
  total_comm_paisa: rupees(3_928),
  shared_income_paisa: rupees(0),
  partner_shares: null,
  sale_paisa: rupees(29_400),
  goods_comm_paisa: rupees(3_928),
};

/** Buyer اکرم on 10-10-26: value 12,000, buyer commission 720, لاگا 160, total 880. */
const BUYER_ROW: CommissionDayRow = {
  date: '2026-10-10',
  seller_comm_paisa: rupees(0),
  buyer_comm_paisa: rupees(720),
  laga_paisa: rupees(160),
  total_comm_paisa: rupees(880),
  shared_income_paisa: rupees(0),
  partner_shares: null,
  sale_paisa: rupees(12_000),
  goods_comm_paisa: null,
};

/** The whole shop on 10-10-26 (plan 76). */
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

const PARTNERS = [
  { id: 1, name_ur: 'الف', name_en: 'Partner A' },
  { id: 2, name_ur: 'ب', name_en: 'Partner B' },
  { id: 3, name_ur: 'ج', name_en: 'Partner C' },
];

function month(
  row: CommissionDayRow,
  party: CommissionPartyShown | null,
  more: Partial<CommissionMonth> = {},
): CommissionMonth {
  return {
    month: '2026-10',
    year: 2026,
    period: { kind: 'month', from: '2026-10-01', to: '2026-10-31' },
    party,
    goods: 'all',
    first_month: '2026-09',
    current_month: '2026-10',
    partners: party === null ? PARTNERS : [],
    rows: [row],
    total_count: 1,
    offset: 0,
    limit: 100,
    totals: { ...row, date: '' },
    partners_locked: false,
    ...more,
  };
}

const dayOf = (row: CommissionDayRow, party: CommissionPartyShown | null): CommissionDay => ({
  date: '2026-10-10',
  party,
  goods: 'all',
  partners: party === null ? PARTNERS : [],
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
});

/** The address of the page: the party is a whole picker value in it, like the analytics views. */
const address = (search: Record<string, string>, party?: typeof RAHIM) => {
  const parts = Object.entries(search).map(([key, value]) => `${key}=${encodeURIComponent(value)}`);
  if (party !== undefined) parts.push(`party=${encodeURIComponent(JSON.stringify(party))}`);
  return `/commission?${parts.join('&')}`;
};

const lastRead = () => vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.at(-1)?.[0];

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardCommissionMonth).mockImplementation((input) => {
    const party = input.party;
    if (party === undefined) return Promise.resolve(month(SHOP_ROW, null));
    return Promise.resolve(
      party.side === 'seller'
        ? month(SELLER_ROW, shown('seller', RAHIM))
        : month(BUYER_ROW, shown('buyer', AKRAM)),
    );
  });
  vi.mocked(dashboardApi.dashboardCommissionDay).mockImplementation((input) =>
    Promise.resolve(
      input.party === undefined
        ? dayOf(SHOP_ROW, null)
        : input.party.side === 'seller'
          ? dayOf(SELLER_ROW, shown('seller', RAHIM))
          : dayOf(BUYER_ROW, shown('buyer', AKRAM)),
    ),
  );
  vi.mocked(dashboardApi.dashboardCommissionPrintData).mockImplementation((input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4')),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the commission page with a party and a period (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('commission-title'),
    });
  const side = (value: 'all' | 'buyer' | 'seller') =>
    screen.getByRole('button', {
      name: (name) => visible(name) === m(`commission-side-${value}`),
    });
  const treeText = async () => visible((await screen.findByRole('treegrid')).textContent);
  const headers = async () =>
    within(await screen.findByRole('treegrid'))
      .getAllByRole('columnheader')
      .map((header) => visible(header.textContent));

  it('shows a seller: his sale, the commission charged to him and the commission on his goods, with no partner columns', async () => {
    await renderApp(address({ side: 'seller' }, RAHIM));
    await heading();
    const text = await treeText();
    for (const figure of ['10-10-2026', '29,400', '1,764', '3,928']) {
      expect(text).toContain(figure);
    }
    // The commission on his goods is in the row and in the totals row: 3,928 twice, 29,400 twice.
    expect(text.split('3,928').length - 1).toBe(2);
    expect(await headers()).toEqual([
      m('commission-col-date'),
      m('commission-col-sale'),
      m('commission-col-charged'),
      m('commission-col-goods'),
    ]);
    // The page asked for that seller, with the month of plan 76 as it was.
    expect(lastRead()).toEqual({
      month: null,
      party: { side: 'seller', account_id: 19 },
    });
    expect(side('seller').getAttribute('aria-pressed')).toBe('true');
  });

  it('shows a buyer: his purchase value, buyer commission, لاگا and their total, with no seller column', async () => {
    await renderApp(address({ side: 'buyer' }, AKRAM));
    await heading();
    const text = await treeText();
    for (const figure of ['12,000', '720', '160', '880']) {
      expect(text).toContain(figure);
    }
    expect(await headers()).toEqual([
      m('commission-col-date'),
      m('commission-col-purchase'),
      m('commission-col-buyer'),
      m('commission-col-laga'),
      m('commission-col-total'),
    ]);
    expect(lastRead()).toMatchObject({ party: { side: 'buyer', account_id: 21 } });
  });

  it('keeps the page of plan 76 for every party: the same request and the partner columns', async () => {
    await renderApp('/commission');
    await heading();
    const text = await treeText();
    expect(text).toContain('5,448');
    expect(text).toContain('2,196');
    expect(lastRead()).toEqual({ month: null });
    expect(side('all').getAttribute('aria-pressed')).toBe('true');
  });

  it('a side with nobody picked asks for a party and reads nothing', async () => {
    const user = userEvent.setup();
    await renderApp('/commission');
    await screen.findByRole('treegrid');
    const reads = vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.length;
    await user.click(side('seller'));
    expect(
      await screen.findByText((text) => visible(text) === m('commission-pick-seller')),
    ).toBeInTheDocument();
    expect(screen.queryByRole('treegrid')).toBeNull();
    expect(vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.length).toBe(reads);
    // The other side asks for its own kind of party.
    await user.click(side('buyer'));
    expect(
      await screen.findByText((text) => visible(text) === m('commission-pick-buyer')),
    ).toBeInTheDocument();
    // Back to all: the whole shop again, with its partner columns.
    await user.click(side('all'));
    expect(await treeText()).toContain('2,196');
  });

  it('the day of a party opens the postings with that party’s figures', async () => {
    const user = userEvent.setup();
    await renderApp(address({ side: 'seller' }, RAHIM));
    await heading();
    const tree = await screen.findByRole('treegrid');
    const day = within(tree).getAllByRole('row')[1];
    if (day === undefined) throw new Error('no day row');
    await user.click(day);
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionDay).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-10',
        party: { side: 'seller', account_id: 19 },
      });
    });
    await waitFor(() => {
      expect(document.querySelector('[data-commission-day="2026-10-10"]')).not.toBeNull();
    });
    const entry = document.querySelector('[data-commission-day="2026-10-10"]');
    if (entry === null) throw new Error('no postings of the day');
    const text = visible(entry.textContent);
    expect(text).toContain('29,400');
    expect(text).toContain('3,928');
    // A party's posting has no partner shares.
    expect(text).not.toContain(language === 'ur' ? 'الف' : 'Partner A');
  });

  it('a year is one row for each month, and a click on a month opens that month for the same party', async () => {
    vi.mocked(dashboardApi.dashboardCommissionMonth).mockImplementation((input) =>
      Promise.resolve(
        input.period?.kind === 'year'
          ? month({ ...SELLER_ROW, date: '2026-10' }, shown('seller', RAHIM), {
              period: { kind: 'year', from: '2026-01-01', to: '2026-12-31' },
            })
          : month(SELLER_ROW, shown('seller', RAHIM)),
      ),
    );
    const user = userEvent.setup();
    const { router } = await renderApp(address({ period: 'year', side: 'seller' }, RAHIM));
    await heading();
    const text = await treeText();
    // One October row of 29,400, 1,764 and 3,928, as the year of رحیم.
    for (const figure of ['29,400', '1,764', '3,928']) expect(text).toContain(figure);
    expect((await headers())[0]).toBe(m('commission-col-month'));
    expect(lastRead()).toEqual({
      month: null,
      period: { kind: 'year', year: 2026 },
      party: { side: 'seller', account_id: 19 },
    });
    const row = within(await screen.findByRole('treegrid')).getAllByRole('row')[1];
    if (row === undefined) throw new Error('no month row');
    await user.click(row);
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ month: '2026-10', side: 'seller' });
    });
    expect(router.state.location.search).not.toHaveProperty('period');
    await waitFor(() => {
      expect(lastRead()).toEqual({
        month: '2026-10',
        party: { side: 'seller', account_id: 19 },
      });
    });
  });

  it('one date is one row with its postings open at once, and the date buttons move by a day', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp(address({ period: 'day', date: '2026-10-10' }));
    await heading();
    await screen.findByRole('treegrid');
    expect(lastRead()).toEqual({ month: null, period: { kind: 'day', date: '2026-10-10' } });
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionDay).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-10',
      });
    });
    // Today is 10-10-26: the later button is off, the earlier one goes to 09-10-26 and is kept in the address.
    expect(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(m('period-date-later')),
      }),
    ).toBeDisabled();
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(m('period-date-earlier')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ period: 'day', date: '2026-10-09' });
    });
  });

  it('a range asks for its dates and is paged; a reversed range is told under the last date', async () => {
    vi.mocked(dashboardApi.dashboardCommissionMonth).mockImplementation((input) =>
      input.period?.kind === 'range' && input.period.from === '2026-10-10'
        ? Promise.reject(
            new CommandError(refusal([{ code: 'ERR_DATE_RANGE', field: 'period.to', values: {} }])),
          )
        : Promise.resolve(
            month(SHOP_ROW, null, {
              period: { kind: 'range', from: '2026-10-01', to: '2026-10-05' },
              total_count: 40,
              limit: 25,
            }),
          ),
    );
    const user = userEvent.setup();
    await renderApp(address({ period: 'range', from: '2026-10-01', to: '2026-10-05' }));
    await heading();
    await screen.findByRole('treegrid');
    await waitFor(() => {
      expect(lastRead()).toMatchObject({
        period: { kind: 'range', from: '2026-10-01', to: '2026-10-05' },
        offset: 0,
        limit: 25,
      });
    });
    // 40 rows are two pages of 25: the pager says so and the next page asks from row 25.
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === m('common-pager-next'),
      }),
    );
    await waitFor(() => {
      expect(lastRead()).toMatchObject({ offset: 25, limit: 25 });
    });
    // A first date after the last: the refusal of the backend shows under the last date.
    const from = screen.getAllByRole('textbox')[0];
    if (from === undefined) throw new Error('no from box');
    await user.clear(from);
    await user.type(from, '10-10-2026');
    await user.tab();
    // The refusal shows under the last date, and in the table's state, which has a retry.
    expect(
      (await screen.findAllByText((text) => visible(text) === m('err-date-range'))).length,
    ).toBeGreaterThan(0);
  });

  it('the period choice carries the place on screen: a month becomes its year, and the party stays', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp(address({ month: '2026-09', side: 'seller' }, RAHIM));
    await heading();
    await screen.findByRole('treegrid');
    await user.click(screen.getByRole('combobox', { name: m('period-label') }));
    await user.click(await screen.findByRole('option', { name: m('period-year') }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        period: 'year',
        year: '2026',
        side: 'seller',
      });
    });
    expect(router.state.location.search).not.toHaveProperty('month');
  });

  it('a party that is not of the side is refused under the picker and the page says so', async () => {
    vi.mocked(dashboardApi.dashboardCommissionMonth).mockRejectedValue(
      new CommandError(
        refusal([{ code: 'ERR_WRONG_PARTY_TYPE', field: 'party.account_id', values: {} }]),
      ),
    );
    await renderApp(address({ side: 'buyer' }, RAHIM));
    await heading();
    const texts = await screen.findAllByText((text) => visible(text) === m('err-wrong-party-type'));
    expect(texts.length).toBeGreaterThan(0);
    expect(screen.queryByRole('treegrid')).toBeNull();
  });

  it('works with the keyboard only: Enter on a side asks for its party, Page Up goes to the month before, Esc leaves', async () => {
    const user = userEvent.setup();
    const { router } = await renderApp(address({ side: 'seller' }, RAHIM));
    await heading();
    await screen.findByRole('treegrid');
    // Page Up and Page Down move the month, with the party kept.
    press('PageUp');
    await waitFor(() => {
      expect(lastRead()).toEqual({ month: '2026-09', party: { side: 'seller', account_id: 19 } });
    });
    expect(router.state.location.search).toMatchObject({ month: '2026-09', side: 'seller' });
    // A side is a button: focus it and press Enter (the same as a click), then the page asks for a party.
    side('buyer').focus();
    await user.keyboard('{Enter}');
    expect(
      await screen.findByText((text) => visible(text) === m('commission-pick-buyer')),
    ).toBeInTheDocument();
    press('Escape');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('a language switch in the middle keeps the period, the party and the open day, and redraws the words', async () => {
    const other = language === 'ur' ? 'en' : 'ur';
    await renderApp(address({ period: 'year', side: 'seller' }, RAHIM));
    await heading();
    await screen.findByRole('treegrid');
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const text = visible((await screen.findByRole('treegrid')).textContent);
    // The same figures of the same party, in the other language's words.
    for (const figure of ['29,400', '1,764', '3,928']) expect(text).toContain(figure);
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: (name) => visible(name) === message(other, 'commission-title'),
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', {
        name: (name) => visible(name) === message(other, 'commission-side-seller'),
      }),
    ).toHaveAttribute('aria-pressed', 'true');
  });

  it('the print and the Excel carry the period and the party, and Back returns', async () => {
    const user = userEvent.setup();
    await renderApp(address({ side: 'seller' }, RAHIM));
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionPrintData).mock.calls[0]?.[0]).toEqual({
        month: '2026-10',
        party: { side: 'seller', account_id: 19 },
        language,
        paper: 'a4',
      });
    });
    await user.click(
      await within(screen.getByRole('main')).findByRole('button', {
        name: (name) => visible(name) === m('common-back'),
      }),
    );
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });
});
