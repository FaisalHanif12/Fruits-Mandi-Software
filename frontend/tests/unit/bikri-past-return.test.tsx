// The way between پچھلی بکری and the sale book, with the keyboard only (plan 72 P10, PRD FR-12.46, FR-7.70,
// FR-21.4, FR-7.9), in the Urdu and the English app, against a fake backend. A row opens the whole
// بکری with the window it came from in the address (`past`, `pastAll`); Edit keeps it; a save, on an
// open day or on a closed one (reopen and fix), goes back to the same window and row, the list asks
// the backend again, and the row has focus. Back without a save and a cancel keep the way back too.
// The screens are built by `frontend-past`; the click tests of the same path are in past-sales.test.tsx.
// The second part of the file is the two buttons of a ROW of the list (تبدیل کریں and منسوخ کریں above each
// part of پچھلی بکری): the hint that names the row, the buttons off while no row is picked or the row is
// locked (with the reason in words), Edit by keyboard to the sale book and back to the same row, the
// closed day that is reopened first, Cancel by keyboard without leaving the list, a refusal told in
// words, and one run with the keyboard only: open, edit, save, cancel, back. Today in these tests is 10-10-2026.

import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import * as bikriApi from '../../src/features/bikri/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { FALLBACK_ERROR_MESSAGE_ID } from '../../src/shared/api';
import * as editApi from '../../src/shared/edit/api';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { listRow, savedEvent1 } from '../helpers/bikri-saved';
import { cellOf, focusedIn, lotsGrid, named, salesGrid } from '../helpers/bikri-screen';
import { CLOSED_DAY_ENTRY, lockedEntry } from '../helpers/entry-flags';
import { DAY_FIX_CLOSE_E2E_45, DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { paisaForTest } from '../helpers/paisa';
import { EMPTY_PAIRS } from '../helpers/paged';
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

/** The بکری 41 of Event 1, entered on 09-10-2026, so it stands in the window that ends that day. */
const WINDOW_ROWS = [listRow({ id: 41, number: 1, entry_date: '2026-10-09', truck_no: null })];
const WINDOW_GROSS = { '2026-10-09': 29_400 };
const WINDOW_END = '2026-10-09';

/** The double answers both sections of the list from `fake`. */
function fakePast(fake: PastFake = {}) {
  vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
    Promise.resolve(pastAnswer(input, fake)),
  );
}

/** The calls of the window (they have no `date`). */
const windowCalls = () =>
  vi
    .mocked(bikriApi.bikriList)
    .mock.calls.map(([input]) => input)
    .filter((input) => input.date === null);

/**
 * Presses Enter through the eight stops of the lot until the first line's buyer has the focus (plan 71 H1). A key
 * that arrives while a cell is still opening is lost, so the focus is looked at after each press and the key is
 * pressed again if it did not move on; the full run is slow and the keys came too fast.
 */
