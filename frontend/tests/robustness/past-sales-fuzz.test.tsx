// Plan 72 end gate, attack on the keyboard and the input of پچھلی بکری (PRD 12.7, FR-12.46, FR-7.19, FR-7.70,
// FR-7.91, FR-21.4; NFR-USE-7). Owner: `robustness`.
//
// A seeded fuzz on the real screen, the real grids and the real router, against a stateful fake backend
// (`past-world.ts`) whose rules are written again with plain calendar maths. Each trial opens the screen at
// a start address (some of them hostile), then does 4 to 8 random things: the two buttons, Page Up and
// Page Down in the window, the arrow keys on the bar, the button back to the latest week, a day typed in
// "تاریخ پر جائیں" (real, future, impossible and odd texts), the switch "show every بکری", a typed search,
// Backspace and Delete on a row, Page keys in today's part, opening a row, Edit, Cancel, and a language
// switch. After every step the screen must agree with the model, which is kept apart from the screen:
//
//   1. Every list call is well formed: a window is exactly 7 days, ends no later than yesterday, and asks
//      for its days; today's call is for today. No call has no day. No day is after today.
//   2. The window the screen asks for is the window the model says; the address says the same window and
//      the same switch (an end of yesterday may be left out).
//   3. The grid shows the day headings of the window, newest first, each with the backend's count and
//      خام بکری, and under each the rows the backend sent, in its order. An empty window says so in words.
//   4. A day that is in the future or not real is refused in words and the window does not move.
//   5. The leave box never opens, the screen is still the list, and Backspace, Delete and the Page keys
//      never leave it (FR-7.91).
//   6. Open, Edit (with and without a save) and Cancel come back to the same window, the same switch and
//      the same row, and the list asks the backend again.
//
// It does not click the mouse for the keys it is about. The real timing on WebView2 is on the Windows list.

import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriListInput } from '../../src/bindings/BikriListInput';
import * as bikriApi from '../../src/features/bikri/api';
import * as editApi from '../../src/shared/edit/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import { listRow, savedEvent1 } from '../helpers/bikri-saved';
import { cellOf, named, salesGrid } from '../helpers/bikri-screen';
import { money } from '../helpers/accounts';
import { lockedEntry } from '../helpers/entry-flags';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { PAGE_ROWS } from '../helpers/pager';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import {
  TODAY,
  YESTERDAY,
  answerList,
  buildWorld,
  dayAfter,
  daysOfWindow,
  isoOf,
  rng,
  rowsFor,
  rupeesText,
  windowFrom,
  type Random,
  type World,
} from './past-world';

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

const SLOW = { timeout: 10_000 };
const TRIALS = 14;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  leaveAppWindow();
});

// ---- the model -----------------------------------------------------------------------------------

interface State {
  /** The last day of the window the screen must show. */
  end: string;
  /** "Show every بکری" is on. */
  all: boolean;
  /** The text in the search box. */
  search: string;
  lang: Language;
}

/** A start address, and the window it must give. The addresses are what a way back or a reload can hold. */
const STARTS: { search: string; end: string; all: boolean }[] = [
  { search: '', end: YESTERDAY, all: false },
  { search: '?end=2026-10-02', end: '2026-10-02', all: false },
  { search: '?end=2026-10-10', end: YESTERDAY, all: false },
  { search: '?end=2030-01-01', end: YESTERDAY, all: false },
  { search: '?end=2026-02-30', end: YESTERDAY, all: false },
  { search: '?end=garbage', end: YESTERDAY, all: false },
  { search: '?all=true', end: YESTERDAY, all: true },
  { search: '?end=2026-09-25&all=true', end: '2026-09-25', all: true },
  { search: '?row=999999', end: YESTERDAY, all: false },
  { search: '?row=-5&end=2026-10-03', end: '2026-10-03', all: false },
  { search: '?end=2026-10-09&row=41', end: YESTERDAY, all: false },
];

/** What a typed day must do. Written by hand from PRD FR-7.19 and today = 10-10-2026. */
const GOTO: { text: string; go?: string; refusal?: 'invalid' | 'future' }[] = [
  { text: '05-10-2026', go: '2026-10-05' },
  { text: '5', go: '2026-10-05' },
  { text: '5-9', go: '2026-09-05' },
  { text: '15-8-2026', go: '2026-08-15' },
  { text: '09-10-2026', go: YESTERDAY },
  { text: '10-10-2026', go: YESTERDAY },
  { text: '10', go: YESTERDAY },
  { text: '11-10-2026', refusal: 'future' },
  { text: '11', refusal: 'future' },
  { text: '31', refusal: 'future' },
  { text: '29-02-2028', refusal: 'future' },
  { text: '01-01-2100', refusal: 'future' },
  { text: '31-02-2026', refusal: 'invalid' },
  { text: '29-02-2027', refusal: 'invalid' },
  { text: '0', refusal: 'invalid' },
  { text: '32', refusal: 'invalid' },
  { text: '00-00-0000', refusal: 'invalid' },
  { text: '99-99-9999', refusal: 'invalid' },
  { text: '01-01-0050', refusal: 'invalid' },
  { text: '01-01-10000', refusal: 'invalid' },
  { text: 'abc', refusal: 'invalid' },
  { text: '1e5', refusal: 'invalid' },
  { text: '-1', refusal: 'invalid' },
  { text: '١٠-١٠-٢٠٢٦', refusal: 'invalid' },
  { text: "10-10-2026'; DROP TABLE bikri_entries; --", refusal: 'invalid' },
  { text: '9'.repeat(300), refusal: 'invalid' },
];

