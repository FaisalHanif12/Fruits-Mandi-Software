// The four dues tiles of the dashboard (plan 40, PRD 9.1, FR-19.9, FR-10.10, E2E-44): گاہکوں کے ذمے,
// بیوپاریوں کے ذمے, کل اگراہی and ڈوبی رقم, with the other tiles renamed to say exactly what they
// are. The numbers are the backend's. After ارشد's move the buyers' dues fall to 2,880 and the dead
// amount shows 18,684 for one person, and the total اگراہی is still 21,564; after the 4,000 it is
// 17,564. The dead tile opens R38. In the Urdu and in the English app.

import { waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Dashboard } from '../../src/bindings/Dashboard';
import * as backupApi from '../../src/features/backup/api';
import * as maintenanceApi from '../../src/features/settings/maintenance-api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { STATUS_AFTER } from '../helpers/backup-real';
import { HEALTH_LAST, UPDATE_NONE } from '../helpers/b5-real';
import { DASHBOARD_WORKED_DAY, fakeDashboard } from '../helpers/dashboard';
import { TILES_AFTER_MOVE, TILES_AFTER_RECOVERY } from '../helpers/dead-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/features/settings/maintenance-api');

/** The worked day after Events 1 to 5 with ارشد in the dead amount (E2E-44, before his son pays). */
const AFTER_MOVE: Dashboard = {
  ...DASHBOARD_WORKED_DAY,
  tiles: { ...DASHBOARD_WORKED_DAY.tiles, ...TILES_AFTER_MOVE },
};

/** After the 4,000 of 12-10-26: the dead amount is 14,684 and the total اگراہی 17,564. */
const AFTER_RECOVERY: Dashboard = {
  ...DASHBOARD_WORKED_DAY,
  tiles: { ...DASHBOARD_WORKED_DAY.tiles, ...TILES_AFTER_RECOVERY },
};

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(backupApi.backupStatus).mockResolvedValue(STATUS_AFTER);
  vi.mocked(maintenanceApi.healthLast).mockResolvedValue({
    ...HEALTH_LAST,
    checked_at: '2026-10-10T09:05:00Z',
  });
  vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_NONE);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the dues tiles (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);

  /** The button of a tile, found by its label, among the buttons of the screen (not of the menu). */
  function tile(labelId: string): HTMLElement {
    const label = m(labelId);
    const found = Array.from(
      document.querySelectorAll<HTMLElement>('main button[data-nav-field]'),
    ).find((button) => visible(button.textContent).startsWith(label));
    if (found === undefined) throw new Error(`no tile "${label}"`);
    return found;
  }
  const text = (labelId: string) => visible(tile(labelId).textContent);

  async function open() {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    const view = await renderApp('/');
    await waitFor(() => {
      tile('dashboard-tile-sales');
    });
    return view;
  }

  it('has the four dues tiles in the order of PRD 9.1, with the exact words', async () => {
    await open();
    const labels = Array.from(
      document.querySelectorAll<HTMLElement>('main button[data-nav-field]'),
    ).map((button) => visible(button.querySelector('span')?.textContent));
    const wanted = [
      'dashboard-tile-cash',
      'dashboard-tile-dues',
      'dashboard-tile-dues-sellers',
      'dashboard-tile-dues-total',
      'dashboard-tile-dead',
      'dashboard-tile-payable',
      'dashboard-tile-advances',
    ].map((id) => m(id));
    const at = labels.indexOf(wanted[0] ?? '');
    expect(labels.slice(at, at + wanted.length)).toEqual(wanted);
    // The words themselves, as PRD 9.1 fixes them.
    expect(wanted).toEqual(
      language === 'ur'
        ? [
            'موجود روکڑ',
            'گاہکوں کے ذمے',
            'بیوپاریوں کے ذمے',
            'کل اگراہی',
            'ڈوبی رقم',
            'بیوپاریوں کو ادائیگی باقی',
            'پیشگی باقی',
          ]
        : [
            'Cash in hand',
            "Buyers' dues",
            "Sellers' dues",
            'Total Dues',
            'Dead Amount',
            'Sellers payable',
            'Advances open',
          ],
    );
  });

  it('nobody is dead: the dead tile is 0 and says nobody, and the total is the buyers’ dues', async () => {
    await open();
    expect(text('dashboard-tile-dues')).toContain('21,564');
    expect(text('dashboard-tile-dues-sellers')).toContain('0');
    expect(text('dashboard-tile-dues-total')).toContain('21,564');
    expect(text('dashboard-tile-dead')).toContain('0');
    expect(text('dashboard-tile-dead')).toContain(m('dashboard-tile-dead-none'));
  });

  it('after the move: buyers 2,880, dead 18,684 for one person, total still 21,564 (E2E-44)', async () => {
    fakeDashboard({ dashboard: AFTER_MOVE });
    await open();
    expect(text('dashboard-tile-dues')).toContain('2,880');
    expect(text('dashboard-tile-dues-sellers')).toContain('0');
    expect(text('dashboard-tile-dead')).toContain('18,684');
    expect(text('dashboard-tile-dead')).toContain(m('dashboard-tile-dead-note', { count: 1 }));
    expect(text('dashboard-tile-dues-total')).toContain('21,564');
  });

  it('after the 4,000: dead 14,684, total 17,564', async () => {
    fakeDashboard({ dashboard: AFTER_RECOVERY });
    await open();
    expect(text('dashboard-tile-dead')).toContain('14,684');
    expect(text('dashboard-tile-dues-total')).toContain('17,564');
    expect(text('dashboard-tile-dues')).toContain('2,880');
  });

  it('two people are counted in words', async () => {
    fakeDashboard({
      dashboard: { ...AFTER_MOVE, tiles: { ...AFTER_MOVE.tiles, dead_count: 2 } },
    });
    await open();
    expect(text('dashboard-tile-dead')).toContain(m('dashboard-tile-dead-note', { count: 2 }));
  });

  const OPENS: readonly [string, string][] = [
    ['dashboard-tile-dues', '/reports/R20'],
    ['dashboard-tile-dues-sellers', '/reports/R24'],
    ['dashboard-tile-dues-total', '/reports/R20'],
    ['dashboard-tile-dead', '/reports/dead-amount'],
    ['dashboard-tile-payable', '/reports/R24'],
  ];
  it.each(OPENS)('the tile %s opens %s', async (labelId, path) => {
    const { router } = await open();
    await userEvent.setup().click(tile(labelId));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
  });
});