const enterThroughLot = async (user: ReturnType<typeof userEvent.setup>, language: Language) => {
  const inBuyer = () => {
    try {
      return focusedIn(salesGrid(language)) === '0:buyer';
    } catch {
      return false;
    }
  };
  for (let presses = 0; presses < 20 && !inBuyer(); presses += 1) {
    await user.keyboard('{Enter}');
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
};

/** The day of بکری 41 in the window, whose line opens to show its row (plan 75 7.7). */
const DAY_SELECTOR = `[data-day="${WINDOW_END}"]`;
const dayLineOf = () => document.querySelector<HTMLElement>(`${DAY_SELECTOR} [data-day-line]`);
const dayGridOf = (language: Language): HTMLElement => {
  const day = document.querySelector<HTMLElement>(DAY_SELECTOR);
  if (day === null) throw new Error('the day is not on screen');
  return within(day).getByRole('group', { name: named(message(language, 'bikri-past-label')) });
};

/**
 * The saved بکری 41 as the backend reads it: entered on the day of its row in the window, so the list knows
 * which day to open on the way back from an edit (plan 75 7.7).
 */
const keepDayOf41 = () => {
  vi.mocked(bikriApi.bikriGet).mockResolvedValue({ ...savedEvent1(), entry_date: WINDOW_END });
};

describe.each(LANGUAGES)('the way between پچھلی بکری and the sale book (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    keepDayOf41();
    // Today's section is empty: the row of بکری 1 is in the window, once.
    fakePast({ today: [], window: WINDOW_ROWS, gross: WINDOW_GROSS });
  });

  const windowGrid = () => dayGridOf(language);
  // The way back from an edit opens the day of the بکری by itself; the day is not opened by hand here.
  const listShown = async () => {
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-today')),
    });
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="41"]')).not.toBeNull();
    });
  };
  const savedShown = async () => {
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
  };
  const editShown = async () => {
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-edit-title', { number: 1 })),
    });
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'rate')).toBeInTheDocument();
    });
  };

  /** Presses Tab until the focused element is the one `found` says, so no mouse is used. */
  const tabTo = async (
    user: ReturnType<typeof userEvent.setup>,
    found: (element: Element) => boolean,
  ) => {
    for (let tabs = 0; tabs < 80; tabs += 1) {
      const active = document.activeElement;
      // The side menu has an item with the name of the screen too: only the page's own buttons count.
      if (active !== null && active.closest('nav') === null && found(active)) return;
      await user.tab();
    }
    throw new Error('the keyboard never reached it');
  };

  /** The keyboard walks to the first cell of the first line's rate and types a new rate. */
  const typeRateWithKeys = async (user: ReturnType<typeof userEvent.setup>, rate: string) => {
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('0:seller');
    });
    // The eight stops of the lot, then the first line's buyer, quantity and rate (plan 71 H1).
    await enterThroughLot(user, language);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    await user.keyboard('{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:rate');
    });
    await user.keyboard(`${rate}{Enter}`);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
    });
  };

  it('keyboard only: Enter on a row opens the بکری, Edit, a new rate, F10, and the row has focus again', async () => {
    const plan = fakePlan16();
    // This time the بکری is in today's section (a window with nothing in it).
    fakePast({ today: [listRow({ id: 41, number: 1 })] });
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'layout-module-today')),
    });
    const todayGrid = () => {
      const grid = document.querySelector<HTMLElement>('.ag-root-wrapper');
      if (grid === null) throw new Error('no grid');
      return grid;
    };
    await waitFor(() => {
      expect(todayGrid().querySelector('.ag-row[row-id="41"]')).not.toBeNull();
    });
    // Tab into today's grid, down to the row of بکری 1.
    await tabTo(user, (element) => todayGrid().contains(element));
    for (let steps = 0; steps < 6; steps += 1) {
      if (document.activeElement?.closest('.ag-row')?.getAttribute('row-id') === '41') break;
      await user.keyboard('{ArrowDown}');
    }
    expect(document.activeElement?.closest('.ag-row')?.getAttribute('row-id')).toBe('41');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41, past: WINDOW_END });
    await savedShown();
    // Edit by its key: Tab to the button and Enter.
    await tabTo(
      user,
      (element) =>
        element.tagName === 'BUTTON' &&
        visible(element.textContent) === message(language, 'edit-action-edit'),
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: WINDOW_END });
    });
    await editShown();
    await typeRateWithKeys(user, '610');
    // The list was read before the edit; after the save it is read again (nothing is kept on the screen).
    const listCallsBeforeSave = vi.mocked(bikriApi.bikriList).mock.calls.length;
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    // The list reads the backend again, and the row has focus.
    await waitFor(() => {
      expect(todayGrid().querySelector('.ag-row[row-id="41"]')).not.toBeNull();
    });
    await waitFor(() => {
      expect(vi.mocked(bikriApi.bikriList).mock.calls.length).toBeGreaterThan(listCallsBeforeSave);
    });
    await waitFor(() => {
      expect(todayGrid().querySelector('.ag-row[row-id="41"] .ag-cell-focus')).not.toBeNull();
    });
  });

  it('the tick "show every بکری" comes along both ways: it is in the address of the book and of the list', async () => {
    const plan = fakePlan16();
    const { router } = await renderApp(`/bikri/book?id=41&edit=7&past=${WINDOW_END}&pastAll=true`);
    const user = userEvent.setup();
    await editShown();
    await typeRateWithKeys(user, '610');
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41, all: true });
    await listShown();
    await waitFor(() => {
      expect(windowCalls().at(-1)).toMatchObject({ all: true, to: '09-10-2026' });
    });
  });

  it('Esc in the edit without a save goes back to the list, at the same window and on the same row', async () => {
    fakePlan16();
    const { router } = await renderApp(`/bikri/book?id=41&edit=7&past=${WINDOW_END}`);
    const user = userEvent.setup();
    await editShown();
    // Back is on the keyboard too: Esc (the edit has no change, so nothing is asked). The Owner came
    // from the list, so the list is where he lands, not the saved بکری (plan 72 P10).
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    await listShown();
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="41"] .ag-cell-focus')).not.toBeNull();
    });
  });

  it('the saved بکری has a button back to the list that lands on the same row', async () => {
    fakePlan16();
    const { router } = await renderApp(`/bikri/book?id=41&past=${WINDOW_END}`);
    const user = userEvent.setup();
    await savedShown();
    await tabTo(
      user,
      (element) =>
        element.tagName === 'BUTTON' &&
        visible(element.textContent) === message(language, 'bikri-past-back-to-list'),
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    await listShown();
  });

  it('a closed day: "reopen the day and fix" asks why, keeps the way back, and the save returns to the list (PRD FR-21.4)', async () => {
    const plan = fakePlan16();
    vi.mocked(bikriApi.bikriGet).mockResolvedValue({
      ...savedEvent1(),
      needs_reopen: true,
      postings: [
        {
          entry_id: 7,
          posting_type: 'bikri_first',
          entry_date: '2026-10-10',
          total: paisaForTest(3_656_400),
          reversed: false,
          editable: false,
        },
      ],
    });
    vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
    vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_E2E_45);
    const { router } = await renderApp(`/bikri/book?id=41&past=${WINDOW_END}`);
    const user = userEvent.setup();
    await savedShown();
    // The toolbar's button is the first of the two that say it.
    await tabTo(
      user,
      (element) =>
        element.tagName === 'BUTTON' &&
        visible(element.textContent) === message(language, 'edit-action-fix-day'),
    );
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
      'wrong rate{Enter}',
    );
    await user.keyboard('adnan{Enter}');
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', 'adnan');
    });
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: WINDOW_END });
    });
    await editShown();
    await typeRateWithKeys(user, '610');
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    // Leaving the sale book closes the days again, once, with the id of the fix.
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
  });

  it('a cancel keeps the way back: the cancelled بکری has the button back, and the list asks again', async () => {
    const plan = fakePlan16();
    const { router } = await renderApp(`/bikri/book?id=41&past=${WINDOW_END}`);
    const user = userEvent.setup();
    await savedShown();
    await tabTo(
      user,
      (element) =>
        element.tagName === 'BUTTON' &&
        visible(element.textContent).startsWith(message(language, 'edit-action-cancel')),
    );
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('alertdialog');
    await waitFor(() => {
      expect(
        within(dialog).getByRole('combobox', {
          name: named(message(language, 'bikri-cancel-reason')),
        }),
      ).toHaveFocus();
    });
    await user.keyboard('entered twice{Enter}');
    await waitFor(() => {
      expect(plan.cancels).toHaveLength(1);
    });
    // The cancelled بکری is still on screen with its address, and the button back lands on its row.
    expect(router.state.location.search).toMatchObject({ id: 41, past: WINDOW_END });
    const calls = windowCalls().length;
    await tabTo(
      user,
      (element) =>
        element.tagName === 'BUTTON' &&
        visible(element.textContent) === message(language, 'bikri-past-back-to-list'),
    );
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    await listShown();
    await waitFor(() => {
      expect(windowCalls().length).toBeGreaterThan(calls);
    });
  });

  it('a language switch in the middle of the edit loses nothing and the save still goes back to the same row', async () => {
    const plan = fakePlan16();
    const other: Language = language === 'ur' ? 'en' : 'ur';
    const { router } = await renderApp(`/bikri/book?id=41&edit=7&past=${WINDOW_END}`);
    const user = userEvent.setup();
    await editShown();
    await typeRateWithKeys(user, '610');
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    await waitFor(() => {
      expect(cellOf(salesGrid(other), 0, 'rate')).toHaveClass('cell-changed');
    });
    expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: WINDOW_END });
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    expect(plan.editSaves[0]?.draft.lots[0]?.sales[0]).toMatchObject({ seller_rate: '610' });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
  });
});

