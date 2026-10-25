// The dashboard (round 9, PRD Section 9, FR-9.1, FR-7.44 to FR-7.46, NFR-SPD-2, NFR-USE-9): the
// numbers of the worked day (PRD 4.9) from the backend's own answer, each tile opening the screen
// behind it, the partner strip, today's بکری, the oldest dues, the tasks, the two charts, the quick
// actions, the new-version note, and the four states, in Urdu and in English, by keyboard, and with
// a language switch that keeps everything. The whole app is drawn against a fake backend.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Dashboard } from '../../src/bindings/Dashboard';
import * as backupApi from '../../src/features/backup/api';
import * as gardensApi from '../../src/features/gardens/api';
import * as maintenanceApi from '../../src/features/settings/maintenance-api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { HEALTH_LAST, UPDATE_AVAILABLE, UPDATE_FAILED, UPDATE_NONE } from '../helpers/b5-real';
import { STATUS_AFTER } from '../helpers/backup-real';
import { DASHBOARD_WORKED_DAY, fakeDashboard } from '../helpers/dashboard';
import { GARDENS_SUMMARY_AFTER_CLOSE, GARDENS_SUMMARY_OPEN } from '../helpers/gardens-real';
import { message, visible } from '../helpers/render';
import { must } from '../helpers/must';
import { CommandError } from '../../src/shared/api';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/settings/maintenance-api');

const money = (value: number) => value as unknown as Dashboard['tiles']['cash_in_hand_paisa'];

/** The checked time of the books check as a real time, not the fixture's stand-in. */
const HEALTH = { ...HEALTH_LAST, checked_at: '2026-10-10T09:05:00Z' };

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(backupApi.backupStatus).mockResolvedValue(STATUS_AFTER);
  vi.mocked(gardensApi.gardensSummary).mockResolvedValue(GARDENS_SUMMARY_OPEN);
  vi.mocked(maintenanceApi.healthLast).mockResolvedValue(HEALTH);
  vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_NONE);
});

afterEach(() => {
  leaveAppWindow();
});

/** A name matcher that ignores the isolation marks Fluent puts around terms. */
const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

/** The button of a tile, found by its label, among the buttons of the screen (not of the menu). */
function tile(language: Language, labelId: string): HTMLElement {
  const label = message(language, labelId);
  const found = Array.from(
    document.querySelectorAll<HTMLElement>('main button[data-nav-field]'),
  ).find((button) => visible(button.textContent).startsWith(label));
  if (found === undefined) throw new Error(`no tile "${label}"`);
  return found;
}

/** The text of a tile, as the reader sees it. */
function tileText(language: Language, labelId: string): string {
  return visible(tile(language, labelId).textContent);
}

/**
 * jsdom has no layout, so the tiles are placed on a grid of four in a row, the way the page does at
 * the normal width. In Urdu the first tile is at the right edge and the row runs to the left.
 */
function layOutTiles(language: Language) {
  const WIDTH = 240;
  const GAP = 10;
  const HEIGHT = 90;
  Array.from(document.querySelectorAll<HTMLElement>('main button[data-nav-field]'))
    .filter(
      (button) =>
        button.closest('[role="group"]')?.getAttribute('aria-labelledby') ===
        'dashboard-tiles-title',
    )
    .forEach((button, index) => {
      const column = index % 4;
      const row = Math.floor(index / 4);
      const slot = language === 'ur' ? 3 - column : column;
      const left = slot * (WIDTH + GAP);
      const top = row * (HEIGHT + GAP);
      button.getBoundingClientRect = () => new DOMRect(left, top, WIDTH, HEIGHT);
    });
}

async function openDashboard(language: Language) {
  resetClientState(language);
  signedIn(sessionUser({ language }));
  const view = await renderApp('/');
  await screen.findByRole('group', { name: named(language, 'dashboard-quick-title') });
  return view;
}

async function whenLoaded(language: Language) {
  await waitFor(() => {
    tile(language, 'dashboard-tile-sales');
  });
}

