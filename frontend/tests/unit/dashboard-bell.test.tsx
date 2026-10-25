// The attention bell in the top bar (round 9, PRD FR-7.44, FR-5.6, NFR-SPD-22): the number the
// backend counts, the list under it with each kind in words and opening its screen, the most urgent
// record opening itself, a new version as a note, the four states, by keyboard, and read again after
// every save. Urdu and English, against a fake backend.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AttentionBell } from '../../src/app/layout/AttentionBell';
import { createQueryClient } from '../../src/app/providers';
import { REFRESH_AFTER_SAVE_MS } from '../../src/app/refresh-after-save';
import * as dashboardApi from '../../src/features/dashboard/api';
import * as maintenanceApi from '../../src/features/settings/maintenance-api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { UPDATE_AVAILABLE, UPDATE_NONE } from '../helpers/b5-real';
import { ATTENTION_18_10, fakeDashboard } from '../helpers/dashboard';
import type { AttentionList } from '../../src/bindings/AttentionList';
import { message, renderIn, visible } from '../helpers/render';
import { must } from '../helpers/must';
import { paisaForTest } from '../helpers/paisa';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/settings/maintenance-api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeDashboard();
  vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_NONE);
});

afterEach(() => {
  leaveAppWindow();
});

const named =
  (language: Language, id: string, vars: Record<string, string | number> = {}) =>
  (name: string) =>
    visible(name) === message(language, id, vars);

async function openApp(language: Language, path = '/bikri/today') {
  resetClientState(language);
  signedIn(sessionUser({ language }));
  return renderApp(path);
}

/** The bell, by its label with the count it holds. */
async function bell(language: Language, count: number) {
  return screen.findByRole('button', {
    name: named(language, 'attention-bell-label', { count }),
  });
}

