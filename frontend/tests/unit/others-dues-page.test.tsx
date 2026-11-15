// The page behind the tile "Garden owners' and others' dues" (plan 76 point 10, PRD FR-9.1, FR-16.13, FR-19.11),
// in the Urdu and the English app, against a fake backend: the tile shows the open total and the number of
// people and opens the page; the page shows one month of money given and returned, the people with something
// open, and the backend's totals; the filters (person type, link) and the month buttons go to the backend; a
// row opens the person on the ادھار رقم screen; and the four states. The page adds nothing up.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { LentDues } from '../../src/bindings/LentDues';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as dashboardApi from '../../src/features/dashboard/api';
import { LANGUAGES } from '../../src/i18n';
import * as windowsApi from '../../src/shared/windows/api';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeDashboard } from '../helpers/dashboard';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/windows/api');

const rupees = (value: number) => paisaForTest(value * 100);

const NAWAZ = { id: 91, code: '01030091', name_ur: 'نواز', name_en: 'Nawaz' };
const SHAFIQ = { id: 92, code: '01030092', name_ur: 'شفیق', name_en: 'Shafiq' };

/** The example of the plan: Nawaz (garden owner, linked to partner A) was given 20,000 and returned 5,000. */
const DUES: LentDues = {
  partner_locked: false,
  people_total_count: 2,
  month: '2026-10',
  rows: [
    {
      kind: 'given',
      entry_date: '2026-10-01',
      person: NAWAZ,
      person_type: 'garden_owner',
      link: { kind: 'partner', partner_id: 1 },
      voucher_id: 301,
      number: 4,
      reason: 'next season garden',
      amount_paisa: rupees(20_000),
    },
    {
      kind: 'returned',
      entry_date: '2026-10-10',
      person: NAWAZ,
      person_type: 'garden_owner',
      link: { kind: 'partner', partner_id: 1 },
      voucher_id: 302,
      number: 2,
      reason: '',
      amount_paisa: rupees(5_000),
    },
    {
      kind: 'given',
      entry_date: '2026-10-05',
      person: SHAFIQ,
      person_type: 'trip_person',
      link: { kind: 'shop', partner_id: null },
      voucher_id: 303,
      number: 5,
      reason: 'trip money',
      amount_paisa: rupees(3_000),
    },
  ],
  total_count: 3,
  totals: {
    given_paisa: rupees(23_000),
    returned_paisa: rupees(5_000),
    open_paisa: rupees(18_000),
    open_people_count: 2,
  },
  people: [
    {
      person: NAWAZ,
      person_type: 'garden_owner',
      link: { kind: 'partner', partner_id: 1 },
      lent_paisa: rupees(20_000),
      returned_paisa: rupees(5_000),
      open_paisa: rupees(15_000),
    },
    {
      person: SHAFIQ,
      person_type: 'trip_person',
      link: { kind: 'shop', partner_id: null },
      lent_paisa: rupees(3_000),
      returned_paisa: rupees(0),
      open_paisa: rupees(3_000),
    },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  fakePartners(THREE_PARTNERS);
  vi.mocked(windowsApi.windowsBounds).mockResolvedValue({
    first_date: '2026-08-15',
    last_date: '2026-10-10',
  });
  vi.mocked(cashbookApi.lentDues).mockResolvedValue(DUES);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the dues of garden owners and others (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = {
    nawaz: language === 'ur' ? 'نواز' : 'Nawaz',
    shafiq: language === 'ur' ? 'شفیق' : 'Shafiq',
    partnerA: language === 'ur' ? 'الف' : 'Partner A',
    partnerB: language === 'ur' ? 'ب' : 'Partner B',
  };
  const open = () => renderApp('/others-dues');
  const heading = () =>
    screen.findByRole('heading', {
      level: 1,
      name: (name) => visible(name) === m('cashbook-dues-title'),
    });
  const lastAsked = () => vi.mocked(cashbookApi.lentDues).mock.calls.at(-1)?.[0];
  /** A matcher for a name or text without the isolation marks of a glossary word. */
  const said = (text: string) => (name: string) => visible(name) === text;
  const rowOf = (id: string) => document.querySelector<HTMLElement>(`.ag-row[row-id="${id}"]`);
  const button = (id: string) =>
    screen.getByRole('button', { name: (name) => visible(name).includes(m(id)) });

  it('shows what is open and how many people, what was given and returned in the month, and the people', async () => {
    await open();
    await heading();
    // The grids draw their rows after the answer: wait for the last row of each.
    await waitFor(() => {
      expect(rowOf('92')).not.toBeNull();
      expect(rowOf('given-303-2026-10-05')).not.toBeNull();
    });
    const page = visible(document.body.textContent);
    // The backend's totals: open 18,000 of 2 people, given 23,000, returned 5,000.
    for (const figure of ['18,000', '23,000', '5,000', '15,000', '3,000', '20,000']) {
      expect(page).toContain(figure);
    }
    expect(page).toContain(names.nawaz);
    expect(page).toContain(names.shafiq);
    // Each person has his type and his link in words; a partner link shows the partner's name.
    expect(page).toContain(m('cashbook-person-type-garden-owner'));
    expect(page).toContain(m('cashbook-person-type-trip-person'));
    expect(page).toContain(names.partnerA);
    expect(page).toContain(m('cashbook-link-word-shop'));
    // The entries of the month, with the reason and whether it was given or returned.
    expect(page).toContain('next season garden');
    expect(page).toContain(m('cashbook-dues-kind-given'));
    expect(page).toContain(m('cashbook-dues-kind-returned'));
    // The first request: every type but staff, any link, the current month (a month window of the day).
    expect(vi.mocked(cashbookApi.lentDues).mock.calls[0]?.[0]).toMatchObject({
      person_types: [],
      link_kinds: [],
      partner_id: null,
      month: '2026-10',
    });
  });

  it('sends the type and the link filters to the backend, and the partner for a partner link', async () => {
    const user = userEvent.setup();
    await open();
    await heading();
    await screen.findAllByText((text) => visible(text).includes('18,000'));
    await user.click(
      await screen.findByRole('combobox', { name: said(m('cashbook-dues-filter-type')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: said(m('cashbook-person-type-garden-owner')) }),
    );
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({ person_types: ['garden_owner'] });
    });
    await user.click(
      await screen.findByRole('combobox', { name: said(m('cashbook-dues-filter-link')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: said(m('cashbook-dues-link-partner')) }),
    );
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({ link_kinds: ['partner'], partner_id: null });
    });
    await user.click(
      await screen.findByRole('combobox', { name: said(m('cashbook-link-partner')) }),
    );
    await user.click(await screen.findByRole('option', { name: said(names.partnerB) }));
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({
        person_types: ['garden_owner'],
        link_kinds: ['partner'],
        partner_id: 2,
      });
    });
  });

  /** The dues as the backend answers them while the partner section is locked: no partner is named. */
  const lockedDues = (): LentDues => ({
    ...DUES,
    partner_locked: true,
    rows: DUES.rows.map((row) =>
      row.link.kind === 'partner' ? { ...row, link: { kind: 'partner', partner_id: null } } : row,
    ),
    people: DUES.people.map((person) =>
      person.link.kind === 'partner'
        ? { ...person, link: { kind: 'partner', partner_id: null } }
        : person,
    ),
  });

  it('while the partner section is locked no partner is named, the code box stands there, and the amounts stay', async () => {
    vi.mocked(cashbookApi.lentDues).mockResolvedValue(lockedDues());
    const user = userEvent.setup();
    await open();
    await heading();
    await waitFor(() => {
      expect(rowOf('92')).not.toBeNull();
    });
    // The amounts are the shop's and show; the partner of a link does not: A is not named.
    const page = visible(document.body.textContent);
    for (const figure of ['18,000', '23,000', '5,000', '15,000']) expect(page).toContain(figure);
    expect(page).not.toContain(names.partnerA);
    // The box says why, with the button that opens the code.
    expect(await screen.findByText(m('partners-code-box-text'))).toBeInTheDocument();
    // The partner filter is the box too, and no partner is asked for.
    await user.click(
      await screen.findByRole('combobox', { name: said(m('cashbook-dues-filter-link')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: said(m('cashbook-dues-link-partner')) }),
    );
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({ link_kinds: ['partner'], partner_id: null });
    });
    expect(screen.queryByRole('combobox', { name: said(m('cashbook-link-partner')) })).toBeNull();
    expect(screen.getAllByText(m('partners-code-box-text'))).toHaveLength(1);
  });

  it('a refusal of the partner filter, when the section locked after the page opened, shows the box and asks again with no partner', async () => {
    const user = userEvent.setup();
    await open();
    await heading();
    await screen.findAllByText((text) => visible(text).includes('18,000'));
    await user.click(
      await screen.findByRole('combobox', { name: said(m('cashbook-dues-filter-link')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: said(m('cashbook-dues-link-partner')) }),
    );
    await user.click(
      await screen.findByRole('combobox', { name: said(m('cashbook-link-partner')) }),
    );
    // Locked meanwhile: the filter on partner B is refused at its field, then the page is asked without it.
    vi.mocked(cashbookApi.lentDues).mockImplementation((input) =>
      input.partner_id === null
        ? Promise.resolve(lockedDues())
        : Promise.reject(
            new CommandError(refusal([{ code: 'ERR_PARTNER_LOCKED', field: 'partner_id' }])),
          ),
    );
    await user.click(await screen.findByRole('option', { name: said(names.partnerB) }));
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({ link_kinds: ['partner'], partner_id: null });
    });
    expect(await screen.findByText(m('partners-code-box-text'))).toBeInTheDocument();
    expect(screen.queryByRole('combobox', { name: said(m('cashbook-link-partner')) })).toBeNull();
  });

  it('the people have pages of their own: the first page is asked for, the next one comes from the pager, and the entries ask for one person only', async () => {
    // Thirty people owe something: the backend gives a page of them and says how many there are in all.
    const everyone = Array.from({ length: 30 }, (_, index) => ({
      person: {
        id: 200 + index,
        code: `0103${String(200 + index)}`,
        name_ur: `شخص ${String(index)}`,
        name_en: `Person ${String(index)}`,
      },
      person_type: 'other' as const,
      link: { kind: 'shop' as const, partner_id: null },
      lent_paisa: rupees(1_000),
      returned_paisa: rupees(0),
      open_paisa: rupees(1_000),
    }));
    vi.mocked(cashbookApi.lentDues).mockImplementation((input) =>
      Promise.resolve({
        ...DUES,
        people: everyone.slice(input.people_offset, input.people_offset + input.people_limit),
        people_total_count: everyone.length,
      }),
    );
    const user = userEvent.setup();
    await open();
    await heading();
    await waitFor(() => {
      expect(rowOf('200')).not.toBeNull();
    });
    const asked = () => vi.mocked(cashbookApi.lentDues).mock.calls.map(([input]) => input);
    // The people: from the first one, a page of 25. The entries: their own page, with one person only.
    expect(asked()).toContainEqual(expect.objectContaining({ people_offset: 0, people_limit: 25 }));
    expect(asked()).toContainEqual(
      expect.objectContaining({ people_offset: 0, people_limit: 1, limit: 25 }),
    );
    expect(rowOf('225')).toBeNull();
    // The next page of the people (the people's pager is the first one on the page).
    const next = screen.getAllByRole('button', {
      name: (name) => visible(name).includes(m('common-pager-next')),
    })[0];
    if (next === undefined) throw new Error('no next button');
    await user.click(next);
    await waitFor(() => {
      expect(asked()).toContainEqual(
        expect.objectContaining({ people_offset: 25, people_limit: 25 }),
      );
    });
    await waitFor(() => {
      expect(rowOf('225')).not.toBeNull();
    });
  });

  it('Page Down on the people turns their page, and the pager shows the page and the rows per page', async () => {
    const everyone = Array.from({ length: 60 }, (_, index) => ({
      person: {
        id: 300 + index,
        code: `0103${String(300 + index)}`,
        name_ur: `شخص ${String(index)}`,
        name_en: `Person ${String(index)}`,
      },
      person_type: 'other' as const,
      link: { kind: 'shop' as const, partner_id: null },
      lent_paisa: rupees(1_000),
      returned_paisa: rupees(0),
      open_paisa: rupees(1_000),
    }));
    vi.mocked(cashbookApi.lentDues).mockImplementation((input) =>
      Promise.resolve({
        ...DUES,
        people: everyone.slice(input.people_offset, input.people_offset + input.people_limit),
        people_total_count: everyone.length,
      }),
    );
    const user = userEvent.setup();
    await open();
    await heading();
    await waitFor(() => {
      expect(rowOf('300')).not.toBeNull();
    });
    // First page of 25 out of 60: "page 1 of 3", and the rows-per-page choice (25, 50, 100).
    expect(visible(document.body.textContent)).toContain(
      m('common-pager-page', { page: 1, pages: 3 }),
    );
    await user.click(rowOf('300')?.querySelector<HTMLElement>('[col-id]') ?? document.body);
    await user.keyboard('{PageDown}');
    await waitFor(() => {
      expect(
        vi
          .mocked(cashbookApi.lentDues)
          .mock.calls.some(([input]) => input.people_offset === 25 && input.people_limit === 25),
      ).toBe(true);
    });
    await waitFor(() => {
      expect(rowOf('325')).not.toBeNull();
    });
    expect(visible(document.body.textContent)).toContain(
      m('common-pager-page', { page: 2, pages: 3 }),
    );
  });

  it('the month buttons go to the month before, and it stops at the first month with an entry', async () => {
    const user = userEvent.setup();
    await open();
    await heading();
    await screen.findAllByText((text) => visible(text).includes('18,000'));
    // The later button is off on the current month; the earlier one goes to September, then August.
    expect((button('common-month-later') as HTMLButtonElement).disabled).toBe(true);
    await user.click(button('common-month-earlier'));
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({ month: '2026-09' });
    });
    await user.click(button('common-month-earlier'));
    await waitFor(() => {
      expect(lastAsked()).toMatchObject({ month: '2026-08' });
    });
    // August holds the first day of data (15-08-2026): there is nothing before it.
    await waitFor(() => {
      expect((button('common-month-earlier') as HTMLButtonElement).disabled).toBe(true);
    });
  });

  it('a row opens the person on the ادھار رقم screen, with his name chosen and no amount', async () => {
    const user = userEvent.setup();
    const { router } = await open();
    await heading();
    await screen.findAllByText((text) => visible(text).includes('18,000'));
    await waitFor(() => {
      expect(rowOf('given-303-2026-10-05')).not.toBeNull();
    });
    const row = rowOf('given-303-2026-10-05');
    if (row === null) throw new Error('no row of the trip money');
    expect(visible(row.textContent)).toContain('trip money');
    await user.dblClick(row.querySelector<HTMLElement>('[col-id]') ?? row);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/lent');
    });
    expect(router.state.location.search).toMatchObject({
      party: { id: 92, name_en: 'Shafiq' },
    });
  });

  it('a month with nothing says so, and the buttons still work', async () => {
    vi.mocked(cashbookApi.lentDues).mockResolvedValue({
      ...DUES,
      rows: [],
      total_count: 0,
      people: [],
      people_total_count: 0,
      totals: {
        given_paisa: rupees(0),
        returned_paisa: rupees(0),
        open_paisa: rupees(0),
        open_people_count: 0,
      },
    });
    await open();
    await heading();
    expect(await screen.findByText(m('cashbook-dues-empty'))).toBeInTheDocument();
    expect(await screen.findByText(m('cashbook-dues-people-empty'))).toBeInTheDocument();
    expect((button('common-month-earlier') as HTMLButtonElement).disabled).toBe(false);
  });

  it('a failure says so with a retry that works', async () => {
    vi.mocked(cashbookApi.lentDues).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    await open();
    await heading();
    const title = await screen.findByText(m('cashbook-dues-failed'));
    const alert = title.closest('[role="alert"]');
    if (!(alert instanceof HTMLElement)) throw new Error('no alert');
    await userEvent.setup().click(
      within(alert).getByRole('button', {
        name: (name) => visible(name).includes(m('common-retry')),
      }),
    );
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain('18,000');
    });
  });

  it('the dashboard tile shows the open amount and the people, and opens this page', async () => {
    const dashboard = (await import('../helpers/dashboard')).DASHBOARD_WORKED_DAY;
    fakeDashboard({
      dashboard: {
        ...dashboard,
        tiles: {
          ...dashboard.tiles,
          others_dues_paisa: rupees(15_000),
          others_people_count: 2,
        },
      },
    });
    const user = userEvent.setup();
    const { router } = await renderApp('/');
    const tile = await screen.findByRole('button', {
      name: (name) => visible(name).includes(m('dashboard-tile-others')),
    });
    expect(visible(tile.textContent)).toContain('15,000');
    expect(visible(tile.textContent)).toContain(
      m('dashboard-tile-others-note', { count: 2 }).replace(/\s+/g, ' '),
    );
    await user.click(tile);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/others-dues');
    });
    expect(dashboardApi.dashboardGet).toHaveBeenCalled();
  });
});