const SEARCHES = ['1', '2', '4', '999', 'zzz', 'LES'];

// ---- the screen ----------------------------------------------------------------------------------

/** The ids of the rows and headings of a grid, in the order they stand. */
function rowIds(grid: HTMLElement): (string | null)[] {
  return Array.from(grid.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)'))
    .sort((a, b) => Number(a.getAttribute('row-index')) - Number(b.getAttribute('row-index')))
    .map((row) => row.getAttribute('row-id'));
}

const outcomes: Record<string, number> = {};
function counted(kind: string): void {
  outcomes[kind] = (outcomes[kind] ?? 0) + 1;
}

/** What the grid of the window must hold, in order: each day's heading, then its rows on this page. */
function expectedWindowIds(world: World, state: State, page: number): string[] {
  const filter = { all: state.all, show_finished: false, search: state.search };
  const rows = rowsFor(world, filter, windowFrom(state.end), state.end);
  const pageCount = Math.max(1, Math.ceil(rows.length / PAGE_ROWS));
  const onPage = rows.slice((page - 1) * PAGE_ROWS, page * PAGE_ROWS);
  const days = daysOfWindow(state.end);
  const from = page <= 1 ? 0 : days.indexOf(onPage[0]?.entry_date ?? '');
  const to =
    page >= pageCount ? days.length - 1 : days.indexOf(onPage[onPage.length - 1]?.entry_date ?? '');
  const ids: string[] = [];
  for (const day of days.slice(Math.max(from, 0), to + 1)) {
    ids.push(`group:${day}`);
    for (const row of onPage) if (row.entry_date === day) ids.push(String(row.id));
  }
  return ids;
}

interface Trial {
  world: World;
  state: State;
  user: ReturnType<typeof userEvent.setup>;
  router: Awaited<ReturnType<typeof renderApp>>['router'];
  plan: ReturnType<typeof fakePlan16>;
  asked: BikriListInput[];
  /** What the trial has already done, for the message of a failure. */
  steps: string[];
}

const label = (t: Trial) => `steps: ${t.steps.join(' | ')}`;
const m = (t: Trial, id: string, vars: Record<string, string | number> = {}) =>
  message(t.state.lang, id, vars);
const windowGrid = (t: Trial) =>
  screen.queryByRole('group', { name: named(m(t, 'bikri-past-label')) });
const todayGrid = (t: Trial) =>
  screen.queryByRole('group', { name: named(m(t, 'bikri-list-label')) });
const pageText = () => visible(document.body.textContent);
const button = (t: Trial, id: string) =>
  screen.getByRole('button', {
    name: (name) => visible(name).startsWith(m(t, id, { days: 7 })),
  });
const dateBox = (t: Trial) =>
  screen.getByRole<HTMLInputElement>('textbox', { name: named(m(t, 'bikri-past-goto')) });
const searchBox = (t: Trial) =>
  screen.getByRole<HTMLInputElement>('textbox', { name: named(m(t, 'common-search-label')) });
const tickBox = (t: Trial) =>
  screen.getByRole<HTMLInputElement>('checkbox', { name: named(m(t, 'bikri-past-show-all')) });
const windowCalls = (t: Trial) => t.asked.filter((input) => input.date === null);

/** Rule 1: every call that has been made is a well-formed one. Checked at once, never waited for. */
function checkCalls(t: Trial): void {
  for (const input of t.asked) {
    if (input.date !== null) {
      expect(isoOf(input.date), `today's call asked ${input.date}. ${label(t)}`).toBe(TODAY);
      expect(input.from, label(t)).toBeNull();
      expect(input.group_by_day, label(t)).toBe(false);
    } else {
      expect(input.from, `a call with no day at all. ${label(t)}`).not.toBeNull();
      expect(input.to, `a call with no day at all. ${label(t)}`).not.toBeNull();
      const to = isoOf(input.to);
      expect(to <= YESTERDAY, `a window ends ${to}, after yesterday. ${label(t)}`).toBe(true);
      expect(isoOf(input.from), `a window is not 7 days. ${label(t)}`).toBe(dayAfter(to, -6));
      expect(input.group_by_day, `a window must ask for its days. ${label(t)}`).toBe(true);
    }
  }
}