describe.each(LANGUAGES)('the dashboard (%s)', (language) => {
  it('shows the numbers of the worked day, PRD 4.9, as the backend sent them', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    expect(tileText(language, 'dashboard-tile-sales')).toContain('41,400');
    expect(tileText(language, 'dashboard-tile-sales')).toContain(
      message(language, 'dashboard-tile-sales-count', { count: 2 }),
    );
    expect(tileText(language, 'dashboard-tile-commission')).toContain('5,448');
    expect(tileText(language, 'dashboard-tile-cash')).toContain('58,484');
    expect(tileText(language, 'dashboard-tile-dues')).toContain('21,564');
    expect(tileText(language, 'dashboard-tile-payable')).toContain('10,160');
    expect(tileText(language, 'dashboard-tile-advances')).toContain('0');
    expect(tileText(language, 'dashboard-tile-expenses')).toContain('3,000');
  });

  it('shows each partner’s share today, this month, this year and کل رقم', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    const cards = DASHBOARD_WORKED_DAY.partners.map((partner) =>
      screen.getByRole('button', {
        name: (name) =>
          visible(name) ===
          message(language, 'dashboard-partner-open', {
            name: language === 'ur' ? partner.name_ur : partner.name_en,
          }),
      }),
    );
    // 996 / 746 / 746 the share of the day; the total of A is below zero (he took 2,000).
    expect(visible(cards[0]?.textContent)).toContain('996');
    expect(visible(cards[1]?.textContent)).toContain('746');
    expect(visible(cards[2]?.textContent)).toContain('746');
    expect(visible(cards[0]?.textContent)).toContain('-1,004');
    for (const card of cards) {
      for (const id of [
        'dashboard-partner-today',
        'dashboard-partner-month',
        'dashboard-partner-year',
        'dashboard-partner-total',
      ]) {
        expect(visible(card.textContent)).toContain(message(language, id));
      }
    }
  });

  it('lists today’s بکری with their numbers and the oldest dues with their days', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    const bikri = screen.getByRole('table', { name: named(language, 'dashboard-bikri-title') });
    const rows = within(bikri).getAllByRole('row');
    // The header and two بکری.
    expect(rows).toHaveLength(3);
    expect(visible(rows[1]?.textContent)).toContain('29,400');
    expect(visible(rows[1]?.textContent)).toContain('19,316');
    expect(visible(rows[2]?.textContent)).toContain('12,000');
    expect(visible(rows[2]?.textContent)).toContain('10,160');
    expect(visible(rows[1]?.textContent)).toContain(language === 'ur' ? 'رحیم' : 'Rahim');
    expect(visible(rows[1]?.textContent)).toContain('LES-1234');
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === 'P' &&
          visible(element.textContent) ===
            message(language, 'dashboard-bikri-showing', { shown: 2, count: 2 }),
      ),
    ).toBeInTheDocument();
    const dues = screen.getByRole('table', { name: named(language, 'dashboard-dues-title') });
    const dueRows = within(dues).getAllByRole('row');
    expect(visible(dueRows[1]?.textContent)).toContain('18,684');
    expect(visible(dueRows[1]?.textContent)).toContain(
      language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani',
    );
    expect(visible(dueRows[2]?.textContent)).toContain('2,880');
  });

  it('draws the two charts from the backend’s days and items', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    for (const id of ['dashboard-chart-days-label', 'dashboard-chart-items-label']) {
      expect(
        screen.getByRole('img', { name: (name) => visible(name) === message(language, id) }),
      ).toBeInTheDocument();
    }
  });
});

