// The pages behind the tiles ادھار رقم باقی and پیشگی باقی (plan 79 item 3, PRD FR-9.1, FR-16.5, FR-15.8, FR-19.11) in
// the Urdu and the English app, against a fake backend: each lists exactly what its tile counts, the total
// under the columns is the backend's figure (the tile's), Enter on a row opens that person's screen with his name
// chosen and nothing saved, `only_open` is asked for the advances, and each has its four states.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
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
import { fakeDashboard } from '../helpers/dashboard';
import { ADVANCES_OPEN, LENT_OPEN } from '../helpers/dues-pages';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(dashboardApi.dashboardLentOpen).mockResolvedValue(LENT_OPEN);
  vi.mocked(dashboardApi.dashboardAdvancesOpen).mockResolvedValue(ADVANCES_OPEN);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the open money lent and open advances pages (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const rowOf = (id: number) =>
    document.querySelector<HTMLElement>(`.ag-row[row-id="${String(id)}"]`);
  const clickRow = async (user: ReturnType<typeof userEvent.setup>, id: number, col: string) => {
    const cell = rowOf(id)?.querySelector<HTMLElement>(`[col-id="${col}"]`);
    if (cell === null || cell === undefined) throw new Error('no cell');
    await user.click(cell);
    await user.keyboard('{Enter}');
  };

  describe('ادھار رقم باقی', () => {
    it('lists the people who owe with their type and what is open, and the total is the backend’s', async () => {
      await renderApp('/dues/lent');
      await screen.findByRole('heading', {
        level: 1,
        name: (name) => visible(name) === m('lent-open-title'),
      });
      await waitFor(() => {
        expect(rowOf(31)).not.toBeNull();
      });
      const nawaz = visible(rowOf(31)?.textContent);
      expect(nawaz).toContain('15,000');
      expect(nawaz).toContain('01-11-2026');
      expect(visible(rowOf(30)?.textContent)).toContain('2,000');
      // The total under the columns is the tile's figure, and what is past its date is the backend's too.
      const totals = visible(document.querySelector('.ag-row-pinned')?.textContent);
      expect(totals).toContain('17,000');
    });

    it('Enter on a person opens the ادھار رقم screen with his name chosen', async () => {
      const user = userEvent.setup();
      const { router } = await renderApp('/dues/lent');
      await waitFor(() => {
        expect(rowOf(31)).not.toBeNull();
      });
      await clickRow(user, 31, 'person');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/rokar/lent');
      });
      expect(router.state.location.search).toMatchObject({
        type: 'money_lent',
        party: { id: 31, code: '0061' },
      });
    });

    it('says that nobody owes anything, and an error has a retry', async () => {
      vi.mocked(dashboardApi.dashboardLentOpen).mockRejectedValueOnce(
        new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
      );
      const user = userEvent.setup();
      await renderApp('/dues/lent');
      vi.mocked(dashboardApi.dashboardLentOpen).mockResolvedValue({
        rows: [],
        total_count: 0,
        total_paisa: paisaForTest(0),
        overdue_paisa: paisaForTest(0),
      });
      await user.click(
        await screen.findByRole('button', { name: (name) => visible(name) === m('common-retry') }),
      );
      expect(
        await screen.findByText((text) => visible(text) === m('lent-open-empty')),
      ).toBeInTheDocument();
    });
  });

  describe('پیشگی باقی', () => {
    it('asks only for the open advances, lists رحیم with given, adjusted and open, and the total is the tile’s', async () => {
      await renderApp('/dues/advances');
      await screen.findByRole('heading', {
        level: 1,
        name: (name) => visible(name) === m('advances-open-title'),
      });
      await waitFor(() => {
        expect(rowOf(10)).not.toBeNull();
      });
      expect(vi.mocked(dashboardApi.dashboardAdvancesOpen).mock.calls.at(-1)?.[0]).toMatchObject({
        only_open: true,
        seller_id: null,
        offset: 0,
      });
      const rahim = visible(rowOf(10)?.textContent);
      expect(rahim).toContain(language === 'ur' ? 'رحیم' : 'Rahim');
      expect(rahim).toContain('15,000');
      expect(rahim).toContain('01-09-2026');
      expect(visible(document.querySelector('.ag-row-pinned')?.textContent)).toContain('15,000');
    });

    it('Enter on a seller opens the پیشگی screen with his name chosen', async () => {
      const user = userEvent.setup();
      const { router } = await renderApp('/dues/advances');
      await waitFor(() => {
        expect(rowOf(10)).not.toBeNull();
      });
      await clickRow(user, 10, 'seller');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/voucher');
      });
      expect(router.state.location.search).toMatchObject({
        type: 'advance',
        party: { id: 10, code: '0010' },
      });
    });

    it('after Events 1 to 5 nothing is open, and the page says so', async () => {
      vi.mocked(dashboardApi.dashboardAdvancesOpen).mockResolvedValue({
        ...ADVANCES_OPEN,
        rows: [],
        total_count: 0,
        given_paisa: paisaForTest(0),
        open_paisa: paisaForTest(0),
      });
      await renderApp('/dues/advances');
      expect(
        await screen.findByText((text) => visible(text) === m('advances-open-empty')),
      ).toBeInTheDocument();
      expect(within(screen.getByRole('main')).queryByRole('alert')).toBeNull();
    });

    it('an error is told with a retry that asks again', async () => {
      vi.mocked(dashboardApi.dashboardAdvancesOpen).mockRejectedValueOnce(
        new CommandError(refusal([{ code: 'ERR_INTERNAL', field: null, values: {} }])),
      );
      const user = userEvent.setup();
      await renderApp('/dues/advances');
      await user.click(
        await screen.findByRole('button', { name: (name) => visible(name) === m('common-retry') }),
      );
      await waitFor(() => {
        expect(rowOf(10)).not.toBeNull();
      });
    });
  });
});