describe.each(LANGUAGES)('the bell (%s)', (language) => {
  it('wears the number the backend counts and opens a list of what to do', async () => {
    await openApp(language);
    const button = await bell(language, ATTENTION_18_10.total_count);
    expect(button).toHaveAttribute('data-attention-count', String(ATTENTION_18_10.total_count));
    await userEvent.setup().click(button);
    const list = await screen.findByRole('heading', { name: named(language, 'attention-title') });
    const panel = must(list.parentElement);
    // Three kinds of the fixture: 8 earlier days, a payable of 10,160, the backup a day old.
    const lines = within(panel)
      .getAllByRole('button')
      .filter((one) => one.hasAttribute('data-attention-kind'));
    expect(lines.map((line) => line.getAttribute('data-attention-kind'))).toEqual([
      'open_days',
      'payables_old',
      'backup_old',
    ]);
    expect(visible(lines[0]?.textContent)).toContain(
      message(language, 'attention-open-days', { count: 8 }),
    );
    expect(visible(lines[1]?.textContent)).toContain('10,160');
    expect(visible(lines[2]?.textContent)).toContain(
      message(language, 'attention-backup-old', { days: 1 }),
    );
  });

  it.each([
    ['open_days', '/day'],
    ['backup_old', '/backup'],
  ] as const)('the %s line opens %s and closes the list', async (kind, path) => {
    const { router } = await openApp(language);
    await userEvent.setup().click(await bell(language, ATTENTION_18_10.total_count));
    const line = await waitFor(() => {
      const found = document.querySelector(`[data-attention-kind="${kind}"]`);
      if (found === null) throw new Error('no line yet');
      return found;
    });
    await userEvent.setup().click(line);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
    expect(
      screen.queryByRole('heading', { name: named(language, 'attention-title') }),
    ).not.toBeInTheDocument();
  });

  it('the most urgent record is a second button that opens the record itself: the seller’s ledger', async () => {
    const { router } = await openApp(language);
    await userEvent.setup().click(await bell(language, ATTENTION_18_10.total_count));
    const first = await screen.findByRole('button', {
      name: (name) =>
        visible(name).startsWith(message(language, 'attention-open-first', { name: '' }).trim()),
    });
    expect(visible(first.textContent)).toContain(language === 'ur' ? 'کریم' : 'Karim');
    await userEvent.setup().click(first);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/reports/R13');
    });
    // The seller of the fixture, کریم, is account 19.
    expect(router.state.location.search).toMatchObject({ party: 19 });
  });

  it('works by keyboard: Enter opens the list, ↓ moves between its lines, Enter opens one, Esc closes it', async () => {
    const { router } = await openApp(language);
    const button = await bell(language, ATTENTION_18_10.total_count);
    const user = userEvent.setup();
    button.focus();
    await user.keyboard('{Enter}');
    await screen.findByRole('heading', { name: named(language, 'attention-title') });
    const first = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('[data-attention-kind="open_days"]');
      if (found === null) throw new Error('no line yet');
      return found;
    });
    first.focus();
    await user.keyboard('{ArrowDown}');
    expect(document.activeElement?.getAttribute('data-nav-item')).not.toBeNull();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: named(language, 'attention-title') }),
      ).not.toBeInTheDocument();
    });
    expect(router.state.location.pathname).toBe('/bikri/today');
  });

  it('says nothing needs attention when the list is empty, and the bell wears no number', async () => {
    fakeDashboard({ attention: { items: [], total_count: 0 } });
    await openApp(language);
    const button = await bell(language, 0);
    expect(button).toHaveAttribute('data-attention-count', '0');
    await userEvent.setup().click(button);
    expect(
      await screen.findByRole('heading', { name: named(language, 'attention-empty-title') }),
    ).toBeInTheDocument();
  });

  it('shows an error with a retry when the list cannot be read, and the retry works', async () => {
    fakeDashboard({ attention: new CommandError(refusal([{ code: 'ERR_INTERNAL' }])) });
    resetClientState(language);
    // The bell alone, so no other part of the app can take the focus from the list.
    renderIn(
      language,
      <QueryClientProvider client={createQueryClient()}>
        <AttentionBell />
      </QueryClientProvider>,
    );
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', {
        name: named(language, 'attention-bell-label', { count: 0 }),
      }),
    );
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'attention-load-failed'));
    expect(alert.textContent).not.toContain('ERR_');
    fakeDashboard();
    await user.click(
      within(alert).getByRole('button', { name: message(language, 'common-retry') }),
    );
    // The list is now the lines of the answer, in the same open list.
    await waitFor(() => {
      expect(document.querySelectorAll('[data-attention-kind]')).toHaveLength(3);
    });
  });

  it('a new version is one more line at the top, only a note; it opens the settings and installs nothing', async () => {
    vi.mocked(maintenanceApi.updatesCheck).mockResolvedValue(UPDATE_AVAILABLE);
    const { router } = await openApp(language);
    await userEvent.setup().click(await bell(language, ATTENTION_18_10.total_count));
    const note = await waitFor(() => {
      const found = document.querySelector('[data-update-note]');
      if (found === null) throw new Error('no note yet');
      return found;
    });
    expect(visible(note.textContent)).toContain(String(UPDATE_AVAILABLE.version));
    await userEvent.setup().click(note);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/settings');
    });
    expect(router.state.location.search).toMatchObject({ tab: 'app' });
    expect(maintenanceApi.updatesInstall).not.toHaveBeenCalled();
  });

  // Every kind of the list, each with the screen it opens (PRD FR-7.44) and the record it names.
  const ALL_KINDS: AttentionList = {
    items: [
      { kind: 'drafts', count: 2, amount_paisa: null, qty: null, days: null, first: null },
      { kind: 'open_days', count: 8, amount_paisa: null, qty: null, days: 8, first: null },
      { kind: 'stock_waiting', count: 1, amount_paisa: null, qty: 5, days: 3, first: null },
      {
        kind: 'over_credit_limit',
        count: 1,
        amount_paisa: null,
        qty: null,
        days: null,
        first: null,
      },
      {
        kind: 'dues_old',
        count: 1,
        amount_paisa: paisaForTest(288_000),
        qty: null,
        days: 41,
        first: null,
      },
      {
        kind: 'payables_old',
        count: 1,
        amount_paisa: paisaForTest(1_016_000),
        qty: null,
        days: null,
        first: null,
      },
      { kind: 'backup_old', count: 1, amount_paisa: null, qty: null, days: 2, first: null },
      {
        kind: 'loans_overdue',
        count: 1,
        amount_paisa: paisaForTest(200_000),
        qty: null,
        days: 1,
        first: null,
      },
      {
        kind: 'takes_overdue',
        count: 1,
        amount_paisa: paisaForTest(200_000),
        qty: null,
        days: 3,
        first: null,
      },
      { kind: 'names_missing', count: 4, amount_paisa: null, qty: null, days: null, first: null },
    ],
    total_count: 22,
  };

  it.each([
    ['drafts', '/bikri/book'],
    ['open_days', '/day'],
    ['stock_waiting', '/bikri/stock'],
    ['over_credit_limit', '/reports/R20'],
    ['dues_old', '/reports/R20'],
    ['payables_old', '/reports/R24'],
    ['backup_old', '/backup'],
    ['loans_overdue', '/reports/R32'],
    ['takes_overdue', '/rokar/expense'],
    ['names_missing', '/index'],
  ] as const)('the %s line says its words and opens %s', async (kind, path) => {
    fakeDashboard({ attention: ALL_KINDS });
    const { router } = await openApp(language);
    await userEvent.setup().click(await bell(language, ALL_KINDS.total_count));
    const line = await waitFor(() => {
      const found = document.querySelector(`[data-attention-kind="${kind}"]`);
      if (found === null) throw new Error('no line yet');
      return found;
    });
    // Words, never a raw message ID.
    expect(visible(line.textContent)).not.toMatch(/attention-|ERR_/);
    expect(visible(line.textContent).length).toBeGreaterThan(3);
    await userEvent.setup().click(line);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe(path);
    });
  });
});

