// The commission page against the backend's own answers of plan 77 (worked day, October 2026): a seller, a buyer, a
// چکتا buyer with his person key, a year, a range and the print of a seller. They show that the page reads the real
// shapes (the rows of a year are `YYYY-MM`, the party is named in the answer, a party has no partners) and sends
// the party and the period the way the backend's own requests do. The screen adds nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
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
import {
  COMMISSION_BUYER_AKRAM,
  COMMISSION_BUYER_AKRAM_REQUEST,
  COMMISSION_BUYER_SHAHID,
  COMMISSION_BUYER_SHAHID_REQUEST,
  COMMISSION_DAY_SELLER_RAHIM,
  COMMISSION_PRINT_SELLER_EN,
  COMMISSION_RANGE_SHOP,
  COMMISSION_RANGE_SHOP_REQUEST,
  COMMISSION_SELLER_RAHIM,
  COMMISSION_SELLER_RAHIM_REQUEST,
  COMMISSION_YEAR_RAHIM,
  COMMISSION_YEAR_RAHIM_REQUEST,
} from '../helpers/commission-real';
import { fakeDashboard } from '../helpers/dashboard';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

const address = (search: Record<string, string>, party?: object) => {
  const parts = Object.entries(search).map(([key, value]) => `${key}=${encodeURIComponent(value)}`);
  if (party !== undefined) parts.push(`party=${encodeURIComponent(JSON.stringify(party))}`);
  return `/commission?${parts.join('&')}`;
};

const RAHIM = { id: 19, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' };
const AKRAM = { id: 21, name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax', code: '0020' };
const SHAHID = { id: 9, name_ur: 'شاہد', name_en: 'Shahid', code: '0030' };

const lastRead = () => vi.mocked(dashboardApi.dashboardCommissionMonth).mock.calls.at(-1)?.[0];

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardCommissionMonth).mockImplementation((input) => {
    if (input.period?.kind === 'year') return Promise.resolve(COMMISSION_YEAR_RAHIM);
    if (input.period?.kind === 'range') return Promise.resolve(COMMISSION_RANGE_SHOP);
    if (input.party?.chukta_person_key !== undefined) {
      return Promise.resolve(COMMISSION_BUYER_SHAHID);
    }
    if (input.party?.side === 'buyer') return Promise.resolve(COMMISSION_BUYER_AKRAM);
    return Promise.resolve(COMMISSION_SELLER_RAHIM);
  });
  vi.mocked(dashboardApi.dashboardCommissionDay).mockResolvedValue(COMMISSION_DAY_SELLER_RAHIM);
  vi.mocked(dashboardApi.dashboardCommissionPrintData).mockResolvedValue(
    COMMISSION_PRINT_SELLER_EN,
  );
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the commission page with the backend’s own answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const treeText = async () => visible((await screen.findByRole('treegrid')).textContent);

  it('a seller: 29,400, 1,764 and 3,928 for 10-10-2026, asked for the way the backend’s request is', async () => {
    await renderApp(address({ side: 'seller' }, RAHIM));
    const text = await treeText();
    for (const figure of ['10-10-2026', '29,400', '1,764', '3,928']) expect(text).toContain(figure);
    // The same party and the same month as the backend's own request.
    expect(lastRead()?.party).toEqual(COMMISSION_SELLER_RAHIM_REQUEST.party);
    expect(lastRead()).not.toHaveProperty('offset');
  });

  it('a buyer: 12,000, 720, 160 and 880', async () => {
    await renderApp(address({ side: 'buyer' }, AKRAM));
    const text = await treeText();
    for (const figure of ['12,000', '720', '160', '880']) expect(text).toContain(figure);
    expect(lastRead()?.party).toEqual(COMMISSION_BUYER_AKRAM_REQUEST.party);
  });

  it('a چکتا buyer keeps his person key in the request, and his total is 800', async () => {
    const person = COMMISSION_BUYER_SHAHID_REQUEST.party?.chukta_person_key;
    if (person === undefined) throw new Error('the fixture has no person key');
    await renderApp(address({ side: 'buyer', person }, SHAHID));
    const text = await treeText();
    for (const figure of ['12,000', '800']) expect(text).toContain(figure);
    expect(lastRead()?.party).toEqual(COMMISSION_BUYER_SHAHID_REQUEST.party);
  });

  it('a year is one row for each month with the same figures, and asks the way the backend’s request does', async () => {
    await renderApp(address({ period: 'year', year: '2026', side: 'seller' }, RAHIM));
    const tree = await screen.findByRole('treegrid');
    const text = visible(tree.textContent);
    for (const figure of ['29,400', '1,764', '3,928']) expect(text).toContain(figure);
    // One October row, written with its month name.
    expect(text).toContain(message(language, 'report-month-10'));
    expect(within(tree).getAllByRole('row').length).toBe(3);
    expect(lastRead()?.period).toEqual(COMMISSION_YEAR_RAHIM_REQUEST.period);
    expect(lastRead()?.party).toEqual(COMMISSION_YEAR_RAHIM_REQUEST.party);
  });

  it('a range asks with its dates and the page the backend’s request has, and shows the shop’s day', async () => {
    await renderApp(address({ period: 'range', from: '2026-10-01', to: '2026-10-10' }));
    const text = await treeText();
    for (const figure of ['5,448', '2,484']) expect(text).toContain(figure);
    await waitFor(() => {
      expect(lastRead()).toMatchObject({ offset: 0, limit: COMMISSION_RANGE_SHOP_REQUEST.limit });
    });
    expect(lastRead()?.period).toMatchObject({
      kind: 'range',
      from: '2026-10-01',
      to: '2026-10-10',
    });
  });

  it('the print of a seller names him in the header and keeps the seller’s columns', async () => {
    const user = userEvent.setup();
    await renderApp(address({ side: 'seller' }, RAHIM));
    await screen.findByRole('treegrid');
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).includes(m('report-preview')),
      }),
    );
    await waitFor(() => {
      expect(document.querySelector('.print-sheet')).not.toBeNull();
    });
    const sheet = visible(document.querySelector('.print-sheet')?.textContent);
    expect(sheet).toContain('Rahim (0010)');
    expect(sheet).toContain('3,928');
    // The words of the seller's columns come from the print messages the backend named.
    expect(sheet).toContain(message('en', 'print-commission-col-goods'));
    expect(vi.mocked(dashboardApi.dashboardCommissionPrintData).mock.calls[0]?.[0]).toMatchObject({
      party: { side: 'seller', account_id: 19 },
    });
  });
});