/** A refusal with nothing in it: the backend's slip, told in the plain words of the app. */
const SLIP: UiError = { ok: false, ref: 'R-77', errors: [] };

/** The window part's two buttons live in the second block of actions (today's part has the first). */
const WINDOW_ACTIONS = 1;

describe.each(LANGUAGES)('the Edit and Cancel buttons of a row of پچھلی بکری (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  /** The row of بکری 1 as the window lists it on 09-10-2026, with the flags a test is about. */
  const windowRow = (overrides: Partial<Parameters<typeof listRow>[0]> = {}) =>
    listRow({ id: 41, number: 1, entry_date: WINDOW_END, truck_no: null, ...overrides });

  const fakeRows = (rows: () => ReturnType<typeof windowRow>[]) => {
    vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
      Promise.resolve(pastAnswer(input, { today: [], window: rows(), gross: WINDOW_GROSS })),
    );
  };

  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    keepDayOf41();
  });

  const windowGrid = () => dayGridOf(language);
  const actionsOf = (part: number): HTMLElement => {
    const found = document.querySelectorAll<HTMLElement>('[data-past-row-actions]')[part];
    if (found === undefined) throw new Error(`no actions block ${String(part)}`);
    return found;
  };
  const windowActions = () => actionsOf(WINDOW_ACTIONS);
  const buttonIn = (block: HTMLElement, label: string) =>
    within(block).getByRole('button', {
      name: (name) => visible(name).startsWith(label),
    });
  const editButton = () => buttonIn(windowActions(), m('edit-action-edit'));
  const cancelButton = () => buttonIn(windowActions(), m('edit-action-cancel'));
  const blockText = (block: HTMLElement) => visible(block.textContent);

  // The list is opened from the menu here, so the day of the row is closed until it is opened by hand (plan 75
  // 7.7); with the tick "show every بکری" it opens by itself. After a way back from an edit it opens by itself.
  const listShown = async (byHand = false) => {
    await screen.findByRole('heading', { level: 1, name: named(m('layout-module-today')) });
    if (byHand) {
      await waitFor(() => {
        expect(dayLineOf()).not.toBeNull();
      });
      const line = dayLineOf();
      if (line?.getAttribute('aria-expanded') === 'false') await userEvent.click(line);
    }
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="41"]')).not.toBeNull();
    });
  };
  const editShown = async () => {
    await screen.findByRole('heading', {
      level: 1,
      name: named(m('bikri-edit-title', { number: 1 })),
    });
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'rate')).toBeInTheDocument();
    });
  };
  const onRow41 = () => document.activeElement?.closest('.ag-row')?.getAttribute('row-id') === '41';

  /** Tab (or Shift+Tab) until the focused element is the one `found` says, so no mouse is used. */
  const tabTo = async (
    user: ReturnType<typeof userEvent.setup>,
    found: (element: Element) => boolean,
    backwards = false,
  ) => {
    for (let tabs = 0; tabs < 80; tabs += 1) {
      const active = document.activeElement;
      if (active !== null && active.closest('nav') === null && found(active)) return;
      await (backwards ? user.tab({ shift: true }) : user.tab());
    }
    throw new Error('the keyboard never reached it');
  };
  const toButton = (user: ReturnType<typeof userEvent.setup>, label: string) =>
    tabTo(
      user,
      (element) =>
        element.tagName === 'BUTTON' &&
        windowActions().contains(element) &&
        visible(element.textContent).startsWith(label),
      true,
    );

  /** Tab into the window's grid and arrow down to the row of بکری 1: the row has focus, no mouse used. */
  const pickRow = async (user: ReturnType<typeof userEvent.setup>) => {
    await listShown(true);
    await tabTo(user, (element) => windowGrid().contains(element));
    for (let steps = 0; steps < 20 && !onRow41(); steps += 1) await user.keyboard('{ArrowDown}');
    expect(onRow41()).toBe(true);
  };

  const typeRate = async (user: ReturnType<typeof userEvent.setup>, rate: string) => {
    await waitFor(() => {
      expect(focusedIn(lotsGrid(language))).toBe('0:seller');
    });
    await enterThroughLot(user, language);
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:buyer');
    });
    await user.keyboard('{Enter}{Enter}');
    await waitFor(() => {
      expect(focusedIn(salesGrid(language))).toBe('0:rate');
    });
    await user.keyboard(`${rate}{Enter}`);
    await waitFor(() => {
      expect(cellOf(salesGrid(language), 0, 'rate')).toHaveClass('cell-changed');
    });
  };

  const cancelWithReason = async (user: ReturnType<typeof userEvent.setup>, reason: string) => {
    const dialog = await screen.findByRole('alertdialog');
    await waitFor(() => {
      expect(
        within(dialog).getByRole('combobox', { name: named(m('bikri-cancel-reason')) }),
      ).toHaveFocus();
    });
    await user.keyboard(`${reason}{Enter}`);
  };

  it('with no row picked the hint says to pick one and both buttons are off', async () => {
    fakePlan16();
    fakeRows(() => [windowRow()]);
    await renderApp('/bikri/today');
    await listShown(true);
    for (const part of [0, WINDOW_ACTIONS]) {
      expect(blockText(actionsOf(part))).toContain(m('bikri-past-row-none'));
    }
    expect(editButton()).toBeDisabled();
    expect(cancelButton()).toBeDisabled();
    expect(vi.mocked(bikriApi.bikriGet)).not.toHaveBeenCalled();
  });

  it('the keyboard picks a row: the hint names it and its buttons come on, the other part stays off', async () => {
    fakePlan16();
    fakeRows(() => [windowRow()]);
    await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await waitFor(() => {
      expect(blockText(windowActions())).toContain(m('bikri-past-row-for', { number: '1' }));
    });
    expect(editButton()).toBeEnabled();
    expect(cancelButton()).toBeEnabled();
    // Today's part has no row picked, so its hint is the same and its buttons are off.
    expect(blockText(actionsOf(0))).toContain(m('bikri-past-row-none'));
    expect(buttonIn(actionsOf(0), m('edit-action-edit'))).toBeDisabled();
  });

  it('Edit by keyboard opens the sale book in edit mode with the way back, and the save lands on the same row', async () => {
    const plan = fakePlan16();
    fakeRows(() => [windowRow()]);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await toButton(user, m('edit-action-edit'));
    await user.keyboard('{Enter}');
    // The entry is read when Edit is pressed, so it opens what the backend says now.
    await waitFor(() => {
      expect(bikriApi.bikriGet).toHaveBeenCalledWith(41);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: WINDOW_END });
    await editShown();
    await typeRate(user, '610');
    const reads = vi.mocked(bikriApi.bikriList).mock.calls.length;
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="41"] .ag-cell-focus')).not.toBeNull();
    });
    expect(vi.mocked(bikriApi.bikriList).mock.calls.length).toBeGreaterThan(reads);
    // The row has focus, so its hint and its buttons are there again.
    expect(blockText(windowActions())).toContain(m('bikri-past-row-for', { number: '1' }));
    expect(editButton()).toBeEnabled();
  });

  it('the tick "show every بکری" goes along with Edit from a row', async () => {
    const plan = fakePlan16();
    fakeRows(() => [windowRow()]);
    const { router } = await renderApp('/bikri/today?all=true');
    const user = userEvent.setup();
    await pickRow(user);
    await toButton(user, m('edit-action-edit'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({
      id: 41,
      edit: 7,
      past: WINDOW_END,
      pastAll: true,
    });
    await editShown();
    await typeRate(user, '610');
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41, all: true });
  });

  it('a row of a closed day reads "reopen the day and fix": it asks why, then opens the edit, and the save closes the days again', async () => {
    const plan = fakePlan16();
    fakeRows(() => [windowRow({ ...CLOSED_DAY_ENTRY })]);
    vi.mocked(bikriApi.bikriGet).mockResolvedValue({
      ...savedEvent1(),
      needs_reopen: true,
      postings: [
        {
          entry_id: 7,
          posting_type: 'bikri_first',
          entry_date: '2026-10-10',
          total: paisaForTest(3_656_400),
          reversed: false,
          editable: false,
        },
      ],
    });
    vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
    vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_E2E_45);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    expect(
      within(windowActions()).queryByRole('button', {
        name: (name) => visible(name) === m('edit-action-edit'),
      }),
    ).toBeNull();
    await toButton(user, m('edit-action-fix-day'));
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(m('edit-fix-label')) }),
      'wrong rate{Enter}',
    );
    await user.keyboard('adnan{Enter}');
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', 'adnan');
    });
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: WINDOW_END });
    });
    await editShown();
    await typeRate(user, '610');
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    expect(router.state.location.search).toMatchObject({ end: WINDOW_END, row: 41 });
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
  });

  it('Cancel by keyboard asks the reason, cancels without leaving the list, and the row comes back cancelled and locked', async () => {
    const plan = fakePlan16();
    fakeRows(() => [
      plan.cancels.length === 0
        ? windowRow()
        : windowRow({ status: 'cancelled', ...lockedEntry('cancelled') }),
    ]);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await toButton(user, m('edit-action-cancel'));
    await user.keyboard('{Enter}');
    const reads = windowCalls().length;
    await cancelWithReason(user, 'entered twice');
    await waitFor(() => {
      expect(plan.cancels).toHaveLength(1);
    });
    expect(plan.cancels[0]).toMatchObject({ id: 41, reason: 'entered twice' });
    // The Owner never left the list, and the box is closed.
    expect(router.state.location.pathname).toBe('/bikri/today');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    // The list was read again; the row says why it is locked, and both buttons are off.
    await waitFor(() => {
      expect(windowCalls().length).toBeGreaterThan(reads);
    });
    await waitFor(() => {
      expect(blockText(windowActions())).toContain(m('edit-locked-cancelled'));
    });
    expect(editButton()).toBeDisabled();
    expect(cancelButton()).toBeDisabled();
  });

  it('Esc in the reason box closes it and nothing is cancelled', async () => {
    const plan = fakePlan16();
    fakeRows(() => [windowRow()]);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await toButton(user, m('edit-action-cancel'));
    await user.keyboard('{Enter}');
    await screen.findByRole('alertdialog');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(plan.cancels).toHaveLength(0);
    expect(router.state.location.pathname).toBe('/bikri/today');
    expect(cancelButton()).toBeEnabled();
  });

  it('Enter pressed twice at once in the reason box cancels the بکری exactly once', async () => {
    const plan = fakePlan16();
    fakeRows(() => [windowRow()]);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await toButton(user, m('edit-action-cancel'));
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('alertdialog');
    const reason = within(dialog).getByRole('combobox', { name: named(m('bikri-cancel-reason')) });
    await waitFor(() => {
      expect(reason).toHaveFocus();
    });
    await user.type(reason, 'entered twice');
    // Both keys arrive before the box has drawn "sending", as a double press does.
    act(() => {
      fireEvent.keyDown(reason, { key: 'Enter', code: 'Enter' });
      fireEvent.keyDown(reason, { key: 'Enter', code: 'Enter' });
    });
    await waitFor(() => {
      expect(plan.cancels.length).toBeGreaterThanOrEqual(1);
    });
    await new Promise((resolve) => setTimeout(resolve, 300));
    expect(plan.cancels).toHaveLength(1);
    expect(router.state.location.pathname).toBe('/bikri/today');
  });

  it('a row that cannot be changed says why in words, and its buttons do nothing', async () => {
    fakePlan16();
    fakeRows(() => [windowRow({ ...lockedEntry('goods_moved') })]);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await waitFor(() => {
      expect(blockText(windowActions())).toContain(m('edit-locked-goods-moved'));
    });
    expect(editButton()).toBeDisabled();
    expect(cancelButton()).toBeDisabled();
    await user.click(editButton());
    await user.click(cancelButton());
    expect(bikriApi.bikriGet).not.toHaveBeenCalled();
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(router.state.location.pathname).toBe('/bikri/today');
  });

  it('when the entry cannot be read the refusal is told in words under the buttons and the list stays', async () => {
    fakePlan16();
    fakeRows(() => [windowRow()]);
    vi.mocked(bikriApi.bikriGet).mockRejectedValue(SLIP);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await pickRow(user);
    await toButton(user, m('edit-action-edit'));
    await user.keyboard('{Enter}');
    const alert = await within(windowActions()).findByRole('alert');
    expect(visible(alert.textContent)).toContain(m(FALLBACK_ERROR_MESSAGE_ID));
    expect(visible(alert.textContent)).toContain('R-77');
    expect(router.state.location.pathname).toBe('/bikri/today');
    // The buttons are free again: nothing is left working.
    await waitFor(() => {
      expect(editButton()).toBeEnabled();
    });
  });

  it('keyboard only, from the list: open, edit, save, cancel and back', async () => {
    const plan = fakePlan16();
    fakeRows(() => [
      plan.cancels.length === 0
        ? windowRow()
        : windowRow({ status: 'cancelled', ...lockedEntry('cancelled') }),
    ]);
    const { router } = await renderApp('/bikri/today');
    const user = userEvent.setup();

    // 1. Open: Enter on the row opens the whole بکری, with the way back in the address.
    await pickRow(user);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41, past: WINDOW_END });
    await screen.findByRole('heading', {
      level: 1,
      name: named(m('bikri-saved-title', { number: 1 })),
    });

    // 2. Back: Esc on the saved بکری returns to the same row of the list (as its button does); the row has focus.
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="41"] .ag-cell-focus')).not.toBeNull();
    });

    // 3. Edit from the row, a new rate, F10: back on the list with the row focused.
    await toButton(user, m('edit-action-edit'));
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/book');
    });
    expect(router.state.location.search).toMatchObject({ id: 41, edit: 7, past: WINDOW_END });
    await editShown();
    await typeRate(user, '610');
    press('F10');
    await waitFor(() => {
      expect(plan.editSaves).toHaveLength(1);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/today');
    });
    await waitFor(() => {
      expect(windowGrid().querySelector('.ag-row[row-id="41"] .ag-cell-focus')).not.toBeNull();
    });

    // 4. Cancel from the row with a reason; the Owner stays on the list and the row is locked.
    await toButton(user, m('edit-action-cancel'));
    await user.keyboard('{Enter}');
    await cancelWithReason(user, 'entered twice');
    await waitFor(() => {
      expect(plan.cancels).toHaveLength(1);
    });
    expect(router.state.location.pathname).toBe('/bikri/today');
    await waitFor(() => {
      expect(blockText(windowActions())).toContain(m('edit-locked-cancelled'));
    });

    // 5. Back: Esc on the list leaves the screen, as on every other list.
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });
});