describe('the bell and the dashboard are read again after every save', () => {
  it('a saved entry makes the bell, the dashboard and the gardens tile ask once, 300 ms after the last save', async () => {
    vi.useFakeTimers();
    try {
      const client: QueryClient = createQueryClient();
      const invalidate = vi.spyOn(client, 'invalidateQueries');
      const run = () =>
        client
          .getMutationCache()
          .build(client, { mutationFn: () => Promise.resolve(null) })
          .execute(undefined);
      await run();
      await run();
      // Two saves close together are one read.
      expect(invalidate).not.toHaveBeenCalled();
      await vi.advanceTimersByTimeAsync(REFRESH_AFTER_SAVE_MS + 10);
      // The dashboard, the bell and the gardens tile (a بکری of a garden's lot changes its balance).
      expect(invalidate.mock.calls.map((call) => call[0]?.queryKey)).toEqual([
        ['dashboard'],
        ['attention'],
        ['gardens', 'summary'],
      ]);
    } finally {
      vi.useRealTimers();
    }
  });

  it('a failed save reads nothing again', async () => {
    vi.useFakeTimers();
    try {
      const client = createQueryClient();
      const invalidate = vi.spyOn(client, 'invalidateQueries');
      await client
        .getMutationCache()
        .build(client, { mutationFn: () => Promise.reject(new Error('no')) })
        .execute(undefined)
        .catch(() => undefined);
      await vi.advanceTimersByTimeAsync(REFRESH_AFTER_SAVE_MS + 10);
      expect(
        invalidate.mock.calls.filter((call) => call[0]?.queryKey?.[0] === 'attention'),
      ).toHaveLength(0);
    } finally {
      vi.useRealTimers();
    }
  });

  it('the bell on screen asks again when a save happens anywhere', async () => {
    const client = createQueryClient();
    renderIn(
      'en',
      <QueryClientProvider client={client}>
        <AttentionBell />
      </QueryClientProvider>,
    );
    await screen.findByRole('button', {
      name: named('en', 'attention-bell-label', { count: ATTENTION_18_10.total_count }),
    });
    const asked = vi.mocked(dashboardApi.assistAttention).mock.calls.length;
    fakeDashboard({ attention: { items: [], total_count: 0 } });
    // Any command that writes goes through a mutation; one stands for them all.
    await client
      .getMutationCache()
      .build(client, { mutationFn: () => Promise.resolve(null) })
      .execute(undefined);
    await screen.findByRole('button', {
      name: named('en', 'attention-bell-label', { count: 0 }),
    });
    expect(vi.mocked(dashboardApi.assistAttention).mock.calls.length).toBeGreaterThan(asked);
  });
});