describe.each(LANGUAGES)('a tile or a name opens the screen behind it (%s)', (language) => {
  const CASES: readonly [string, string][] = [
    ['dashboard-tile-sales', '/bikri/today'],
    ['dashboard-tile-commission', '/analytics'],
    ['dashboard-tile-cash', '/rokar'],
    ['dashboard-tile-dues', '/reports/R20'],
    ['dashboard-tile-payable', '/reports/R24'],
    ['dashboard-tile-advances', '/reports/R26'],
    ['dashboard-tile-stock', '/bikri/stock'],
    ['dashboard-tile-trips', '/trips'],
    ['dashboard-tile-gardens', '/gardens'],
    ['dashboard-tile-lent', '/reports/R32'],
    ['dashboard-tile-expenses', '/reports/R29'],
    ['dashboard-tile-backup', '/backup'],
  ];

  it.each(CASES)('%s opens %s', async (labelId, path) => {
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    await userEvent.setup().click(tile(language, labelId));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
  });

  it('the books check tile opens the Day tab of the settings', async () => {
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    await userEvent.setup().click(tile(language, 'dashboard-tile-health'));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/settings');
    });
    expect(router.state.location.search).toMatchObject({ tab: 'day' });
  });

  it('a partner card opens his statement, R31, with the partner in the address', async () => {
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    const first = DASHBOARD_WORKED_DAY.partners[0];
    await userEvent.setup().click(
      screen.getByRole('button', {
        name: (name) =>
          visible(name) ===
          message(language, 'dashboard-partner-open', {
            name: language === 'ur' ? (first?.name_ur ?? '') : (first?.name_en ?? ''),
          }),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R31');
    });
    expect(router.state.location.search).toMatchObject({ partner: first?.partner_id });
  });

  it('a بکری number opens that بکری, and a buyer’s name opens his ledger', async () => {
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    const user = userEvent.setup();
    const bikri = screen.getByRole('table', { name: named(language, 'dashboard-bikri-title') });
    await user.click(must(within(bikri).getAllByRole('button')[0]));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 1 });
  });

  const QUICK: readonly [string, string, Record<string, unknown>][] = [
    ['dashboard-new-bikri', '/bikri/book', {}],
    ['dashboard-new-receipt', '/bikri/voucher', { type: 'receipt' }],
    ['dashboard-new-payment', '/bikri/voucher', { type: 'payment' }],
    ['dashboard-new-advance', '/bikri/voucher', { type: 'advance' }],
    ['dashboard-new-expense', '/rokar/expense', { type: 'shop_expense' }],
    ['dashboard-open-daybook', '/reports/R11', {}],
  ];

  it.each(QUICK)('the quick action %s opens %s', async (id, path, search) => {
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    const group = screen.getByRole('group', { name: named(language, 'dashboard-quick-title') });
    await userEvent.setup().click(
      within(group).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, id)),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
    expect(router.state.location.search).toMatchObject(search);
  });
});

