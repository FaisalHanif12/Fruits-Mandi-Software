// تبدیل کریں and منسوخ کریں on the rows of پچھلی بکری (plan 72, PRD FR-7.70, FR-21.4, FR-12.46), in the Urdu and
// the English app, against a fake backend. The two buttons above each part work on the row that has focus
// in it: they are off, with a hint, while no row has focus; Edit opens the بکری on the sale book straight in
// edit mode with the way back kept; a row of a closed day reads "reopen the day and fix" and goes through
// FR-21.4; a row that cannot be changed says why in words; Cancel asks the reason and cancels without
// leaving the list, which then reads the backend again. Keyboard only works too. Today in these tests is
// 10-10-2026, so the window is 03-10-2026 to 09-10-2026. The flags are the rows': the screen follows them.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { listRow, savedEvent1 } from '../helpers/bikri-saved';
import { gridOf, named } from '../helpers/bikri-screen';
import { DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { paisaForTest } from '../helpers/paisa';
import { pastAnswer, type PastFake } from '../helpers/past';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

/** A بکری of 09-10-2026 in the window, and Event 1 as today's. */
const WINDOW_ROW = listRow({ id: 61, number: 2, entry_date: '2026-10-09' });
const CLOSED_ROW = listRow({
  id: 62,
  number: 3,
  entry_date: '2026-10-08',
  needs_reopen: true,
});
const LOCKED_ROW = listRow({
  id: 63,
  number: 4,
  entry_date: '2026-10-07',
  can_edit: false,
  can_cancel: false,
  locked_reason: 'goods_moved',
});

function fakeRows(fake: PastFake = {}) {
  vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
    Promise.resolve(pastAnswer(input, fake)),
  );
}

/** The posting list of a بکری opened by `bikriGet`: one posting, 7, as in the other edit tests. */
function fakeDetail(id: number, overrides: Parameters<typeof savedEvent1>[0] = {}) {
  vi.mocked(bikriApi.bikriGet).mockResolvedValue({
    ...savedEvent1(overrides),
    id,
  });
}

function present<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) throw new Error('not on screen');
  return value;
}