/** Rules 2 and 3: the screen has settled on the window of the model. */
async function settle(t: Trial): Promise<void> {
  checkCalls(t);
  const { state, world } = t;
  expect(screen.queryByRole('alertdialog'), `the leave box opened. ${label(t)}`).toBeNull();
  await waitFor(() => {
    const last = windowCalls(t).at(-1);
    expect(last, label(t)).toBeDefined();
    expect(isoOf(last?.to ?? null), `the window the screen asked for. ${label(t)}`).toBe(state.end);
    expect(isoOf(last?.from ?? null), label(t)).toBe(windowFrom(state.end));
    expect(last?.all, label(t)).toBe(state.all);
  }, SLOW);
  const filter = { all: state.all, show_finished: false, search: state.search };
  const inWindow = rowsFor(world, filter, windowFrom(state.end), state.end);
  const todays = rowsFor(world, filter, TODAY, TODAY);
  await waitFor(() => {
    expect(t.router.state.location.pathname, label(t)).toBe('/bikri/today');
    const search: Record<string, unknown> = t.router.state.location.search;
    // An address the screen was given and has not yet rewritten (a day that is not real, or after
    // yesterday) is kept as it was while the window is yesterday's: it is read, not used.
    const given = search['end'];
    const keptAsGiven =
      typeof given === 'string' &&
      state.end === YESTERDAY &&
      (given > YESTERDAY || !/^\d{4}-\d{2}-\d{2}$/.test(given) || dayAfter(given, 0) !== given);
    const end = typeof given === 'string' ? given : YESTERDAY;
    if (!keptAsGiven) expect(end, `the address window. ${label(t)}`).toBe(state.end);
    expect(search['all'] === true, `the address switch. ${label(t)}`).toBe(state.all);
    const grid = windowGrid(t);
    if (inWindow.length === 0) {
      expect(grid, `a window with no بکری draws no grid. ${label(t)}`).toBeNull();
      const said =
        pageText().includes(m(t, 'bikri-past-window-empty')) ||
        pageText().includes(m(t, 'bikri-list-no-match'));
      expect(said, `an empty window says so in words. ${label(t)}`).toBe(true);
      return;
    }
    expect(grid, `the window grid. ${label(t)}`).not.toBeNull();
    if (grid === null) return;
    // The grid draws only the rows near the top while the page has no height (jsdom), so the rows on
    // screen must be the first rows of what the backend sent, in its order, and not fewer than 8 of them.
    const wanted = expectedWindowIds(world, state, 1);
    const shown = rowIds(grid);
    expect(shown, `the window of ${state.end}. ${label(t)}`).toEqual(wanted.slice(0, shown.length));
    expect(shown.length, `rows drawn. ${label(t)}`).toBeGreaterThanOrEqual(
      Math.min(wanted.length, 8),
    );
    // Each heading has the backend's count and خام بکری, and a day with none says so.
    for (const day of daysOfWindow(state.end)) {
      const ofDay = inWindow.filter((row) => row.entry_date === day);
      if (!rowIds(grid).includes(`group:${day}`)) continue;
      const heading = grid.querySelector(`.ag-row[row-id="group:${day}"]`);
      const chips = Array.from(heading?.querySelectorAll('[data-heading-chip]') ?? []).map((chip) =>
        visible(chip.textContent),
      );
      if (ofDay.length === 0) {
        expect(chips, `the empty day ${day}. ${label(t)}`).toEqual([]);
      } else {
        const gross = ofDay.reduce((sum, row) => sum + (world.rupees.get(row.id) ?? 0), 0);
        expect(chips, `the figures of ${day}. ${label(t)}`).toEqual([
          `${m(t, 'bikri-past-day-sales')}${String(ofDay.length)}`,
          `${m(t, 'glossary-kham-bikri')}${rupeesText(gross)}`,
        ]);
      }
    }
  }, SLOW);
  // Today's part always shows today's rows.
  await waitFor(() => {
    const grid = todayGrid(t);
    if (todays.length === 0) {
      expect(grid, label(t)).toBeNull();
      return;
    }
    expect(grid, `today's grid. ${label(t)}`).not.toBeNull();
    if (grid !== null) {
      expect(rowIds(grid), `today's rows. ${label(t)}`).toEqual(
        todays.map((row) => String(row.id)),
      );
    }
  }, SLOW);
}

/** Focuses a row of a grid by clicking its number cell, and returns its id. */
async function focusRow(t: Trial, grid: HTMLElement, id?: number): Promise<number | null> {
  const ids = rowIds(grid).filter(
    (row): row is string => row !== null && !row.startsWith('group:'),
  );
  const chosen = id === undefined ? ids[0] : String(id);
  if (chosen === undefined || !ids.includes(chosen)) return null;
  const cell = grid.querySelector<HTMLElement>(`.ag-row[row-id="${chosen}"] [col-id="number"]`);
  if (cell === null) return null;
  await t.user.click(cell);
  return Number(chosen);
}

const stepEarlier = (t: Trial) => {
  t.state.end = dayAfter(t.state.end, -7);
};
const stepNearer = (t: Trial) => {
  const next = dayAfter(t.state.end, 7);
  t.state.end = next > YESTERDAY ? YESTERDAY : next;
};

async function clearSearch(t: Trial) {
  if (t.state.search === '') return;
  await t.user.clear(searchBox(t));
  t.state.search = '';
}

/** Gets to the latest week with nothing typed, so row 41 (yesterday) is in the window. */
async function toRow41(t: Trial) {
  await clearSearch(t);
  if (t.state.end !== YESTERDAY) {
    await t.user.click(
      screen.getByRole('button', { name: named(m(t, 'bikri-past-nav-latest-button')) }),
    );
    t.state.end = YESTERDAY;
  }
  await settle(t);
}

const actionsOf = (part: number): HTMLElement => {
  const found = document.querySelectorAll<HTMLElement>('[data-past-row-actions]')[part];
  if (found === undefined) throw new Error(`no actions block ${String(part)}`);
  return found;
};
const buttonIn = (t: Trial, part: number, id: string) =>
  within(actionsOf(part)).getByRole('button', {
    name: (name) => visible(name).startsWith(m(t, id)),
  });