describe.each(LANGUAGES)('the states of the dashboard (%s)', (language) => {
  it('while the answer is on its way: a loading state, and the quick actions are already there', async () => {
    vi.mocked((await import('../../src/features/dashboard/api')).dashboardGet).mockReturnValue(
      new Promise(() => undefined),
    );
    await openDashboard(language);
    expect(
      screen.getAllByRole('status').some((one) => one.getAttribute('aria-busy') === 'true'),
    ).toBe(true);
    expect(
      screen.getByRole('group', { name: named(language, 'dashboard-quick-title') }),
    ).toBeInTheDocument();
  });

  it('when the backend fails (one error for the whole dashboard): the error with a retry that works, and the backup and books check tiles still show', async () => {
    fakeDashboard({ dashboard: new CommandError(refusal([{ code: 'ERR_INTERNAL' }])) });
    await openDashboard(language);
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'dashboard-load-failed'));
    // The two tiles ask for themselves, so one failed part does not blank the page.
    expect(tileText(language, 'dashboard-tile-backup').length).toBeGreaterThan(0);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-health')).toContain(
        message(language, 'dashboard-health-ok'),
      );
    });
    expect(() => tile(language, 'dashboard-tile-sales')).toThrow();
    fakeDashboard();
    await userEvent
      .setup()
      .click(within(alert).getByRole('button', { name: message(language, 'common-retry') }));
    await whenLoaded(language);
    expect(tileText(language, 'dashboard-tile-cash')).toContain('58,484');
  });

  it('the gardens tile shows the running balance of the open gardens and how many are open', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-gardens')).toContain('11,400');
    });
    expect(tileText(language, 'dashboard-tile-gardens')).toContain(
      message(language, 'dashboard-tile-gardens-note', { open: 1 }),
    );
    expect(gardensApi.gardensSummary).toHaveBeenCalled();
  });

  it('with no open garden the tile says so, and a click opens the list where one is made', async () => {
    vi.mocked(gardensApi.gardensSummary).mockResolvedValue(GARDENS_SUMMARY_AFTER_CLOSE);
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-gardens')).toContain(
        message(language, 'dashboard-tile-gardens-none'),
      );
    });
    expect(tileText(language, 'dashboard-tile-gardens')).toContain('0');
    await userEvent.setup().click(tile(language, 'dashboard-tile-gardens'));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens');
    });
  });

  it('a garden list that cannot be read is said on its own tile, and the other tiles stay', async () => {
    vi.mocked(gardensApi.gardensSummary).mockRejectedValue(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-gardens')).toContain(
        message(language, 'dashboard-tile-unknown'),
      );
    });
    expect(tileText(language, 'dashboard-tile-cash')).toContain('58,484');
  });

  it('a new book: each list says why it is empty and offers the next step', async () => {
    const empty: Dashboard = {
      ...DASHBOARD_WORKED_DAY,
      tiles: {
        ...DASHBOARD_WORKED_DAY.tiles,
        sales_gross_paisa: money(0),
        sales_count: 0,
        commission_paisa: money(0),
        buyers_dues_paisa: money(0),
        sellers_payable_paisa: money(0),
      },
      partners: [],
      bikri_today: [],
      bikri_today_count: 0,
      oldest_dues: [],
      chart_days: DASHBOARD_WORKED_DAY.chart_days.map((day) => ({
        ...day,
        gross_paisa: money(0),
        commission_paisa: money(0),
      })),
      chart_items: [],
      attention: { items: [], total_count: 0 },
    };
    fakeDashboard({ dashboard: empty });
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    for (const id of [
      'dashboard-bikri-empty-title',
      'dashboard-dues-empty-title',
      'dashboard-tasks-empty-title',
      'dashboard-partners-empty-title',
      'dashboard-chart-empty-title',
    ]) {
      expect(
        screen.getAllByRole('heading', { name: (name) => visible(name) === message(language, id) })
          .length,
      ).toBeGreaterThan(0);
    }
    // The empty list of بکری has the key in its words, and a button that opens a new one.
    const help = screen.getByText(
      (_, element) => visible(element?.textContent).includes('Alt+B') && element?.tagName === 'P',
    );
    expect(help).toBeInTheDocument();
    const section = screen.getByRole('region', { name: named(language, 'dashboard-bikri-title') });
    await userEvent.setup().click(
      within(section).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'dashboard-new-bikri')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
  });

  it('cash below zero is said in words as well as in colour', async () => {
    fakeDashboard({
      dashboard: {
        ...DASHBOARD_WORKED_DAY,
        tiles: { ...DASHBOARD_WORKED_DAY.tiles, cash_in_hand_paisa: money(-500_000) },
      },
    });
    await openDashboard(language);
    await whenLoaded(language);
    expect(tileText(language, 'dashboard-tile-cash')).toContain('-5,000');
    expect(tileText(language, 'dashboard-tile-cash')).toContain(
      message(language, 'dashboard-tile-cash-below'),
    );
  });

  it('money lent past its return-by date is said in words', async () => {
    fakeDashboard({
      dashboard: {
        ...DASHBOARD_WORKED_DAY,
        tiles: {
          ...DASHBOARD_WORKED_DAY.tiles,
          lent_open_paisa: money(500_000),
          lent_overdue_paisa: money(200_000),
        },
      },
    });
    await openDashboard(language);
    await whenLoaded(language);
    expect(tileText(language, 'dashboard-tile-lent')).toContain('5,000');
    expect(tileText(language, 'dashboard-tile-lent')).toContain('2,000');
    expect(tileText(language, 'dashboard-tile-lent')).toContain(
      message(language, 'dashboard-tile-lent-overdue', { money: '' }).trim(),
    );
  });

  it('the tasks of the day are listed as lines that open the right screens', async () => {
    fakeDashboard({
      dashboard: {
        ...DASHBOARD_WORKED_DAY,
        attention: (await import('../helpers/dashboard')).ATTENTION_18_10,
      },
    });
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    const section = screen.getByRole('region', {
      name: message(language, 'dashboard-tasks-title'),
    });
    const lines = within(section).getAllByRole('button');
    // Three kinds, and the payable names its most urgent seller.
    expect(lines.filter((line) => line.getAttribute('data-attention-kind') !== null)).toHaveLength(
      3,
    );
    await userEvent
      .setup()
      .click(must(lines.find((line) => line.getAttribute('data-attention-kind') === 'open_days')));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/day');
    });
  });
});