describe.each(LANGUAGES)('Edit and Cancel on the rows of پچھلی بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  /** The day of each row of the window: the rows are under the line of their day (plan 75 7.7). */
  const DAY_OF: Record<number, string> = { 61: '2026-10-09', 62: '2026-10-08', 63: '2026-10-07' };
  const dayBox = (rowId: number) =>
    present(document.querySelector<HTMLElement>(`[data-day="${present(DAY_OF[rowId])}"]`));
  const windowGrid = (rowId: number) =>
    within(dayBox(rowId)).getByRole('group', { name: named(m('bikri-past-label')) });
  const todayGrid = () => gridOf(language, 'bikri-list-label');
  const open = async (path = '/bikri/today') => {
    const view = await renderApp(path);
    await screen.findByRole('group', { name: named(m('bikri-past-nav-label')) });
    return { ...view, user: userEvent.setup() };
  };
  /** The bar above a part: 0 is today's, 1 is the window's. */
  const bars = () => Array.from(document.querySelectorAll<HTMLElement>('[data-past-row-actions]'));
  const bar = (index: number) => present(bars()[index]);
  const buttonIn = (index: number, words: string) =>
    within(bar(index)).getByRole('button', {
      name: (name) => visible(name).startsWith(words),
    });
  const cell = (grid: HTMLElement, rowId: number, field: string) =>
    present(
      grid.querySelector<HTMLElement>(`.ag-row[row-id="${String(rowId)}"] [col-id="${field}"]`),
    );
  // The day of the row is closed when the list opens (with the tick "show every بکری" it is open already), so
  // the Owner opens it with its arrow first.
  const windowShown = async (rowId: number) => {
    await waitFor(() => {
      expect(dayBox(rowId).querySelector('[data-day-line]')).not.toBeNull();
    });
    const line = present(dayBox(rowId).querySelector<HTMLElement>('[data-day-line]'));
    if (line.getAttribute('aria-expanded') === 'false') await userEvent.click(line);
    await waitFor(() => {
      expect(windowGrid(rowId).querySelector(`.ag-row[row-id="${String(rowId)}"]`)).not.toBeNull();
    });
  };

  it('each part has the two buttons, off with a hint while no row has focus', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    await open();
    await windowShown(61);
    expect(bars()).toHaveLength(2);
    for (const index of [0, 1]) {
      expect(visible(bar(index).textContent)).toContain(m('bikri-past-row-none'));
      expect(buttonIn(index, m('edit-action-edit'))).toBeDisabled();
      expect(buttonIn(index, m('edit-action-cancel'))).toBeDisabled();
    }
  });

  it('a click on a row of the window says which بکری the buttons work on, and turns them on', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    const { user } = await open();
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    await waitFor(() => {
      expect(visible(bar(1).textContent)).toContain(m('bikri-past-row-for', { number: 2 }));
    });
    expect(buttonIn(1, m('edit-action-edit'))).toBeEnabled();
    expect(buttonIn(1, m('edit-action-cancel'))).toBeEnabled();
    // Today's part is still waiting for its own row.
    expect(visible(bar(0).textContent)).toContain(m('bikri-past-row-none'));
  });

  it('Edit opens the بکری straight in edit mode, with the first editable posting and the way back kept', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    fakePlan16();
    fakeDetail(61);
    const { router, user } = await open();
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    await user.click(buttonIn(1, m('edit-action-edit')));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 61, edit: 7, past: '2026-10-09' });
    expect(router.state.location.search).not.toHaveProperty('pastAll');
  });

  it('with the switch on, the way back says so', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    fakePlan16();
    fakeDetail(61);
    const { router, user } = await open('/bikri/today?all=true');
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    await user.click(buttonIn(1, m('edit-action-edit')));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({
      id: 61,
      edit: 7,
      past: '2026-10-09',
      pastAll: true,
    });
  });

  it('a row of today works the same: its own buttons, its own way back', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    fakePlan16();
    fakeDetail(41);
    const { router, user } = await open();
    await waitFor(() => {
      expect(todayGrid().querySelector('.ag-row[row-id="41"]')).not.toBeNull();
    });
    await user.click(cell(todayGrid(), 41, 'number'));
    await user.click(buttonIn(0, m('edit-action-edit')));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: '2026-10-09' });
  });

  it('keyboard only: Shift+Tab from the row reaches Edit, Enter opens the edit', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    fakePlan16();
    fakeDetail(61);
    const { router, user } = await open();
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    await waitFor(() => {
      expect(buttonIn(1, m('edit-action-edit'))).toBeEnabled();
    });
    const edit = buttonIn(1, m('edit-action-edit'));
    for (let tabs = 0; tabs < 40 && document.activeElement !== edit; tabs += 1) {
      await user.tab({ shift: true });
    }
    expect(edit).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 61, edit: 7 });
  });

  it('Enter on a row still opens the whole بکری to look at, and the buttons do not take that key', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    const { router, user } = await open();
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 61, past: '2026-10-09' });
    expect(router.state.location.search).not.toHaveProperty('edit');
  });

  it('a row of a closed day reads "reopen the day and fix", asks why, reopens the days, then opens the edit (FR-21.4)', async () => {
    fakeRows({ window: [CLOSED_ROW] });
    fakePlan16();
    fakeDetail(62, { needs_reopen: true, entry_date: '2026-10-08' });
    vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
    const { router, user } = await open();
    await windowShown(62);
    await user.click(cell(windowGrid(62), 62, 'number'));
    const fix = buttonIn(1, m('edit-action-fix-day'));
    expect(fix).toBeEnabled();
    await user.click(fix);
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(m('edit-fix-label')) }),
      'wrong rate{Enter}',
    );
    await user.keyboard('adnan{Enter}');
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledTimes(1);
    });
    // The day of the posting (2026-10-10 in the backend's saved بکری) goes as typed, DD-MM-YYYY (Umar, 13-11-2026).
    expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', 'adnan');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 62, edit: 7, past: '2026-10-09' });
  });

  it('a row that cannot be changed says why in words, and both buttons stay off', async () => {
    fakeRows({ window: [LOCKED_ROW] });
    const { user } = await open();
    await windowShown(63);
    await user.click(cell(windowGrid(63), 63, 'number'));
    await waitFor(() => {
      expect(visible(bar(1).textContent)).toContain(m('edit-locked-goods-moved'));
    });
    expect(buttonIn(1, m('edit-action-edit'))).toBeDisabled();
    expect(buttonIn(1, m('edit-action-cancel'))).toBeDisabled();
  });

  it('Cancel asks the reason in the shared box, cancels, and the list stays and reads the backend again', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    const plan = fakePlan16();
    const { router, user } = await open();
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    const before = vi.mocked(bikriApi.bikriList).mock.calls.length;
    await user.click(buttonIn(1, m('edit-action-cancel')));
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(m('bikri-cancel-reason')) }),
      'entered twice{Enter}',
    );
    await waitFor(() => {
      expect(plan.cancels).toHaveLength(1);
    });
    expect(plan.cancels[0]).toMatchObject({ id: 61, reason: 'entered twice', confirm: false });
    // The Owner has not left the list, and it asks the backend again (the figures may have moved).
    expect(router.state.location.pathname).toBe('/bikri/today');
    await waitFor(() => {
      expect(vi.mocked(bikriApi.bikriList).mock.calls.length).toBeGreaterThan(before);
    });
  });

  it('after a cancel the row shows cancelled with the switch on, and the day figures the backend sends are drawn', async () => {
    const cancelled = listRow({
      ...WINDOW_ROW,
      status: 'cancelled',
      can_edit: false,
      can_cancel: false,
    });
    fakeRows({
      window: [cancelled],
      gross: { '2026-10-09': 0 },
      windowTotals: { gross: paisaForTest(0) },
    });
    const { user } = await open('/bikri/today?all=true');
    await windowShown(61);
    expect(visible(cell(windowGrid(61), 61, 'status').textContent)).toBe(
      m('bikri-status-cancelled'),
    );
    await user.click(cell(windowGrid(61), 61, 'number'));
    // A cancelled بکری has no Edit or Cancel left to press.
    expect(buttonIn(1, m('edit-action-edit'))).toBeDisabled();
    expect(buttonIn(1, m('edit-action-cancel'))).toBeDisabled();
  });

  it('a refusal while the edit is being opened is told in words, and the Owner stays on the list', async () => {
    fakeRows({ window: [WINDOW_ROW] });
    vi.mocked(bikriApi.bikriGet).mockRejectedValue({
      ok: false,
      errors: [{ code: 'ERR_DB_BUSY', field: null, values: {} }],
      ref: 'E-1',
    });
    const { router, user } = await open();
    await windowShown(61);
    await user.click(cell(windowGrid(61), 61, 'number'));
    await user.click(buttonIn(1, m('edit-action-edit')));
    await waitFor(() => {
      expect(within(bar(1)).queryByRole('alert')).not.toBeNull();
    });
    expect(router.state.location.pathname).toBe('/bikri/today');
  });
});