/** Presses Esc, up to three times, until the screen is no longer `from`. Returns how many it took. */
async function escapeUntilLeft(t: Trial, from: string): Promise<number> {
  for (let press = 1; press <= 3; press += 1) {
    await t.user.keyboard('{Escape}');
    try {
      await waitFor(
        () => {
          expect(t.router.state.location.pathname).not.toBe(from);
        },
        { timeout: 1_500 },
      );
      return press;
    } catch {
      // The press closed a list or stopped a typing, as it should; the next one leaves.
    }
  }
  throw new Error(`Esc three times and the screen is still ${from}. ${label(t)}`);
}

// ---- the things a trial does ---------------------------------------------------------------------

const OPS = [
  'earlier',
  'earlier',
  'nearer',
  'pageUp',
  'pageDown',
  'arrow',
  'latest',
  'goto',
  'goto',
  'goto',
  'tick',
  'search',
  'deleteKeys',
  'todayPageKeys',
  'openButton',
  'openEsc',
  'editEsc',
  'editSave',
  'cancelEsc',
  'cancel',
  'language',
] as const;
type Op = (typeof OPS)[number];

async function perform(t: Trial, op: Op, random: Random): Promise<void> {
  const { user, state } = t;
  switch (op) {
    case 'earlier': {
      await user.click(button(t, 'bikri-past-back'));
      stepEarlier(t);
      break;
    }
    case 'nearer': {
      const off = state.end === YESTERDAY;
      if (off) expect(button(t, 'bikri-past-forward'), label(t)).toBeDisabled();
      await user.click(button(t, 'bikri-past-forward'));
      stepNearer(t);
      break;
    }
    case 'pageUp':
    case 'pageDown': {
      const grid = windowGrid(t);
      if (grid === null || (await focusRow(t, grid)) === null) {
        // No row to focus: the bar has the same keys through its buttons.
        await user.click(button(t, 'bikri-past-back'));
        stepEarlier(t);
        t.steps[t.steps.length - 1] = 'earlier (no row for Page keys)';
        break;
      }
      await user.keyboard(op === 'pageUp' ? '{PageUp}' : '{PageDown}');
      if (op === 'pageUp') stepEarlier(t);
      else stepNearer(t);
      break;
    }
    case 'arrow': {
      const back = state.lang === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';
      const nearer = state.lang === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
      button(t, 'bikri-past-back').focus();
      if (random.below(2) === 0) {
        await user.keyboard(back);
        stepEarlier(t);
      } else {
        await user.keyboard(nearer);
        stepNearer(t);
      }
      break;
    }
    case 'latest': {
      const latest = screen.queryByRole('button', {
        name: named(m(t, 'bikri-past-nav-latest-button')),
      });
      if (state.end === YESTERDAY) {
        expect(latest, `the latest-week button on the latest week. ${label(t)}`).toBeNull();
      } else {
        expect(latest, `no latest-week button on an earlier week. ${label(t)}`).not.toBeNull();
        if (latest !== null) await user.click(latest);
        state.end = YESTERDAY;
      }
      break;
    }
    case 'goto': {
      const wanted = random.pick(GOTO);
      t.steps[t.steps.length - 1] = `goto "${wanted.text.slice(0, 20)}"`;
      const box = dateBox(t);
      await user.click(box);
      await user.clear(box);
      // A paste, because the minus key of a real keyboard is the previous-day key (see the probe test).
      await user.paste(wanted.text);
      const calls = windowCalls(t).length;
      await user.keyboard(random.below(2) === 0 ? '{Enter}' : '{Tab}');
      if (wanted.go !== undefined) {
        state.end = wanted.go;
        counted('goto-went');
        await waitFor(() => {
          expect(box, label(t)).toHaveValue('');
        }, SLOW);
      } else {
        counted(`goto-${wanted.refusal ?? 'x'}`);
        const words =
          wanted.refusal === 'future'
            ? m(t, 'err-date-in-future', { today: '10-10-2026' })
            : m(t, 'err-date-invalid');
        await waitFor(() => {
          expect(pageText(), `a refusal in words. ${label(t)}`).toContain(words);
        }, SLOW);
        // Nothing was asked and the window did not move.
        expect(windowCalls(t).length, `a refused day asked the backend. ${label(t)}`).toBe(calls);
        await user.clear(box);
      }
      break;
    }
    case 'tick': {
      await user.click(tickBox(t));
      state.all = !state.all;
      break;
    }
    case 'search': {
      const text = random.pick(SEARCHES);
      t.steps[t.steps.length - 1] = `search "${text}"`;
      const box = searchBox(t);
      await user.clear(box);
      await user.type(box, text);
      state.search = text;
      break;
    }
    case 'deleteKeys': {
      const grid = windowGrid(t) ?? todayGrid(t);
      if (grid !== null) await focusRow(t, grid);
      const before = t.plan.cancels.length + t.plan.editSaves.length;
      await user.keyboard('{Backspace}{Delete}{Backspace}');
      expect(screen.queryByRole('alertdialog'), `Delete opened a box. ${label(t)}`).toBeNull();
      expect(t.router.state.location.pathname, `Delete left the list. ${label(t)}`).toBe(
        '/bikri/today',
      );
      expect(t.plan.cancels.length + t.plan.editSaves.length, `Delete wrote. ${label(t)}`).toBe(
        before,
      );
      counted('delete-keys');
      break;
    }
    case 'todayPageKeys': {
      const grid = todayGrid(t);
      if (grid === null || (await focusRow(t, grid)) === null) {
        t.steps[t.steps.length - 1] = 'todayPageKeys (no row today)';
        break;
      }
      const calls = windowCalls(t).length;
      await user.keyboard('{PageUp}{PageDown}{PageUp}');
      expect(
        windowCalls(t).length,
        `a Page key in today's part moved the window. ${label(t)}`,
      ).toBe(calls);
      break;
    }
    case 'openButton':
    case 'openEsc': {
      const grid = windowGrid(t);
      const id = grid === null ? null : await focusRow(t, grid);
      if (id === null) {
        t.steps[t.steps.length - 1] = 'open (no row)';
        break;
      }
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(t.router.state.location.pathname, label(t)).toBe('/bikri/book');
      }, SLOW);
      const search: Record<string, unknown> = t.router.state.location.search;
      expect(search['id'], label(t)).toBe(id);
      expect(search['past'], `the way back keeps the window. ${label(t)}`).toBe(state.end);
      expect(search['pastAll'] === true, `the way back keeps the switch. ${label(t)}`).toBe(
        state.all,
      );
      await screen.findByRole(
        'heading',
        { level: 1, name: named(m(t, 'bikri-saved-title', { number: 1 })) },
        SLOW,
      );
      const calls = t.asked.length;
      if (op === 'openButton') {
        t.steps[t.steps.length - 1] = 'open (back button)';
        await user.click(
          await screen.findByRole('button', { name: named(m(t, 'bikri-past-back-to-list')) }),
        );
      } else {
        t.steps[t.steps.length - 1] = 'open (Esc)';
        counted(`open-left-after-${String(await escapeUntilLeft(t, '/bikri/book'))}-esc`);
      }
      await waitFor(() => {
        expect(t.router.state.location.pathname, `the way back. ${label(t)}`).toBe('/bikri/today');
      }, SLOW);
      expect(t.router.state.location.search, label(t)).toMatchObject({ end: state.end, row: id });
      await waitFor(() => {
        expect(t.asked.length, `the list asked again. ${label(t)}`).toBeGreaterThan(calls);
      }, SLOW);
      counted('opened-and-came-back');
      break;
    }
    case 'editEsc':
    case 'editSave': {
      await toRow41(t);
      const grid = windowGrid(t);
      if (t.world.rows.find((row) => row.id === 41)?.status === 'cancelled') {
        // A cancelled بکری has no Edit: both buttons are off, with the reason in words.
        t.steps[t.steps.length - 1] = 'edit (row 41 is cancelled)';
        if (grid !== null && (await focusRow(t, grid, 41)) !== null) {
          expect(buttonIn(t, 1, 'edit-action-edit'), label(t)).toBeDisabled();
          expect(buttonIn(t, 1, 'edit-action-cancel'), label(t)).toBeDisabled();
          counted('cancelled-row-has-no-buttons');
        }
        break;
      }
      if (grid === null || (await focusRow(t, grid, 41)) === null) {
        t.steps[t.steps.length - 1] = 'edit (row 41 is not shown)';
        break;
      }
      await user.click(buttonIn(t, 1, 'edit-action-edit'));
      await waitFor(() => {
        expect(t.router.state.location.pathname, label(t)).toBe('/bikri/book');
      }, SLOW);
      expect(t.router.state.location.search, label(t)).toMatchObject({
        id: 41,
        edit: 7,
        past: YESTERDAY,
      });
      await screen.findByRole(
        'heading',
        { level: 1, name: named(m(t, 'bikri-edit-title', { number: 1 })) },
        SLOW,
      );
      await waitFor(() => {
        expect(cellOf(salesGrid(state.lang), 0, 'rate')).toBeInTheDocument();
      }, SLOW);
      const calls = t.asked.length;
      if (op === 'editEsc') {
        // Out with no change: nothing is asked, nothing is written.
        t.steps[t.steps.length - 1] = 'edit (Esc)';
        // Esc answers in the order of PRD FR-7.18: an open list first, then the typing in a cell, then it
        // leaves. So it may take up to three presses, and no press may write anything.
        const presses = await escapeUntilLeft(t, '/bikri/book');
        expect(t.plan.editSaves, `Esc saved. ${label(t)}`).toHaveLength(0);
        counted(`edit-left-after-${String(presses)}-esc`);
        counted('edit-left');
      } else {
        t.steps[t.steps.length - 1] = 'edit (rate, F10)';
        await user.click(cellOf(salesGrid(state.lang), 0, 'rate'));
        await user.keyboard('610{Enter}');
        await waitFor(() => {
          expect(cellOf(salesGrid(state.lang), 0, 'rate')).toHaveClass('cell-changed');
        }, SLOW);
        press('F10');
        await waitFor(() => {
          expect(t.plan.editSaves, label(t)).toHaveLength(1);
        }, SLOW);
        counted('edit-saved');
      }
      await waitFor(() => {
        expect(t.router.state.location.pathname, `the way back. ${label(t)}`).toBe('/bikri/today');
      }, SLOW);
      expect(t.router.state.location.search, label(t)).toMatchObject({ end: YESTERDAY, row: 41 });
      await waitFor(() => {
        expect(t.asked.length, `the list asked again. ${label(t)}`).toBeGreaterThan(calls);
      }, SLOW);
      await waitFor(() => {
        expect(
          windowGrid(t)?.querySelector('.ag-row[row-id="41"] .ag-cell-focus'),
          `the row has focus again. ${label(t)}`,
        ).not.toBeNull();
      }, SLOW);
      break;
    }
    case 'cancelEsc':
    case 'cancel': {
      await toRow41(t);
      const row = t.world.rows.find((candidate) => candidate.id === 41);
      const grid = windowGrid(t);
      if (row === undefined || row.status === 'cancelled' || grid === null) {
        t.steps[t.steps.length - 1] = 'cancel (row 41 is gone)';
        break;
      }
      if ((await focusRow(t, grid, 41)) === null) {
        t.steps[t.steps.length - 1] = 'cancel (row 41 is not shown)';
        break;
      }
      await user.click(buttonIn(t, 1, 'edit-action-cancel'));
      const dialog = await screen.findByRole('alertdialog');
      if (op === 'cancelEsc') {
        // Out of the box with Esc: nothing is cancelled.
        await user.keyboard('{Escape}');
        await waitFor(() => {
          expect(screen.queryByRole('alertdialog')).toBeNull();
        }, SLOW);
        expect(t.plan.cancels, `Esc cancelled. ${label(t)}`).toHaveLength(0);
        counted('cancel-left');
        break;
      }
      await user.type(
        within(dialog).getByRole('combobox', { name: named(m(t, 'bikri-cancel-reason')) }),
        'entered twice{Enter}',
      );
      await waitFor(() => {
        expect(t.plan.cancels, label(t)).toHaveLength(1);
      }, SLOW);
      expect(t.plan.cancels[0], label(t)).toMatchObject({ id: 41, reason: 'entered twice' });
      counted('cancelled');
      break;
    }
    case 'language': {
      const other: Language = state.lang === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      state.lang = other;
      counted('language');
      break;
    }
  }
}

