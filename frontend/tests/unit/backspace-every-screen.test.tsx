// D8 Bug 2 (plan 62, T3): Backspace and Delete never open the "Save changes?" box and never change the
// route, on every entry screen, in Urdu and in English (PRD FR-7.91, NFR-USE-7). The web view goes back
// for a Backspace that nobody cancels (see backspace-delete.test.tsx), so for each screen the key is
// pressed on the page, on a button and on a grid cell, and each Backspace must be cancelled. The
// one listener that does it is mounted once in the shell; this proves every route sits under it.
//
// The commands of every feature answer with nothing here (their wrappers are mocked without answers),
// so each screen shows the state it has with no data. The entry itself, with unsaved work, is tested on
// the sale book in backspace-delete.test.tsx and on the shared grid in entry-grid.test.tsx.

import { fireEvent, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as analyticsApi from '../../src/features/analytics/api';
import * as assistantApi from '../../src/features/assistant/api';
import * as auditApi from '../../src/features/audit/api';
import * as backupApi from '../../src/features/backup/api';
import * as bikriApi from '../../src/features/bikri/api';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as dashboardApi from '../../src/features/dashboard/api';
import * as dayApi from '../../src/features/day/api';
import * as fruitApi from '../../src/features/fruit/api';
import * as gardensApi from '../../src/features/gardens/api';
import * as mastersApi from '../../src/features/masters/api';
import * as monthApi from '../../src/features/month/api';
import * as partnersApi from '../../src/features/partners/api';
import * as purchasesApi from '../../src/features/purchases/api';
import * as reportsApi from '../../src/features/reports/api';
import * as restoreApi from '../../src/features/restore/api';
import * as rulesApi from '../../src/features/rules/api';
import * as tripsApi from '../../src/features/trips/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES } from '../../src/i18n';
import { STATUS_ON } from '../helpers/assistant-real';
import * as accountsApi from '../../src/shared/accounts/api';
import * as draftsApi from '../../src/shared/drafts/api';
import * as editApi from '../../src/shared/edit/api';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';

vi.mock('../../src/app/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/assistant/api');
vi.mock('../../src/features/audit/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/day/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/features/month/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/restore/api');
vi.mock('../../src/features/rules/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** Every entry screen of PRD 7.67 and the forms of the Index, by its address. */
const SCREENS: readonly { name: string; path: string }[] = [
  { name: 'sale book', path: '/bikri/book' },
  { name: 'later sale', path: '/bikri/later?bikri=1&lot=1' },
  { name: 'voucher: receipt', path: '/bikri/voucher?type=receipt' },
  { name: 'voucher: payment', path: '/bikri/voucher?type=payment' },
  { name: 'voucher: advance', path: '/bikri/voucher?type=advance' },
  { name: 'expense', path: '/rokar/expense' },
  { name: 'fruit taken', path: '/rokar/fruit' },
  { name: 'money lent', path: '/rokar/lent' },
  { name: 'separate khata', path: '/rokar/khata' },
  { name: 'journal', path: '/rokar/general' },
  { name: 'partner capital', path: '/partners/capital' },
  { name: 'new trip', path: '/trips/new' },
  { name: 'a trip', path: '/trips/1' },
  { name: 'new garden', path: '/gardens/new' },
  { name: 'a garden', path: '/gardens/1' },
  { name: 'new account', path: '/index/accounts/new' },
  { name: 'an account', path: '/index/accounts/1' },
  { name: 'opening balances', path: '/index/openings' },
  { name: 'new item', path: '/index/item/new' },
  { name: 'new city', path: '/index/city/new' },
  { name: 'new commission rule', path: '/index/rules/new' },
  { name: 'partner shares', path: '/partners/setup' },
  { name: 'settings', path: '/settings' },
];

/** Every command of these features fails with no answer, so each screen shows its error state, and no screen calls `.then` on nothing. */
const FEATURE_APIS: readonly Record<string, unknown>[] = [
  analyticsApi,
  assistantApi,
  auditApi,
  backupApi,
  bikriApi,
  cashbookApi,
  dashboardApi,
  dayApi,
  fruitApi,
  gardensApi,
  mastersApi,
  monthApi,
  partnersApi,
  purchasesApi,
  reportsApi,
  restoreApi,
  rulesApi,
  tripsApi,
  vouchersApi,
  accountsApi,
  draftsApi,
  editApi,
  searchApi,
  urduApi,
];

beforeEach(() => {
  vi.resetAllMocks();
  for (const feature of FEATURE_APIS) {
    for (const command of Object.values(feature)) {
      if (vi.isMockFunction(command)) command.mockRejectedValue(new Error('no answer'));
    }
  }
  healthyBackend();
  // The two commands the shell and the partner gate always ask: the attention list with nothing in it,
  // and a partner code that is not made yet.
  vi.mocked(dashboardApi.assistAttention).mockResolvedValue({
    items: [],
    total_count: 0,
    partners_locked: false,
  });
  vi.mocked(assistantApi.assistantStatus).mockResolvedValue(STATUS_ON);
  vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue({
    is_set: false,
    unlocked: false,
    wait_seconds: 0,
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('Backspace and Delete on every entry screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it.each(SCREENS)(
    '$name: never cancelled wrongly, never a box, never another route',
    async (screenOf) => {
      const { router } = await renderApp(screenOf.path);
      // The page of the route is loaded when its heading or its buttons are there.
      await waitFor(() => {
        expect(document.querySelector('main')?.textContent.length ?? 0).toBeGreaterThan(0);
      });
      await new Promise((done) => setTimeout(done, 300));
      const where = router.state.location.pathname;
      const targets: Element[] = [
        document.body,
        ...Array.from(document.querySelectorAll<HTMLElement>('main button')).slice(0, 3),
        ...Array.from(document.querySelectorAll<HTMLElement>('.ag-cell')).slice(0, 3),
      ];
      for (const target of targets) {
        if (target instanceof HTMLElement && target !== document.body) target.focus();
        // fireEvent gives back false when the key press was cancelled: the web view then cannot go back.
        const backspace = fireEvent.keyDown(target, { key: 'Backspace', code: 'Backspace' });
        const field = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
        expect(backspace).toBe(field);
        fireEvent.keyDown(target, { key: 'Delete', code: 'Delete' });
      }
      expect(screen.queryByRole('alertdialog')).toBeNull();
      expect(router.state.location.pathname).toBe(where);
      expect(where).toBe(screenOf.path.split('?')[0]);
    },
  );
});
