// The commission page (plan 76 point 7, PRD FR-9.3, FR-19.11) in the Urdu and the English app, against a fake
// backend: the tile "Today's commission" opens it on the current month, a row for each day with its seller
// and buyer commission, لاگا, total, shared income and each partner's share (E2E numbers of 10-10-26), the
// month totals from the backend, a day that opens in place with the postings behind it, the month buttons
// and the box that go to another month (kept in the address), the partner columns behind the partner
// code, and the four states. The screen adds nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { CommissionDay } from '../../src/bindings/CommissionDay';
import type { CommissionDayRow } from '../../src/bindings/CommissionDayRow';
import type { CommissionMonth } from '../../src/bindings/CommissionMonth';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
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
import { refusal } from '../helpers/ui-error';

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

/** 10-10-26 after Events 1 to 5 (PRD FR-9.3): 2,484 + 2,484 + 480 = 5,448, shared income 5,448 + 40 = 5,488. */
const DAY_ROW: CommissionDayRow = {
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

const MONTH: CommissionMonth = {
  month: '2026-10',
  year: 2026,
  period: { kind: 'month', from: '2026-10-01', to: '2026-10-31' },
  party: null,
  total_count: 1,
  offset: 0,
  limit: 100,
  first_month: '2026-09',
  current_month: '2026-10',
  partners: PARTNERS,
  rows: [DAY_ROW],
  totals: { ...DAY_ROW, date: '' },
  partners_locked: false,
};

const DAY: CommissionDay = {
  date: '2026-10-10',
  party: null,
  partners: PARTNERS,
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
      figures: DAY_ROW,
    },
  ],
  total_count: 1,
  totals: DAY_ROW,
  partners_locked: false,
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardCommissionMonth).mockImplementation((input) =>
    Promise.resolve(
      input.month === '2026-09'
        ? {
            ...MONTH,
            month: '2026-09',
            rows: [],
            totals: { ...MONTH.totals, partner_shares: null },
          }
        : MONTH,
    ),
  );
  vi.mocked(dashboardApi.dashboardCommissionDay).mockResolvedValue(DAY);
  vi.mocked(dashboardApi.dashboardCommissionPrintData).mockImplementation((input) =>
    Promise.resolve(printData(input.language, input.paper ?? 'a4')),
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the commission page (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const open = () => renderApp('/commission');
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('commission-title'),
    });
  const button = (id: string) =>
    screen.getByRole('button', { name: (name) => visible(name).includes(m(id)) });

  it('opens on the current month and shows the day, the month totals and each partner’s share', async () => {
    await open();
    await heading();
    expect(await screen.findByRole('treegrid')).toBeTruthy();
    const text = visible(screen.getByRole('treegrid').textContent);
    // The day of the example, and the figures as the backend gave them.
    for (const figure of ['10-10-2026', '2,484', '480', '5,448', '5,488', '2,196', '1,646']) {
      expect(text).toContain(figure);
    }
    expect(text).toContain(language === 'ur' ? 'الف' : 'Partner A');
    // The first request is for the current month (no month in the address).
    expect(vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls[0]?.[0]).toEqual({
      month: null,
    });
    // The month's own totals row is the backend's: 5,448 appears on the day and in the total.
    expect(text.split('5,448').length - 1).toBe(2);
  });

  it('a click on a day opens the postings behind it, in place, with a link to the بکری', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await heading();
    const tree = await screen.findByRole('treegrid');
    const day = within(tree).getAllByRole('row')[1];
    if (day === undefined) throw new Error('no day row');
    await user.click(day);
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionDay).mock.calls[0]?.[0]).toMatchObject({
        date: '2026-10-10',
      });
    });
    const link = await screen.findByRole('button', {
      name: (name) => visible(name) === m('commission-bikri', { number: 1 }),
    });
    await user.click(link);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41 });
  });

  it('the month buttons go to the month before, stop at the first month with commission, and keep the month in the address', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await heading();
    await screen.findByRole('treegrid');
    // The later button is off on the current month, the earlier one goes to September.
    expect((button('common-month-later') as HTMLButtonElement).disabled).toBe(true);
    await user.click(button('common-month-earlier'));
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.at(-1)?.[0]).toEqual({
        month: '2026-09',
      });
    });
    expect(router.state.location.search).toMatchObject({ month: '2026-09' });
    // September is the first month with commission: the earlier button is off, and an empty month says so.
    await waitFor(() => {
      expect((button('common-month-earlier') as HTMLButtonElement).disabled).toBe(true);
    });
    expect(
      await screen.findByText((text) => visible(text) === m('commission-empty')),
    ).toBeInTheDocument();
  });

  it('goes to a month that is typed, in the short form', async () => {
    const user = userEvent.setup();
    await open();
    await heading();
    await screen.findByRole('treegrid');
    await user.type(screen.getByPlaceholderText(m('common-goto-month-placeholder')), '9{Enter}');
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.at(-1)?.[0]).toEqual({
        month: '2026-09',
      });
    });
  });

  it('the partner columns wait for the code: the box shows and the rest of the page works', async () => {
    vi.mocked(dashboardApi.dashboardCommissionMonth).mockResolvedValue({
      ...MONTH,
      partners_locked: true,
      rows: [{ ...DAY_ROW, partner_shares: null }],
      totals: { ...DAY_ROW, date: '', partner_shares: null },
    });
    await open();
    await heading();
    expect(
      await screen.findByText((text) => visible(text) === m('partners-code-box-text')),
    ).toBeInTheDocument();
    const text = visible(screen.getByRole('treegrid').textContent);
    expect(text).toContain('5,448');
    expect(text).not.toContain('2,196');
    expect(text).not.toContain(language === 'ur' ? 'الف' : 'Partner A');
  });

  it('the print button shows the month on screen in the print layout, with the partner columns as the backend sent them, and Back returns', async () => {
    const user = userEvent.setup();
    await open();
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(vi.mocked(dashboardApi.dashboardCommissionPrintData).mock.calls[0]?.[0]).toEqual({
        month: '2026-10',
        language,
        paper: 'a4',
      });
    });
    await waitFor(() => {
      expect(visible(document.querySelector('.print-sheet')?.textContent)).toContain('44,364');
    });
    await user.click(
      await screen.findByRole('button', { name: (name) => visible(name) === m('common-back') }),
    );
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });

  it('an error is told with a retry that asks again', async () => {
    vi.mocked(dashboardApi.dashboardCommissionMonth).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
    );
    const user = userEvent.setup();
    await open();
    const retry = await screen.findByRole('button', {
      name: (name) => visible(name) === m('common-retry'),
    });
    await user.click(retry);
    expect(await screen.findByRole('treegrid')).toBeTruthy();
  });
});