/** Opens the screen on a world, at a start address, with every double the screen needs. */
async function startTrial(
  language: Language,
  world: World,
  start: { search: string; end: string; all: boolean },
): Promise<Trial> {
  fakeBikri();
  const plan = fakePlan16();
  const asked: BikriListInput[] = [];
  vi.mocked(bikriApi.bikriList).mockImplementation((input) => {
    asked.push(input);
    return Promise.resolve(answerList(world, input));
  });
  // The backend cancels the row in the same call, so the list the screen asks for next already
  // shows it cancelled and locked.
  const realCancel = vi.mocked(bikriApi.bikriCancel).getMockImplementation();
  vi.mocked(bikriApi.bikriCancel).mockImplementation((...args) => {
    const index = world.rows.findIndex((row) => row.id === args[0]);
    const old = world.rows[index];
    if (old !== undefined) {
      world.rows[index] = { ...old, status: 'cancelled', ...lockedEntry('cancelled') };
    }
    if (realCancel === undefined) throw new Error('the cancel double is not installed');
    return realCancel(...args);
  });
  vi.mocked(bikriApi.bikriGet).mockImplementation((id) =>
    Promise.resolve({ ...savedEvent1(), id }),
  );
  const { router } = await renderApp(`/bikri/today${start.search}`);
  const trial: Trial = {
    world,
    state: { end: start.end, all: start.all, search: '', lang: language },
    user: userEvent.setup(),
    router,
    plan,
    asked,
    steps: [`start "${start.search}"`],
  };
  await screen.findByRole('group', { name: named(m(trial, 'bikri-past-nav-label')) }, SLOW);
  await settle(trial);
  return trial;
}