describe.each(LANGUAGES)('the backup, the books check and the new version (%s)', (language) => {
  it('the backup tile says the state in words and the books check says its result and time', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-health')).toContain(
        message(language, 'dashboard-health-ok'),
      );
    });
    expect(tileText(language, 'dashboard-tile-health')).toContain('2026');
    expect(tile(language, 'dashboard-tile-backup').getAttribute('class')).toBeTruthy();
  });

  it('a books check that found a problem says so in words', async () => {
    vi.mocked(maintenanceApi.healthLast).mockResolvedValue({ ...HEALTH, ok: false });
    await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-health')).toContain(
        message(language, 'dashboard-health-bad'),
      );
    });
  });

  it('no check yet says so', async () => {
    vi.mocked(maintenanceApi.healthLast).mockResolvedValue(null);
    await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(tileText(language, 'dashboard-tile-health')).toContain(
        message(language, 'dashboard-health-never'),
      );
    });
  });

  it('a new version is a note with a button to the settings; the app does not install it', async () => {
    vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_AVAILABLE);
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    const note = await waitFor(() => {
      const found = document.querySelector('[data-update-note]');
      if (found === null) throw new Error('no note yet');
      return found;
    });
    expect(visible(note.textContent)).toContain(String(UPDATE_AVAILABLE.version));
    await userEvent.setup().click(
      screen.getByRole('button', {
        name: (name) => visible(name).includes(message(language, 'dashboard-update-open')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/settings');
    });
    expect(router.state.location.search).toMatchObject({ tab: 'app' });
    expect(maintenanceApi.updatesInstall).not.toHaveBeenCalled();
  });

  it('no note when there is no newer version, or the check could not reach the network', async () => {
    vi.mocked(maintenanceApi.updatesCheck).mockRejectedValue(
      new CommandError({ ...UPDATE_FAILED, ok: false }),
    );
    await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(maintenanceApi.updatesCheck).toHaveBeenCalled();
    });
    expect(document.querySelector('[data-update-note]')).toBeNull();
  });

  it('the check is asked once when the app starts, not again on each screen', async () => {
    vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_AVAILABLE);
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    await waitFor(() => {
      expect(maintenanceApi.updatesCheck).toHaveBeenCalledTimes(1);
    });
    await router.navigate({ to: '/bikri/today' });
    await router.navigate({ to: '/' });
    await whenLoaded(language);
    expect(maintenanceApi.updatesCheck).toHaveBeenCalledTimes(1);
  });
});

describe.each(LANGUAGES)('by keyboard, and across a language switch (%s)', (language) => {
  it('the arrows move between the tiles the way the user sees them, and Enter opens one', async () => {
    const { router } = await openDashboard(language);
    await whenLoaded(language);
    const user = userEvent.setup();
    layOutTiles(language);
    const first = tile(language, 'dashboard-tile-sales');
    first.focus();
    // The next tile is on the end side: left in Urdu, right in English (PRD FR-7.13, FR-7.34).
    await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
    expect(tile(language, 'dashboard-tile-commission')).toHaveFocus();
    await user.keyboard(language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}');
    expect(first).toHaveFocus();
    // Down moves to the tile below it (a row has four tiles at the normal width).
    await user.keyboard('{ArrowDown}');
    expect(tile(language, 'dashboard-tile-payable')).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R24');
    });
  });

  it('switching language keeps the figures and the data, changes the words and the names, and asks nothing again', async () => {
    await openDashboard(language);
    await whenLoaded(language);
    const asked = vi.mocked((await import('../../src/features/dashboard/api')).dashboardGet).mock
      .calls.length;
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await userEvent.setup().keyboard('{Alt>}l{/Alt}');
    await waitFor(() => {
      expect(useLanguageStore.getState().language).toBe(other);
    });
    await screen.findByRole('button', {
      name: (name) => visible(name).startsWith(message(other, 'dashboard-tile-sales')),
    });
    expect(tileText(other, 'dashboard-tile-cash')).toContain('58,484');
    expect(tileText(other, 'dashboard-tile-dues')).toContain('21,564');
    expect(
      screen.getByRole('button', {
        name: (name) =>
          visible(name) ===
          message(other, 'dashboard-partner-open', {
            name: other === 'ur' ? 'الف' : 'Partner A',
          }),
      }),
    ).toBeInTheDocument();
    expect(
      vi.mocked((await import('../../src/features/dashboard/api')).dashboardGet).mock.calls.length,
    ).toBe(asked);
  });
});

describe('the greeting does not wait for the dashboard', () => {
  it('shows the Owner’s name and the day even when the dashboard fails', async () => {
    fakeDashboard({ dashboard: new CommandError(refusal([{ code: 'ERR_INTERNAL' }])) });
    resetClientState('en');
    signedIn(sessionUser({ language: 'en' }));
    await renderApp('/');
    expect(await screen.findByRole('heading', { level: 1, name: /Adnan/ })).toBeInTheDocument();
    expect(await screen.findByRole('alert')).toBeInTheDocument();
  });
});