describe.each(LANGUAGES)('پچھلی بکری under random keys and input (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  // PAST_FUZZ_ONLY="ur:1" runs one trial again, to look at a failure by itself.
  const only = process.env['PAST_FUZZ_ONLY'];
  it.each(
    Array.from({ length: TRIALS }, (_, i) => i + 1).filter(
      (seed) => only === undefined || only === `${language}:${String(seed)}`,
    ),
  )(
    'seed %i',
    async (seed) => {
      const random = rng(seed * 11 + (language === 'ur' ? 3 : 4));
      const world = buildWorld(random, seed % 4 === 0);
      const start = random.pick(STARTS);
      const trial = await startTrial(language, world, start);
      // PAST_FUZZ_OPS="tick,edit" does exactly those things, in that order, to look at a failure.
      const forced = process.env['PAST_FUZZ_OPS']
        ?.split(',')
        .filter((op): op is Op => (OPS as readonly string[]).includes(op));
      const operations = forced?.length ?? 4 + random.below(5);
      for (let n = 0; n < operations; n += 1) {
        const op = forced?.[n] ?? random.pick(OPS);
        trial.steps.push(op);
        await perform(trial, op, random);
        await settle(trial);
      }
      counted('trial');
    },
    180_000,
  );
});

describe.each(LANGUAGES)('پچھلی بکری when a key or a click arrives twice (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const LATEST = { search: '', end: YESTERDAY, all: false };

  // Found by this gate (plan 72) and fixed: the reason box sent the cancel on every Enter, and its "sending"
  // state came only after the next draw, so two Enters at once sent two `bikri_cancel` calls. The backend
  // refused the second, but the Owner could be told of a refusal after a cancel that worked. The cancel now
  // runs in `useSingleFlight`, like the save flows.
  it('Enter twice at once in the reason box cancels the بکری exactly once', async () => {
    const trial = await startTrial(language, buildWorld(rng(5), false), LATEST);
    const grid = windowGrid(trial);
    expect(grid).not.toBeNull();
    if (grid === null) return;
    await focusRow(trial, grid, 41);
    await trial.user.click(buttonIn(trial, 1, 'edit-action-cancel'));
    const dialog = await screen.findByRole('alertdialog');
    const reason = within(dialog).getByRole('combobox', {
      name: named(m(trial, 'bikri-cancel-reason')),
    });
    await trial.user.type(reason, 'entered twice');
    // Both keys arrive before the screen has drawn again, as a double press does.
    act(() => {
      fireEvent.keyDown(reason, { key: 'Enter', code: 'Enter' });
      fireEvent.keyDown(reason, { key: 'Enter', code: 'Enter' });
    });
    await waitFor(() => {
      expect(trial.plan.cancels.length).toBeGreaterThanOrEqual(1);
    }, SLOW);
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(trial.plan.cancels, 'one cancel for two presses').toHaveLength(1);
    expect(trial.router.state.location.pathname).toBe('/bikri/today');
  });

  it('two clicks on Edit at once open one edit screen, and the way back is still the list', async () => {
    const trial = await startTrial(language, buildWorld(rng(6), false), LATEST);
    const grid = windowGrid(trial);
    expect(grid).not.toBeNull();
    if (grid === null) return;
    await focusRow(trial, grid, 41);
    const edit = buttonIn(trial, 1, 'edit-action-edit');
    act(() => {
      fireEvent.click(edit);
      fireEvent.click(edit);
    });
    await waitFor(() => {
      expect(trial.router.state.location.pathname).toBe('/bikri/book');
    }, SLOW);
    await screen.findByRole(
      'heading',
      { level: 1, name: named(m(trial, 'bikri-edit-title', { number: 1 })) },
      SLOW,
    );
    expect(screen.queryByRole('alert'), 'an error was told').toBeNull();
    expect(trial.router.state.location.search).toMatchObject({ id: 41, edit: 7, past: YESTERDAY });
    await escapeUntilLeft(trial, '/bikri/book');
    expect(trial.router.state.location.search).toMatchObject({ end: YESTERDAY, row: 41 });
  });

  it('Enter twice at once on a row opens the بکری once, and Esc comes back to the same row', async () => {
    const trial = await startTrial(language, buildWorld(rng(7), false), LATEST);
    const grid = windowGrid(trial);
    expect(grid).not.toBeNull();
    if (grid === null) return;
    await focusRow(trial, grid, 41);
    act(() => {
      fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Enter', code: 'Enter' });
      fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Enter', code: 'Enter' });
    });
    await waitFor(() => {
      expect(trial.router.state.location.pathname).toBe('/bikri/book');
    }, SLOW);
    await screen.findByRole(
      'heading',
      { level: 1, name: named(m(trial, 'bikri-saved-title', { number: 1 })) },
      SLOW,
    );
    expect(trial.router.state.location.search).toMatchObject({ id: 41, past: YESTERDAY });
    await escapeUntilLeft(trial, '/bikri/book');
    expect(trial.router.state.location.search).toMatchObject({ end: YESTERDAY, row: 41 });
  });
});

describe.each(LANGUAGES)(
  'پچھلی بکری rows of a closed day and rows that are locked (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    const withRows = () => {
      const world = buildWorld(rng(9), false);
      const day = dayAfter(YESTERDAY, -1);
      for (const [id, more] of [
        [700, { needs_reopen: true }],
        [701, { ...lockedEntry('goods_moved') }],
      ] as const) {
        world.rupees.set(id, 5_000);
        world.rows.push(
          listRow({
            id,
            number: id,
            entry_date: day,
            truck_no: null,
            gross: money(5_000),
            ...more,
          }),
        );
      }
      return world;
    };

    it('a closed-day row reads "reopen the day and fix"; Esc in its box reopens nothing and keeps the list', async () => {
      const trial = await startTrial(language, withRows(), {
        search: '',
        end: YESTERDAY,
        all: false,
      });
      const grid = windowGrid(trial);
      expect(grid).not.toBeNull();
      if (grid === null) return;
      await focusRow(trial, grid, 700);
      const fix = buttonIn(trial, 1, 'edit-action-fix-day');
      expect(fix).toBeEnabled();
      await trial.user.click(fix);
      const dialog = await screen.findByRole('alertdialog');
      expect(
        within(dialog).getByRole('combobox', { name: named(m(trial, 'edit-fix-label')) }),
      ).toBeVisible();
      await trial.user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      }, SLOW);
      expect(editApi.dayFixOpen, 'Esc reopened a day').not.toHaveBeenCalled();
      expect(trial.router.state.location.pathname).toBe('/bikri/today');
      await settle(trial);
    });

    it('a locked row says why in words, both buttons stay off, and Enter, Delete and clicks on them change nothing', async () => {
      const trial = await startTrial(language, withRows(), {
        search: '',
        end: YESTERDAY,
        all: false,
      });
      const grid = windowGrid(trial);
      expect(grid).not.toBeNull();
      if (grid === null) return;
      await focusRow(trial, grid, 701);
      await waitFor(() => {
        expect(visible(actionsOf(1).textContent)).toContain(m(trial, 'edit-locked-goods-moved'));
      }, SLOW);
      const edit = buttonIn(trial, 1, 'edit-action-edit');
      const cancel = buttonIn(trial, 1, 'edit-action-cancel');
      expect(edit).toBeDisabled();
      expect(cancel).toBeDisabled();
      await trial.user.click(edit);
      await trial.user.click(cancel);
      await trial.user.keyboard('{Delete}{Backspace}');
      expect(screen.queryByRole('alertdialog')).toBeNull();
      expect(trial.router.state.location.pathname).toBe('/bikri/today');
      expect(trial.plan.cancels).toHaveLength(0);
      expect(trial.plan.editSaves).toHaveLength(0);
      expect(editApi.dayFixOpen).not.toHaveBeenCalled();
      // Enter still opens it to look at (read only), and Esc comes back to the same row.
      await focusRow(trial, grid, 701);
      await trial.user.keyboard('{Enter}');
      await waitFor(() => {
        expect(trial.router.state.location.pathname).toBe('/bikri/book');
      }, SLOW);
      await escapeUntilLeft(trial, '/bikri/book');
      expect(trial.router.state.location.search).toMatchObject({ end: YESTERDAY, row: 701 });
    });
  },
);

describe.each(LANGUAGES)('پچھلی بکری with 30 بکری on one day (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('the page buttons turn the rows of the window, Page Up still moves the window, and the new window starts on page 1', async () => {
    const world = buildWorld(rng(8), true);
    const trial = await startTrial(language, world, { search: '', end: YESTERDAY, all: false });
    const pagerNext = () =>
      screen.getAllByRole('button', {
        name: (name) => visible(name) === m(trial, 'common-pager-next'),
      });
    // The busy day is 08-10-2026, in the window that ends yesterday: more than one page of rows.
    await waitFor(() => {
      expect(pagerNext().length).toBeGreaterThan(0);
    }, SLOW);
    const next = pagerNext().at(-1);
    expect(next).toBeDefined();
    if (next === undefined) return;
    await trial.user.click(next);
    const filter = { all: false, show_finished: false, search: '' };
    const inWindow = rowsFor(world, filter, windowFrom(YESTERDAY), YESTERDAY);
    const wanted = expectedWindowIds(world, trial.state, 2);
    await waitFor(() => {
      const grid = windowGrid(trial);
      expect(grid, 'the window grid on page 2').not.toBeNull();
      if (grid === null) return;
      const shown = rowIds(grid);
      expect(shown, `page 2 of ${String(inWindow.length)} rows`).toEqual(
        wanted.slice(0, shown.length),
      );
      expect(shown.some((id) => id !== null && !id.startsWith('group:'))).toBe(true);
    }, SLOW);
    // Page Up in the window grid goes back 7 days (the window), not to page 1 of the rows.
    const grid = windowGrid(trial);
    if (grid === null) return;
    await focusRow(trial, grid);
    await trial.user.keyboard('{PageUp}');
    stepEarlier(trial);
    await settle(trial);
    counted('pager-then-window');
  });
});

describe('the date box of "تاریخ پر جائیں" with the keys of a real keyboard', () => {
  beforeEach(() => {
    resetClientState('en');
    signedIn(sessionUser({ language: 'en' }));
  });

  // Found by this gate (plan 72) and fixed: user-event gives the character "-" no key code, so the tests that
  // type "05-10-2026" never met the real key. On a real keyboard the "-" key has the code `Minus`, which the
  // date field took as "previous day" (FR-7.19) and swallowed, so the dash of "5-9" could never be typed.
  // Now "-" and "+" are shortcuts only on an empty field or when all the text is selected. This test sends
  // the real key code.
  it('the dash of "5-9" can be typed: the minus key is not swallowed after a digit', async () => {
    const world = buildWorld(rng(1), false);
    fakeBikri();
    vi.mocked(bikriApi.bikriList).mockImplementation((input) =>
      Promise.resolve(answerList(world, input)),
    );
    await renderApp('/bikri/today');
    const box = await screen.findByRole<HTMLInputElement>(
      'textbox',
      { name: named(message('en', 'bikri-past-goto')) },
      SLOW,
    );
    const user = userEvent.setup();
    await user.click(box);
    await user.keyboard('5');
    // The real key: the code is `Minus`. `fireEvent` says false when the page stopped it.
    const letThrough = fireEvent.keyDown(box, { code: 'Minus', key: '-' });
    expect(letThrough, 'the minus key was swallowed by the date field').toBe(true);
    expect(box.value).toBe('5');
  });
});

afterAll(() => {
  process.stdout.write(`past-sales-fuzz: ${JSON.stringify(outcomes)}\n`);
  // A pass that never went to a day, never saved an edit and never cancelled cannot hide.
  if ((outcomes['trial'] ?? 0) >= TRIALS * LANGUAGES.length) {
    expect(outcomes['goto-went'] ?? 0).toBeGreaterThanOrEqual(3);
    expect(outcomes['goto-invalid'] ?? 0).toBeGreaterThanOrEqual(3);
    expect(outcomes['goto-future'] ?? 0).toBeGreaterThanOrEqual(2);
    expect(outcomes['opened-and-came-back'] ?? 0).toBeGreaterThanOrEqual(3);
    expect(outcomes['edit-saved'] ?? 0).toBeGreaterThanOrEqual(1);
    expect(outcomes['cancelled'] ?? 0).toBeGreaterThanOrEqual(1);
    expect(outcomes['delete-keys'] ?? 0).toBeGreaterThanOrEqual(3);
  }
});
